//! DARE Framework CLI (native Rust rewrite).

mod commands;
mod output;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use commands::blueprint::{run_blueprint, BlueprintInput};
use commands::dag::{run_dag_viz, CliVizFormat};
use commands::design::run_design;
use commands::discover::run_discover;
use commands::dna::run_dna_cmd;
use commands::execute::{run_execute, ExecuteAction};
use commands::guard::{run_guard_cmd, GuardCliOpts};
use commands::info::{collect_info, format_human, report_to_json};
use commands::reverse::{run_reverse, ReverseCliOpts};
use commands::review::{run_review_cmd, ReviewCliArgs};
use commands::skill::{run_skill, SkillAction};
use commands::update::run_update;
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
    /// Brownfield reverse engineering → IDEIA.md + REVERSE specs.
    Reverse {
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
        /// Analyze only — do not write DARE artifacts.
        #[arg(long)]
        check: bool,
        /// Generate deep Fase-3 stubs (erd, c4, …).
        #[arg(long)]
        deep: bool,
        /// Comma-separated module ids to include.
        #[arg(long)]
        modules: Option<String>,
        /// Optional AST pass via dare-ast (merge endpoints/entities).
        #[arg(long)]
        ast: bool,
        /// Skip Excalidraw module map (default: write).
        #[arg(long)]
        no_excalidraw: bool,
        /// Write confidence-report.md.
        #[arg(long)]
        report: bool,
        /// Optional AI enrichment of IDEIA.md (soft-fail).
        #[arg(long)]
        ai: bool,
        /// AI provider id (requires `--ai`; default: codex).
        #[arg(long)]
        provider: Option<String>,
    },
    /// Extract project DNA conventions into DARE/PROJECT-DNA.md.
    Dna {
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
        /// Collect facts only — do not write PROJECT-DNA.md / dna-facts.json.
        #[arg(long)]
        check: bool,
        /// Enable optional AST sampling via dare-ast.
        #[arg(long)]
        ast: bool,
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
    /// Security gate: unicode, injection scan, provenance (exit 6 on FAIL).
    Guard {
        /// File or directory target (default: DARE/ + dare.config.json).
        target: Option<PathBuf>,
        /// Scan git staged files only.
        #[arg(long, conflicts_with_all = ["all", "sign"])]
        staged: bool,
        /// Scan all text-ish files under the project (skips heavy dirs).
        #[arg(long, conflicts_with_all = ["staged", "sign"])]
        all: bool,
        /// Sign a target file (writes `.minisig`); requires DARE_GUARD_PRIVATE_KEY.
        #[arg(long, conflicts_with_all = ["staged", "all"])]
        sign: bool,
        /// Unicode mode: strip|block (default: block).
        #[arg(long, default_value = "block")]
        unicode: String,
        /// Treat WARN as FAIL (exit 6).
        #[arg(long)]
        strict: bool,
        /// Fail when severity reaches this level: fail|warn (default: fail).
        #[arg(long, default_value = "fail")]
        fail_on: String,
        /// Output format hint: text|json (JSON envelope still via global --json).
        #[arg(long, default_value = "text")]
        format: String,
        /// Optional comment for --sign (reserved).
        #[arg(long)]
        comment: Option<String>,
    },
    /// DAG utilities (visualization).
    Dag {
        #[command(subcommand)]
        action: DagCmd,
    },
    /// Orchestrate DAG execution (status / next / watch / complete / fail / reset / agent).
    Execute {
        /// Show DAG runtime status (default when no action flag is set).
        #[arg(long, conflicts_with_all = ["next", "watch", "complete", "fail", "reset", "agent", "cleanup_worktrees"])]
        status: bool,
        /// Print next executable tasks at the minimum ready rank.
        #[arg(long, conflicts_with_all = ["status", "watch", "complete", "fail", "reset", "agent", "cleanup_worktrees"])]
        next: bool,
        /// Watch status without mutating runtime state.
        #[arg(long, conflicts_with_all = ["status", "next", "complete", "fail", "reset", "agent", "cleanup_worktrees"])]
        watch: bool,
        /// Mark task DONE after Ralph gates pass.
        #[arg(long, value_name = "TASK_ID", conflicts_with_all = ["status", "next", "watch", "fail", "reset", "agent", "cleanup_worktrees"])]
        complete: Option<String>,
        /// Mark task FAILED (cascade skip dependents).
        #[arg(long, value_name = "TASK_ID", conflicts_with_all = ["status", "next", "watch", "complete", "reset", "agent", "cleanup_worktrees"])]
        fail: Option<String>,
        /// Reset task to PENDING (preserves attempts).
        #[arg(long, value_name = "TASK_ID", conflicts_with_all = ["status", "next", "watch", "complete", "fail", "agent", "cleanup_worktrees"])]
        reset: Option<String>,
        /// Run agent loop (mock/noop drivers in microplano 030).
        #[arg(long, conflicts_with_all = ["status", "next", "watch", "complete", "fail", "reset", "cleanup_worktrees"])]
        agent: bool,
        /// Remove orphan dirs under `.dare/agent-worktrees/`.
        #[arg(long, conflicts_with_all = ["status", "next", "watch", "complete", "fail", "reset", "agent"])]
        cleanup_worktrees: bool,
        /// Agent driver id (default: mock). Requires `--agent`.
        #[arg(long, requires = "agent")]
        driver: Option<String>,
        /// Task id for `--agent` (default: first ready at min rank).
        #[arg(long, requires = "agent")]
        task: Option<String>,
        /// Token budget for `--agent` (`0` = unlimited).
        #[arg(long, requires = "agent", default_value_t = 0)]
        budget_tokens: u64,
        /// Agent policy (only `fixed` in 030).
        #[arg(long, requires = "agent", default_value = "fixed")]
        policy: String,
        /// Completion summary (requires `--complete`; default: Task completed.).
        #[arg(long, requires = "complete")]
        output: Option<String>,
        /// Failure reason (requires `--fail`; default: Task failed.).
        #[arg(long, requires = "fail")]
        reason: Option<String>,
        /// Path to dare-dag.yaml (default: DARE/dare-dag.yaml).
        #[arg(long)]
        dag: Option<PathBuf>,
        /// Watch poll interval in seconds (default: 2).
        #[arg(long, default_value_t = 2)]
        interval: u64,
        /// Stop watch after N ticks (omit for long-running watch).
        #[arg(long)]
        max_ticks: Option<u64>,
    },
    /// Render DARE/DESIGN.md from a feature description (deterministic).
    Design {
        /// Feature/product description (omit with --interactive).
        description: Vec<String>,
        /// Prompt for title and description on a TTY.
        #[arg(long)]
        interactive: bool,
        /// Run optional AI enrichment after deterministic write.
        #[arg(long)]
        ai: bool,
        /// AI provider id (requires `--ai`; default: codex).
        #[arg(long)]
        provider: Option<String>,
    },
    /// Generate DARE/BLUEPRINT.md, TASKS.md, dare-dag.yaml and EXECUTION specs from Design.
    Blueprint {
        /// Optional path to DESIGN.md (default DARE/DESIGN.md).
        design: Option<PathBuf>,
        /// Overwrite existing artifacts even without managed marker.
        #[arg(long)]
        force: bool,
        /// Run optional AI enrichment on BLUEPRINT.md (soft-fail).
        #[arg(long)]
        ai: bool,
        /// AI provider id (requires `--ai`; default: codex).
        #[arg(long)]
        provider: Option<String>,
    },
    /// Static anti-stub / mock / TODO review for a task.
    Review {
        /// Task id (matches DARE/EXECUTION/<id>.md).
        task_id: String,
        /// Treat warnings as failures.
        #[arg(long)]
        strict: bool,
        /// Emit only error-severity findings.
        #[arg(long)]
        errors_only: bool,
        /// Override files to scan (project-relative).
        #[arg(long = "files", value_name = "PATH", num_args = 1..)]
        files: Vec<PathBuf>,
        /// Merge semantic JSON from agent (`passed` / `unmetCriteria`).
        #[arg(long = "from-agent", value_name = "PATH")]
        from_agent: Option<PathBuf>,
        /// Output format: human | json | github.
        #[arg(long = "format", default_value = "human")]
        format: String,
        /// Include markdown PR comment body.
        #[arg(long)]
        comment: bool,
        /// Fail threshold: error | warning | never.
        #[arg(long = "fail-on", default_value = "error")]
        fail_on: String,
        /// Optional AI enrichment (soft stub Class B — static always runs).
        #[arg(long)]
        ai: bool,
        /// AI provider id (requires `--ai`).
        #[arg(long)]
        provider: Option<String>,
    },
    /// Plan (`--dry-run`) or apply project asset updates.
    Update {
        /// Plan only; no writes (`--force` ignored).
        #[arg(long)]
        dry_run: bool,
        /// Keep customized files without prompting (non-interactive keep).
        #[arg(short = 'y', long = "yes")]
        yes: bool,
        /// Overwrite customized files (session backup first).
        #[arg(long)]
        force: bool,
        /// Limit plan to harness: claude-code|cursor|codex|antigravity|hybrid|claude-hybrid
        #[arg(long)]
        target: Option<String>,
        /// Project directory (default: cwd walk).
        #[arg(short = 'd', long = "dir")]
        dir: Option<PathBuf>,
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
    /// Skills-pacote registry (microplano 044).
    Skill {
        #[command(subcommand)]
        action: SkillCmd,
    },
}

#[derive(Debug, Subcommand)]
enum SkillCmd {
    /// List skills from remote > local > mock registries.
    List,
    /// Show details for one skill (merged registries).
    Info {
        /// Skill package name.
        name: String,
    },
}

#[derive(Debug, Subcommand)]
enum DagCmd {
    /// Render Mermaid / DOT / Excalidraw visualization of a DAG.
    Viz {
        /// Path to dare-dag.yaml (default: DARE/dare-dag.yaml).
        #[arg(long)]
        dag: Option<PathBuf>,
        /// Output format (exact lowercase).
        #[arg(short = 'f', long = "format", value_enum, default_value_t = CliVizFormat::Mermaid)]
        format: CliVizFormat,
        /// Write visualization to this path (project-relative or under root).
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
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
            Some(Commands::Review {
                task_id,
                strict,
                errors_only,
                files,
                from_agent,
                format,
                comment,
                fail_on,
                ai,
                provider,
            }) => run_review_cmd(
                ReviewCliArgs {
                    task_id,
                    strict,
                    errors_only,
                    files,
                    from_agent,
                    format,
                    comment,
                    fail_on,
                    ai,
                    provider,
                },
                &renderer,
            ),
            Some(Commands::Guard {
                target,
                staged,
                all,
                sign,
                unicode,
                strict,
                fail_on,
                format,
                comment,
            }) => run_guard_cmd(
                GuardCliOpts {
                    target,
                    staged,
                    all,
                    sign,
                    unicode,
                    strict,
                    fail_on,
                    format,
                    comment,
                },
                &renderer,
            ),
            Some(Commands::Dag {
                action:
                    DagCmd::Viz {
                        dag,
                        format,
                        output,
                    },
            }) => run_dag_viz(dag, format, output, &renderer),
            Some(Commands::Execute {
                status,
                next,
                watch,
                complete,
                fail,
                reset,
                agent,
                cleanup_worktrees,
                driver,
                task,
                budget_tokens,
                policy,
                output,
                reason,
                dag,
                interval,
                max_ticks,
            }) => {
                let action = if cleanup_worktrees {
                    ExecuteAction::CleanupWorktrees
                } else if agent {
                    ExecuteAction::Agent {
                        driver: driver.unwrap_or_else(|| "mock".to_string()),
                        task,
                        budget_tokens,
                        policy,
                    }
                } else if let Some(id) = complete {
                    ExecuteAction::Complete { id, output }
                } else if let Some(id) = fail {
                    ExecuteAction::Fail { id, reason }
                } else if let Some(id) = reset {
                    ExecuteAction::Reset { id }
                } else if watch {
                    ExecuteAction::Watch {
                        interval_secs: interval,
                        max_ticks,
                    }
                } else if next {
                    ExecuteAction::Next
                } else {
                    let _ = status; // default or explicit --status
                    ExecuteAction::Status
                };
                run_execute(dag, action, &renderer)
            }
            Some(Commands::Skill { action }) => {
                let skill_action = match action {
                    SkillCmd::List => SkillAction::List,
                    SkillCmd::Info { name } => SkillAction::Info { name },
                };
                run_skill(skill_action, &renderer)
            }
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
                force_color: None,
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
        Some(Commands::Design {
            description,
            interactive,
            ai,
            provider,
        }) => {
            let desc = if description.is_empty() {
                None
            } else {
                Some(description.join(" "))
            };
            run_design(desc, interactive, ai, provider)
        }
        Some(Commands::Blueprint {
            design,
            force,
            ai,
            provider,
        }) => run_blueprint(BlueprintInput {
            design_rel_or_abs: design,
            force,
            ai,
            provider,
        }),
        Some(Commands::Discover {
            dir,
            check,
            force,
            dry_run,
            strict_conflicts,
        }) => run_discover(dir, check, force, dry_run, strict_conflicts),
        Some(Commands::Reverse {
            dir,
            check,
            deep,
            modules,
            ast,
            no_excalidraw,
            report,
            ai,
            provider,
        }) => run_reverse(ReverseCliOpts {
            dir,
            check,
            deep,
            modules,
            ast,
            no_excalidraw,
            report,
            ai,
            provider,
        }),
        Some(Commands::Dna { dir, check, ast }) => run_dna_cmd(dir, check, ast),
        Some(Commands::Update {
            dry_run,
            yes,
            force,
            target,
            dir,
        }) => run_update(dry_run, yes, force, target, dir),
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
        Some(Commands::Review { .. }) => unreachable!("review handled in main"),
        Some(Commands::Guard { .. }) => unreachable!("guard handled in main"),
        Some(Commands::Dag { .. }) => unreachable!("dag handled in main"),
        Some(Commands::Execute { .. }) => unreachable!("execute handled in main"),
        Some(Commands::Skill { .. }) => unreachable!("skill handled in main"),
    }
}

fn ok_msg(msg: String) -> Result<(String, serde_json::Value), CoreError> {
    Ok((msg.clone(), serde_json::json!({ "message": msg })))
}

fn exit(code: i32) -> ExitCode {
    ExitCode::from(code as u8)
}
