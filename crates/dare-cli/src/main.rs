//! DARE Framework CLI (native Rust rewrite).

mod output;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use dare_assets::{
    load_capability_matrix_from_str, validate_capability_matrix, verify_embedded_assets,
    EmbeddedAssets,
};
use dare_config::{
    default_config, load_effective, CliOverrides, EnvOverrides, DEFAULT_CONFIG_REL,
};
use dare_harness::{
    detect_claude, detect_cursor, generate_claude_md, generate_cursorrules, install_commands,
    install_cursor_commands, validate_cursor_install, validate_install, write_settings_json,
};
use dare_core::{init_tracing, CoreError, CoreResult, ExecutionContext, ProjectRoot, SafeRelativePath};
use output::OutputRenderer;

#[derive(Debug, Parser)]
#[command(
    name = "dare",
    version,
    about = "DARE Framework CLI (native Rust rewrite)",
    disable_help_subcommand = true
)]
struct Cli {
    /// Emit JSON envelopes on stdout (ADR-002).
    #[arg(long, global = true)]
    json: bool,

    /// Disable ANSI colors (also honors NO_COLOR).
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Asset inventory / embed checks (microplano 009).
    Assets {
        #[command(subcommand)]
        action: AssetsCmd,
    },
    /// Configuration load / merge smoke (microplano 008).
    Config {
        #[command(subcommand)]
        action: ConfigCmd,
    },
    /// Canonical capabilities matrix (microplano 010 / ADR-007).
    Capabilities {
        #[command(subcommand)]
        action: CapabilitiesCmd,
    },
    /// IDE harness adapters (microplano 011+).
    Harness {
        #[command(subcommand)]
        ide: HarnessIde,
    },
}

#[derive(Debug, Subcommand)]
enum AssetsCmd {
    /// Verify embedded assets against assets/manifest.yml hashes.
    Verify,
}

#[derive(Debug, Subcommand)]
enum ConfigCmd {
    /// Load effective dare.config.json (CLI > env > file > default).
    Load {
        /// Project root (default: cwd).
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum CapabilitiesCmd {
    /// Validate embedded capability-matrix.yml (ids, duplicates, required fields).
    Validate,
}

#[derive(Debug, Subcommand)]
enum HarnessIde {
    /// Claude Code adapter.
    Claude {
        #[command(subcommand)]
        action: ClaudeCmd,
    },
    /// Cursor IDE adapter.
    Cursor {
        #[command(subcommand)]
        action: CursorCmd,
    },
}

#[derive(Debug, Subcommand)]
enum ClaudeCmd {
    /// Detect CLAUDE.md / .claude presence.
    Detect {
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Install managed .claude/commands from capability matrix.
    Install {
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        force: bool,
    },
    /// Validate installed Claude commands vs matrix.
    Validate {
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum CursorCmd {
    Detect {
        #[arg(long)]
        root: Option<PathBuf>,
    },
    Install {
        #[arg(long)]
        root: Option<PathBuf>,
        #[arg(long)]
        force: bool,
    },
    Validate {
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

fn project_root(root: Option<PathBuf>) -> CoreResult<ProjectRoot> {
    let root_path = root.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    ProjectRoot::new(&root_path)
}

fn main() -> ExitCode {
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let json = args.iter().any(|a| a == "--json");
    let no_color = args.iter().any(|a| a == "--no-color");
    let ctx = ExecutionContext::from_cli(json, no_color);
    let _ = init_tracing(&ctx);
    let renderer = OutputRenderer::new(&ctx);

    if args.len() <= 1 {
        use clap::CommandFactory;
        let mut cmd = Cli::command();
        if let Err(e) = cmd.print_help() {
            return exit(renderer.write_error(&CoreError::io(e.to_string())));
        }
        println!();
        return ExitCode::SUCCESS;
    }

    match Cli::try_parse() {
        Ok(cli) => match run(cli) {
            Ok(msg) => {
                let _ = renderer.write_success(&msg, serde_json::json!({ "message": msg }));
                ExitCode::SUCCESS
            }
            Err(e) => exit(renderer.write_error(&e)),
        },
        Err(e) => {
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                let _ = e.print();
                return ExitCode::SUCCESS;
            }
            exit(renderer.write_error(&CoreError::usage(e.to_string())))
        }
    }
}

fn run(cli: Cli) -> Result<String, CoreError> {
    match cli.command {
        None => Ok(
            "DARE CLI ready. Try: dare assets verify | dare config load | dare capabilities validate | dare harness claude validate"
                .into(),
        ),
        Some(Commands::Assets {
            action: AssetsCmd::Verify,
        }) => {
            verify_embedded_assets()?;
            Ok("assets verify: ok".into())
        }
        Some(Commands::Config {
            action: ConfigCmd::Load { root },
        }) => {
            let project = project_root(root)?;
            let rel = SafeRelativePath::new(DEFAULT_CONFIG_REL)?;
            let cfg = load_effective(
                &project,
                &rel,
                &EnvOverrides::default(),
                &CliOverrides::default(),
            )?;
            let ide = cfg.ide.as_deref().unwrap_or("(none)");
            let extras = cfg.extra.len();
            let _ = default_config();
            Ok(format!(
                "config load: ok (ide={ide}, extra_keys={extras}, path={DEFAULT_CONFIG_REL})"
            ))
        }
        Some(Commands::Capabilities {
            action: CapabilitiesCmd::Validate,
        }) => {
            let file = EmbeddedAssets::get("capability-matrix.yml").ok_or_else(|| {
                CoreError::config("asset missing: capability-matrix.yml")
            })?;
            let yaml = std::str::from_utf8(file.data.as_ref())
                .map_err(|e| CoreError::config(format!("invalid capability-matrix encoding: {e}")))?;
            let matrix = load_capability_matrix_from_str(yaml)?;
            validate_capability_matrix(&matrix)?;
            Ok(format!(
                "capabilities validate: ok ({} entries)",
                matrix.capabilities.len()
            ))
        }
        Some(Commands::Harness {
            ide: HarnessIde::Claude { action },
        }) => match action {
            ClaudeCmd::Detect { root } => {
                let project = project_root(root)?;
                let d = detect_claude(&project)?;
                Ok(format!(
                    "harness claude detect: claude_md={} claude_dir={}",
                    d.claude_md, d.claude_dir
                ))
            }
            ClaudeCmd::Install { root, force } => {
                let project = project_root(root)?;
                let _ = generate_claude_md(&project, force);
                let n = install_commands(&project, force)?;
                let _ = write_settings_json(&project, force);
                Ok(format!("harness claude install: wrote {n} commands"))
            }
            ClaudeCmd::Validate { root } => {
                let project = project_root(root)?;
                let n = validate_install(&project)?;
                Ok(format!("harness claude validate: ok ({n} commands)"))
            }
        },
        Some(Commands::Harness {
            ide: HarnessIde::Cursor { action },
        }) => match action {
            CursorCmd::Detect { root } => {
                let project = project_root(root)?;
                let d = detect_cursor(&project)?;
                Ok(format!(
                    "harness cursor detect: cursor_dir={} cursorrules={}",
                    d.cursor_dir, d.cursorrules
                ))
            }
            CursorCmd::Install { root, force } => {
                let project = project_root(root)?;
                let _ = generate_cursorrules(&project, force);
                let n = install_cursor_commands(&project, force)?;
                Ok(format!("harness cursor install: wrote {n} commands"))
            }
            CursorCmd::Validate { root } => {
                let project = project_root(root)?;
                let n = validate_cursor_install(&project)?;
                Ok(format!("harness cursor validate: ok ({n} commands)"))
            }
        },
    }
}

fn exit(code: i32) -> ExitCode {
    ExitCode::from(code as u8)
}
