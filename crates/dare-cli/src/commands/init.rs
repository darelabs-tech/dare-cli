//! `dare init` — npm-parity greenfield bootstrap (cycle rust-init-npm-parity-011).
//!
//! Same prompts, flags, layout and generated files as npm
//! `@dewtech/dare-cli@3.18.1` `commands/init.ts`, minus the documented B-xx bug
//! fixes. Rust-only additions: `--dir`, `--force`, `--check`, `--json`.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use dare_core::{CoreError, CoreResult};
use dare_scaffold::project::console::{Console, StdConsole, Tone};
use dare_scaffold::project::generator::{generate_project_structure, GenerateEnv};
use dare_scaffold::project::tools::SystemToolRunner;
use dare_scaffold::project::{
    npm_slug, resolve_project_output_dir, validate_npm_project_name, BackendStack, FrontendStack,
    GraphragBackend, Ide, McpLanguage, McpTransport, ProjectConfig, RustWorkspaceLayout, Structure,
    ToolchainMode,
};
use serde_json::{json, Value};

use crate::commands::init_prompts::{
    prompt_init_answers, DialoguerPrompter, InitAnswers, Prompter, MSG_INTERACTIVE_REQUIRES_TTY,
};

pub const INIT_REPORT_SCHEMA: u32 = 1;

pub const MSG_NEED_STACK_OR_MCP: &str =
    "Error: --non-interactive requires --stack <id> or --mcp <language>";
pub const MSG_LEGACY_FULLSTACK: &str = "Error: --fullstack no longer takes a frontend value (react|vue). It is a flag for MVC frameworks (ruby-rails-8, php-laravel). For backend + frontend, run `dare init` interactively and choose Monorepo.";

const BACKEND_STACK_IDS: &[&str] = &[
    "go-gin",
    "go-stdlib",
    "node-nestjs",
    "php-laravel",
    "python-fastapi",
    "ruby-rails-8",
    "rust-axum",
];

/// CLI options (clap → domain).
#[derive(Debug, Clone, Default)]
pub struct InitCliOpts {
    pub name: Option<String>,
    pub dir: Option<PathBuf>,
    pub stack: Option<String>,
    pub mcp: Option<String>,
    /// `Some("")` = bare `--fullstack`; `Some(v)` = legacy `--fullstack <v>`.
    pub fullstack: Option<String>,
    pub transport: Option<String>,
    pub toolchain: Option<String>,
    pub non_interactive: bool,
    pub force: bool,
    pub check: bool,
    pub json: bool,
    pub color: bool,
}

/// Map `--stack`; Rust keeps the `rails` alias.
pub fn resolve_stack_alias(input: &str) -> String {
    if input == "rails" {
        "ruby-rails-8".to_string()
    } else {
        input.to_string()
    }
}

/// `--mcp` language: npm values plus Rust 4.0 aliases (case-insensitive).
pub fn resolve_mcp_language(input: &str) -> CoreResult<McpLanguage> {
    match input.to_ascii_lowercase().as_str() {
        "node-ts" | "ts" | "node" | "typescript" | "mcp-node-ts" => Ok(McpLanguage::NodeTs),
        "python" | "py" | "mcp-python" => Ok(McpLanguage::Python),
        "rust" | "mcp-rust" => Ok(McpLanguage::Rust),
        "go" | "mcp-go" => Ok(McpLanguage::Go),
        _ => Err(CoreError::usage(format!(
            "Error: unknown mcp language '{input}'. Valid: go, node-ts, python, rust"
        ))),
    }
}

fn parse_toolchain(input: Option<&str>) -> CoreResult<ToolchainMode> {
    match input {
        None => Ok(ToolchainMode::Auto),
        Some(v) => ToolchainMode::parse(v).ok_or_else(|| {
            CoreError::usage(format!(
                "Error: invalid --toolchain '{v}'. Valid: auto, docker, native"
            ))
        }),
    }
}

fn parse_transport(input: Option<&str>) -> CoreResult<McpTransport> {
    match input.unwrap_or("stdio") {
        "stdio" => Ok(McpTransport::Stdio),
        "sse" => Ok(McpTransport::Sse),
        "http" | "http-stream" => Ok(McpTransport::HttpStream),
        other => Err(CoreError::usage(format!(
            "Error: invalid --transport '{other}'. Valid: http, sse, stdio"
        ))),
    }
}

fn is_non_interactive(opts: &InitCliOpts) -> bool {
    opts.non_interactive || opts.stack.is_some() || opts.mcp.is_some()
}

/// npm `runNonInteractive` → `ProjectConfig`.
pub fn resolve_non_interactive(opts: &InitCliOpts, parent: &Path) -> CoreResult<ProjectConfig> {
    if opts.fullstack.as_deref().is_some_and(|v| !v.is_empty()) {
        return Err(CoreError::usage(MSG_LEGACY_FULLSTACK));
    }
    let raw_name = opts.name.as_deref().unwrap_or("my-dare-project");
    let name = validate_npm_project_name(raw_name)?;
    let output_dir = resolve_project_output_dir(parent, &name)?;

    if opts.stack.is_none() && opts.mcp.is_none() {
        return Err(CoreError::usage(MSG_NEED_STACK_OR_MCP));
    }
    let backend = match opts.stack.as_deref().map(resolve_stack_alias) {
        Some(id) => Some(BackendStack::parse(&id).ok_or_else(|| {
            CoreError::usage(format!(
                "Error: unknown stack '{id}'. Valid: {}",
                BACKEND_STACK_IDS.join(", ")
            ))
        })?),
        None => None,
    };
    let fullstack = opts.fullstack.is_some();
    if fullstack && backend.is_some_and(|b| !b.is_mvc_capable()) {
        return Err(CoreError::usage(
            "Error: --fullstack only applies to MVC frameworks: php-laravel, ruby-rails-8",
        ));
    }
    let mcp_language = opts.mcp.as_deref().map(resolve_mcp_language).transpose()?;
    let toolchain = parse_toolchain(opts.toolchain.as_deref())?;

    let base = ProjectConfig {
        name,
        structure: Structure::Backend,
        backend: None,
        frontend: None,
        mcp_transport: None,
        mcp_language: None,
        mcp_features: None,
        ide: Ide::Cursor,
        graphrag: GraphragBackend::Sqlite,
        mcp: false,
        toolchain,
        rust_workspace_layout: None,
        crate_prefix: None,
        output_dir,
    };
    if let Some(lang) = mcp_language {
        return Ok(ProjectConfig {
            structure: Structure::McpServer,
            mcp_language: Some(lang),
            mcp_transport: Some(parse_transport(opts.transport.as_deref())?),
            ..base
        });
    }
    Ok(ProjectConfig {
        structure: if fullstack {
            Structure::Mvc
        } else {
            Structure::Backend
        },
        backend,
        ..base
    })
}

fn config_from_answers(a: InitAnswers, parent: &Path) -> CoreResult<ProjectConfig> {
    let name = validate_npm_project_name(&a.name)?;
    let output_dir = resolve_project_output_dir(parent, &name)?;
    Ok(ProjectConfig {
        name,
        structure: a.structure,
        backend: a.backend,
        frontend: a.frontend,
        mcp_transport: a.mcp_transport,
        mcp_language: a.mcp_language,
        mcp_features: a.mcp_features,
        ide: a.ide,
        graphrag: a.graphrag,
        mcp: a.mcp,
        toolchain: a.toolchain,
        rust_workspace_layout: a.rust_workspace_layout,
        crate_prefix: a.crate_prefix,
        output_dir,
    })
}

/// Interactive flow with an injectable prompter (tests drive it directly).
pub fn resolve_interactive(
    opts: &InitCliOpts,
    parent: &Path,
    prompter: &mut dyn Prompter,
) -> CoreResult<ProjectConfig> {
    if let Some(n) = opts.name.as_deref() {
        // npm validates the positional name after the prompts; fail early instead
        // of asking every question first.
        validate_npm_project_name(n)?;
    }
    let answers = prompt_init_answers(prompter, opts.name.as_deref())?;
    config_from_answers(answers, parent)
}

/// npm "Next steps" block (interactive path), with fixes B-06 / B-07.
pub fn next_steps(cfg: &ProjectConfig) -> String {
    let mut out = String::from("\n📋 Next steps:\n\n");
    out.push_str(&format!("  1. cd {}\n", cfg.name));
    if cfg.structure == Structure::McpServer {
        let (install, dev, inspect) = match cfg.mcp_language.unwrap_or(McpLanguage::NodeTs) {
            McpLanguage::Python => (
                "pip install -r requirements.txt",
                "python main.py",
                "npx @modelcontextprotocol/inspector python main.py",
            ),
            McpLanguage::NodeTs => ("npm install", "npm run dev", "npm run inspect"),
            // B-06: npm prints the Node commands for Rust and Go servers.
            McpLanguage::Rust => (
                "cargo build",
                "cargo run",
                "npx @modelcontextprotocol/inspector cargo run",
            ),
            McpLanguage::Go => (
                "go mod tidy",
                "go run ./cmd/server",
                "npx @modelcontextprotocol/inspector go run ./cmd/server",
            ),
        };
        out.push_str(&format!("  2. {install}\n"));
        out.push_str("  3. dare design \"Describe what this MCP server exposes\"\n");
        out.push_str("  4. dare blueprint\n");
        out.push_str("  5. dare execute --parallel\n");
        out.push_str(&format!("  6. {inspect}  ← test with MCP Inspector\n"));
        out.push_str(&format!("  7. {dev}\n\n"));
    } else {
        out.push_str("  2. dare design \"Describe your project here\"\n");
        out.push_str("  3. dare blueprint\n");
        out.push_str("  4. dare execute --parallel\n\n");
    }
    if matches!(cfg.ide, Ide::ClaudeCode | Ide::ClaudeHybrid) {
        out.push_str(
            "  Claude Code tip: use /dare-design, /dare-blueprint, /dare-execute as slash commands\n\n",
        );
    }
    let is_rust_monorepo = cfg.is_rust_monorepo();
    if is_rust_monorepo {
        out.push_str("🦀 Rust workspace created!\n");
        match cfg.layout() {
            RustWorkspaceLayout::Single => out.push_str(&format!(
                "   crates/server/  (rust-axum)  +  crates/web/  ({})\n",
                cfg.frontend.map_or("", |f| f.as_str())
            )),
            RustWorkspaceLayout::Multi => {
                // B-07: npm shows the name slug instead of the chosen prefix.
                let p = cfg
                    .crate_prefix
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .unwrap_or_else(|| npm_slug(&cfg.name));
                out.push_str(&format!(
                    "   crates/{p}-core  |  crates/{p}-server  |  crates/{p}-web  |  crates/{p}-cli\n"
                ));
            }
        }
        out.push_str(
            "   See .cargo/config.toml — do NOT add a global [build] target (breaks WASM + native crates).\n\n",
        );
        out.push_str(
            "   Tip: /dare-rust-leptos for Leptos idioms · /dare-rust-workspace for workspace decisions.\n\n",
        );
    }
    if cfg.frontend.is_some_and(FrontendStack::is_leptos) && !is_rust_monorepo {
        out.push_str(
            "  Leptos tip: use /dare-rust-leptos for component patterns, server functions and workspace config.\n\n",
        );
    }
    out
}

fn report(cfg: &ProjectConfig, created: &[PathBuf], check: bool) -> Value {
    let root = &cfg.output_dir;
    let mut rel: Vec<String> = created
        .iter()
        .filter_map(|p| p.strip_prefix(root).ok())
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect();
    rel.sort();
    json!({
        "schemaVersion": INIT_REPORT_SCHEMA,
        "mode": "init",
        "check": check,
        "projectRoot": root.to_string_lossy().replace('\\', "/"),
        "projectName": cfg.name,
        "structure": cfg.structure.as_str(),
        "backend": cfg.backend.map(|b| b.as_str()),
        "frontend": cfg.frontend.map(|f| f.as_str()),
        "mcpLanguage": cfg.mcp_language.map(|l| l.as_str()),
        "mcpTransport": cfg.mcp_transport.map(|t| t.as_str()),
        "ide": cfg.ide.as_str(),
        "graphrag": cfg.graphrag.as_str(),
        "mcp": cfg.mcp,
        "toolchain": cfg.toolchain.as_str(),
        "created": rel,
    })
}

/// Generate the project for a resolved config (shared by CLI and tests).
pub fn execute_init(
    cfg: &ProjectConfig,
    parent: &Path,
    force: bool,
    console: &mut dyn Console,
) -> CoreResult<Vec<PathBuf>> {
    let mut runner = SystemToolRunner;
    let mut env = GenerateEnv {
        cwd: parent,
        runner: &mut runner,
        console,
        rails_offline: std::env::var("DARE_RAILS_OFFLINE").as_deref() == Ok("1"),
        force,
    };
    generate_project_structure(cfg, &mut env).map(|r| r.created_files)
}

fn banner_allowed(opts: &InitCliOpts) -> bool {
    !opts.json
        && std::io::stdout().is_terminal()
        && std::env::var("DARE_NO_BANNER").as_deref() != Ok("1")
}

/// `dare init` entry point.
pub fn run_init_cmd(opts: InitCliOpts) -> CoreResult<(String, Value)> {
    let parent = match opts.dir.clone() {
        Some(d) => d,
        None => std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?,
    };
    // Live progress goes to stdout like npm; under --json it moves to stderr.
    let mut console = StdConsole::new(opts.color, opts.json);
    let non_interactive = is_non_interactive(&opts);

    let cfg = if non_interactive {
        resolve_non_interactive(&opts, &parent)?
    } else {
        if !std::io::stdin().is_terminal() {
            return Err(CoreError::invalid_input(MSG_INTERACTIVE_REQUIRES_TTY));
        }
        if banner_allowed(&opts) {
            if let Some(b) = crate::commands::welcome::render_banner(!opts.color) {
                print!("{b}");
            }
        }
        console.log(
            Tone::BlueBold,
            "\n🚀 DARE Framework - Project Initialization\n",
        );
        let mut prompter = DialoguerPrompter::default();
        resolve_interactive(&opts, &parent, &mut prompter)?
    };

    if opts.check {
        let data = report(&cfg, &[], true);
        return Ok((
            format!(
                "init --check: {} ({}) → {}",
                cfg.name,
                cfg.structure,
                cfg.output_dir.display()
            ),
            data,
        ));
    }

    console.log(Tone::Plain, &format!("- Creating project {}...", cfg.name));
    match execute_init(&cfg, &parent, opts.force, &mut console) {
        Ok(created) => {
            let human = if non_interactive {
                let stack = cfg.backend.map_or_else(
                    || format!("mcp-{}", cfg.mcp_language.map_or("node-ts", |l| l.as_str())),
                    |b| b.as_str().to_string(),
                );
                format!("✔ Project {} created.\n  Stack: {stack}", cfg.name)
            } else {
                format!(
                    "✔ Project {} created successfully!\n{}",
                    cfg.name,
                    next_steps(&cfg).trim_end_matches('\n')
                )
            };
            Ok((human, report(&cfg, &created, false)))
        }
        Err(err) => {
            console.log(Tone::Red, "✖ Failed to create project");
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::init_prompts::tests::ScriptedPrompter;
    use dare_core::ErrorKind;
    use dare_scaffold::project::console::MemoryConsole;

    fn opts() -> InitCliOpts {
        InitCliOpts {
            name: Some("demo-app".into()),
            ..InitCliOpts::default()
        }
    }

    #[test]
    fn non_interactive_backend_defaults_match_npm() {
        let mut o = opts();
        o.stack = Some("node-nestjs".into());
        let cfg = resolve_non_interactive(&o, Path::new("/w")).expect("cfg");
        assert_eq!(cfg.structure, Structure::Backend);
        assert_eq!(cfg.ide, Ide::Cursor);
        assert_eq!(cfg.graphrag, GraphragBackend::Sqlite);
        assert!(!cfg.mcp);
        assert_eq!(cfg.toolchain, ToolchainMode::Auto);
    }

    #[test]
    fn non_interactive_errors_match_npm() {
        let mut o = opts();
        o.non_interactive = true;
        assert_eq!(
            resolve_non_interactive(&o, Path::new("/w"))
                .unwrap_err()
                .message(),
            MSG_NEED_STACK_OR_MCP
        );
        o.stack = Some("django".into());
        assert_eq!(
            resolve_non_interactive(&o, Path::new("/w")).unwrap_err().message(),
            "Error: unknown stack 'django'. Valid: go-gin, go-stdlib, node-nestjs, php-laravel, python-fastapi, ruby-rails-8, rust-axum"
        );
        o.stack = Some("node-nestjs".into());
        o.fullstack = Some(String::new());
        assert_eq!(
            resolve_non_interactive(&o, Path::new("/w"))
                .unwrap_err()
                .message(),
            "Error: --fullstack only applies to MVC frameworks: php-laravel, ruby-rails-8"
        );
        o.fullstack = Some("vue".into());
        assert_eq!(
            resolve_non_interactive(&o, Path::new("/w"))
                .unwrap_err()
                .kind(),
            ErrorKind::Usage
        );
    }

    #[test]
    fn fullstack_and_mcp_paths() {
        let mut o = opts();
        o.stack = Some("rails".into());
        o.fullstack = Some(String::new());
        let cfg = resolve_non_interactive(&o, Path::new("/w")).expect("mvc");
        assert_eq!(cfg.structure, Structure::Mvc);
        assert_eq!(cfg.backend, Some(BackendStack::RubyRails8));

        let mut o = opts();
        o.mcp = Some("go".into());
        o.transport = Some("http".into());
        let cfg = resolve_non_interactive(&o, Path::new("/w")).expect("mcp");
        assert_eq!(cfg.structure, Structure::McpServer);
        assert_eq!(cfg.mcp_transport, Some(McpTransport::HttpStream));
        assert_eq!(cfg.mcp_features, None);
    }

    #[test]
    fn reported_case_monorepo_nestjs_vue_generates_backend_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let o = InitCliOpts {
            name: Some("api-nest-validations".into()),
            ..InitCliOpts::default()
        };
        let mut p = ScriptedPrompter::new(&[
            "Monorepo",
            "NestJS",
            "Vue",
            "Claude Code",
            "SQLite",
            "y",
            "Auto",
        ]);
        let cfg = resolve_interactive(&o, tmp.path(), &mut p).expect("cfg");
        // No npm on the test PATH is not guaranteed; use docker-less native with a
        // missing tool to prove the layout up to the frontend step instead.
        assert_eq!(cfg.output_dir, tmp.path().join("api-nest-validations"));
        let steps = next_steps(&cfg);
        assert!(steps.contains("Claude Code tip"));
        let mut console = MemoryConsole::default();
        let mut c2 = cfg.clone();
        c2.frontend = Some(FrontendStack::None);
        execute_init(&c2, tmp.path(), false, &mut console).expect("generate");
        let root = tmp.path().join("api-nest-validations");
        assert!(root.join("backend/src/app.module.ts").is_file());
        assert!(!root.join("src").exists());
        assert!(!root.join("frontend").exists());
    }

    #[test]
    fn next_steps_mcp_rust_uses_cargo() {
        let mut o = opts();
        o.mcp = Some("rust".into());
        let cfg = resolve_non_interactive(&o, Path::new("/w")).expect("cfg");
        let s = next_steps(&cfg);
        assert!(s.contains("  2. cargo build\n"));
        assert!(!s.contains("npm install"));
    }
}
