//! DARE Framework CLI (native Rust rewrite).

mod commands;
mod output;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use commands::discover::run_discover;
use commands::info::{collect_info, format_human, report_to_json};
use commands::validate::run_validate;
use commands::welcome::{render_welcome, WelcomeOptions};
use dare_assets::{
    load_capability_matrix_from_str, validate_capability_matrix, verify_embedded_assets,
    EmbeddedAssets,
};
use dare_config::{default_config, load_effective, CliOverrides, EnvOverrides, DEFAULT_CONFIG_REL};
use dare_core::{
    init_tracing, CoreError, CoreResult, ExecutionContext, ProjectRoot, SafeRelativePath,
};
use dare_harness::{
    detect_antigravity, detect_claude, detect_codex, detect_cursor, ensure_workflows_dir,
    generate_agents_md, generate_antigravityrules, generate_claude_md, generate_cursorrules,
    install_antigravity, install_codex_skills, install_commands, install_cursor_commands,
    validate_antigravity_install, validate_codex_install, validate_cursor_install,
    validate_install, write_settings_json,
};
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
    /// Show banner (TTY) and DARE quick-start guide.
    Welcome {
        /// Skip ASCII banner even on TTY.
        #[arg(long)]
        no_banner: bool,
    },
    /// Read-only installation / project diagnostics.
    Info {
        /// Project root hint (default: cwd, walk-up).
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Brownfield project detection and DARE install.
    Discover {
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
        /// Detect only — do not install DARE files.
        #[arg(long)]
        check: bool,
        /// Overwrite managed files when installing.
        #[arg(long)]
        force: bool,
        /// Plan/report without writing files.
        #[arg(long)]
        dry_run: bool,
        /// Abort install when stack conflicts are present.
        #[arg(long)]
        strict_conflicts: bool,
    },
    /// Validate DARE/dare-dag.yaml (read-only).
    Validate {
        /// Path to dare-dag.yaml (default: DARE/dare-dag.yaml under project root).
        #[arg(long)]
        dag: Option<PathBuf>,
        /// Treat warnings as failures.
        #[arg(long)]
        strict: bool,
    },
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
    /// Codex adapter (AGENTS.md + skills).
    Codex {
        #[command(subcommand)]
        action: CodexCmd,
    },
    /// Antigravity adapter (rules + Agent Skills).
    Antigravity {
        #[command(subcommand)]
        action: AntigravityCmd,
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
        /// Overwrite unmanaged CLAUDE.md / commands / settings (default: preserve).
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
    /// Detect .cursor / .cursorrules presence.
    Detect {
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Install managed .cursor/commands from capability matrix.
    Install {
        #[arg(long)]
        root: Option<PathBuf>,
        /// Overwrite unmanaged .cursorrules / commands (default: preserve).
        #[arg(long)]
        force: bool,
    },
    /// Validate installed Cursor commands vs matrix.
    Validate {
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum CodexCmd {
    /// Detect AGENTS.md / .codex / .agents/skills presence.
    Detect {
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Install managed Codex skills + AGENTS.md from capability matrix.
    Install {
        #[arg(long)]
        root: Option<PathBuf>,
        /// Overwrite unmanaged AGENTS.md / skills (default: preserve).
        #[arg(long)]
        force: bool,
    },
    /// Validate installed Codex skills vs matrix.
    Validate {
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum AntigravityCmd {
    /// Detect .antigravityrules / .antigravity / .agents presence.
    Detect {
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Install managed Antigravity rules, workflows, commands and shared skills.
    Install {
        #[arg(long)]
        root: Option<PathBuf>,
        /// Overwrite unmanaged rules / commands / skills (default: preserve).
        #[arg(long)]
        force: bool,
    },
    /// Validate installed Antigravity assets vs matrix.
    Validate {
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

fn project_root(root: Option<PathBuf>) -> CoreResult<ProjectRoot> {
    let root_path =
        root.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
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
        Ok(cli) => match cli.command {
            Some(Commands::Validate { dag, strict }) => run_validate(dag, strict, &renderer),
            other => {
                let cli = Cli {
                    json: cli.json,
                    no_color: cli.no_color,
                    command: other,
                };
                match run(cli) {
                    Ok((msg, data)) => {
                        let _ = renderer.write_success(&msg, data);
                        ExitCode::SUCCESS
                    }
                    Err(e) => exit(renderer.write_error(&e)),
                }
            }
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

fn run(cli: Cli) -> Result<(String, serde_json::Value), CoreError> {
    match cli.command {
        None => {
            let msg =
                "DARE CLI ready. Try: dare welcome | dare info | dare assets verify".to_string();
            ok_msg(msg)
        }
        Some(Commands::Welcome { no_banner }) => {
            let msg = render_welcome(&WelcomeOptions {
                no_banner,
                no_color: cli.no_color,
                stdout_is_tty: None,
            });
            ok_msg(msg)
        }
        Some(Commands::Info { root }) => {
            let cwd = root
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            let report = collect_info(&cwd)?;
            let human = format_human(&report);
            let data = report_to_json(&report);
            Ok((human, data))
        }
        Some(Commands::Discover {
            dir,
            check,
            force,
            dry_run,
            strict_conflicts,
        }) => run_discover(dir, check, force, dry_run, strict_conflicts),
        Some(Commands::Assets {
            action: AssetsCmd::Verify,
        }) => {
            verify_embedded_assets()?;
            ok_msg("assets verify: ok".to_string())
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
            ok_msg(format!(
                "config load: ok (ide={ide}, extra_keys={extras}, path={DEFAULT_CONFIG_REL})"
            ))
        }
        Some(Commands::Capabilities {
            action: CapabilitiesCmd::Validate,
        }) => {
            let file = EmbeddedAssets::get("capability-matrix.yml")
                .ok_or_else(|| CoreError::config("asset missing: capability-matrix.yml"))?;
            let yaml = std::str::from_utf8(file.data.as_ref()).map_err(|e| {
                CoreError::config(format!("invalid capability-matrix encoding: {e}"))
            })?;
            let matrix = load_capability_matrix_from_str(yaml)?;
            validate_capability_matrix(&matrix)?;
            ok_msg(format!(
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
                ok_msg(format!(
                    "harness claude detect: claude_md={} claude_dir={}",
                    d.claude_md, d.claude_dir
                ))
            }
            ClaudeCmd::Install { root, force } => {
                let project = project_root(root)?;
                let _ = generate_claude_md(&project, force);
                let n = install_commands(&project, force)?;
                let _ = write_settings_json(&project, force);
                ok_msg(format!("harness claude install: wrote {n} commands"))
            }
            ClaudeCmd::Validate { root } => {
                let project = project_root(root)?;
                let n = validate_install(&project)?;
                ok_msg(format!("harness claude validate: ok ({n} commands)"))
            }
        },
        Some(Commands::Harness {
            ide: HarnessIde::Cursor { action },
        }) => match action {
            CursorCmd::Detect { root } => {
                let project = project_root(root)?;
                let d = detect_cursor(&project)?;
                ok_msg(format!(
                    "harness cursor detect: cursor_dir={} cursorrules={}",
                    d.cursor_dir, d.cursorrules
                ))
            }
            CursorCmd::Install { root, force } => {
                let project = project_root(root)?;
                let _ = generate_cursorrules(&project, force);
                let n = install_cursor_commands(&project, force)?;
                ok_msg(format!("harness cursor install: wrote {n} commands"))
            }
            CursorCmd::Validate { root } => {
                let project = project_root(root)?;
                let n = validate_cursor_install(&project)?;
                ok_msg(format!("harness cursor validate: ok ({n} commands)"))
            }
        },
        Some(Commands::Harness {
            ide: HarnessIde::Codex { action },
        }) => match action {
            CodexCmd::Detect { root } => {
                let project = project_root(root)?;
                let d = detect_codex(&project)?;
                ok_msg(format!(
                    "harness codex detect: agents_md={} codex_dir={} agents_skills={}",
                    d.agents_md, d.codex_dir, d.agents_skills
                ))
            }
            CodexCmd::Install { root, force } => {
                let project = project_root(root)?;
                generate_agents_md(&project, force)?;
                let n = install_codex_skills(&project, force)?;
                ok_msg(format!(
                    "harness codex install: wrote {n} skills + AGENTS.md"
                ))
            }
            CodexCmd::Validate { root } => {
                let project = project_root(root)?;
                let n = validate_codex_install(&project)?;
                ok_msg(format!("harness codex validate: ok ({n} skills)"))
            }
        },
        Some(Commands::Harness {
            ide: HarnessIde::Antigravity { action },
        }) => match action {
            AntigravityCmd::Detect { root } => {
                let project = project_root(root)?;
                let d = detect_antigravity(&project)?;
                ok_msg(format!(
                    "harness antigravity detect: rules={} dir={} skills={} workflows={}",
                    d.antigravityrules, d.antigravity_dir, d.agents_skills, d.agents_workflows
                ))
            }
            AntigravityCmd::Install { root, force } => {
                let project = project_root(root)?;
                generate_antigravityrules(&project, force)?;
                ensure_workflows_dir(&project, force)?;
                let n = install_antigravity(&project, force)?;
                ok_msg(format!(
                    "harness antigravity install: wrote {n} commands + skills/rules"
                ))
            }
            AntigravityCmd::Validate { root } => {
                let project = project_root(root)?;
                let n = validate_antigravity_install(&project)?;
                ok_msg(format!("harness antigravity validate: ok ({n} commands)"))
            }
        },
        Some(Commands::Validate { .. }) => unreachable!("validate handled in main"),
    }
}

fn ok_msg(msg: String) -> Result<(String, serde_json::Value), CoreError> {
    Ok((msg.clone(), serde_json::json!({ "message": msg })))
}

fn exit(code: i32) -> ExitCode {
    ExitCode::from(code as u8)
}
