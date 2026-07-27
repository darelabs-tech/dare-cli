//! DARE Framework CLI (native Rust rewrite).

mod commands;
mod output;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use commands::bench::{run_bench_cmd, BenchCliOpts};
use commands::blueprint::{run_blueprint, BlueprintInput};
use commands::bootstrap::{run_bootstrap_cmd, BootstrapCliOpts};
use commands::dag::{run_dag_viz, CliVizFormat};
use commands::design::run_design;
use commands::discover::run_discover;
use commands::dna::run_dna_cmd;
use commands::execute::{run_execute, ExecuteAction};
use commands::graph::{run_graph, GraphAction};
use commands::guard::{run_guard_cmd, GuardCliOpts};
use commands::info::{collect_info, format_human, report_to_json};
use commands::init::{run_init_cmd, InitCliOpts};
use commands::migrate::{run_migrate_cmd, MigrateCliOpts};
use commands::patterns::run_patterns_cmd;
use commands::refine::{run_refine_cmd, RefineCliArgs};
use commands::reverse::{run_reverse, ReverseCliOpts};
use commands::review::{run_review_cmd, ReviewCliArgs};
use commands::skill::{run_skill, SkillAction};
use commands::hooks::{run_hooks_cmd, run_hooks_list, run_hooks_validate};
use commands::steering::{run_steering_list, run_steering_show};
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
    /// Greenfield init: scaffold + config + harnesses (microplano 047).
    Init {
        /// Project name (directory name under parent).
        name: Option<String>,
        /// Backend stack id (alias: rails → ruby-rails-8).
        #[arg(long)]
        stack: Option<String>,
        /// MCP language alias (mutually exclusive with --stack).
        #[arg(long)]
        mcp: Option<String>,
        /// Fullstack frontend companion (react|vue; requires --stack).
        #[arg(long)]
        fullstack: Option<String>,
        /// MCP transport (stdio|http|sse; MCP stacks only).
        #[arg(long)]
        transport: Option<String>,
        /// Toolchain overlay (none|docker).
        #[arg(long)]
        toolchain: Option<String>,
        /// Non-interactive mode (requires name + --stack or --mcp).
        #[arg(long)]
        non_interactive: bool,
        /// Overwrite when target exists.
        #[arg(long)]
        force: bool,
        /// Plan only — zero writes.
        #[arg(long)]
        check: bool,
        /// Parent directory (default: cwd); target = {dir}/{name}.
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
    },
    /// Re-apply scaffold on existing greenfield project (microplano 047).
    Bootstrap {
        /// Overwrite existing scaffold files (Replace policy).
        #[arg(long)]
        force: bool,
        /// Toolchain overlay (none|docker).
        #[arg(long)]
        toolchain: Option<String>,
        /// Plan only — zero writes.
        #[arg(long)]
        check: bool,
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
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
    /// Mine recurring code patterns into DARE/PATTERNS.md.
    Patterns {
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
        /// Mine only — do not write PATTERNS.md / patterns-facts.json.
        #[arg(long)]
        check: bool,
        /// Comma-separated module ids to include.
        #[arg(long)]
        modules: Option<String>,
        /// Preserve existing AGENT section bodies when rewriting PATTERNS.md.
        #[arg(long)]
        inject: bool,
        /// Enable optional AST sampling via dare-ast (call-idiom).
        #[arg(long)]
        ast: bool,
    },
    /// Brownfield migration plan → MIGRATION.md + parity skeletons.
    Migrate {
        /// Target stack id (required; closed allowlist).
        #[arg(long)]
        to: String,
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
        /// Plan only — do not write DARE/MIGRATION artifacts.
        #[arg(long)]
        check: bool,
        /// Optional AI enrichment of MIGRATION.md (soft-fail).
        #[arg(long)]
        ai: bool,
        /// AI provider id (requires `--ai`; default: codex).
        #[arg(long)]
        provider: Option<String>,
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
    /// Measure task complexity and optionally splice a sub-DAG.
    Refine {
        /// Task id from DARE/dare-dag.yaml.
        task_id: String,
        /// Include a split proposal even when not applying.
        #[arg(long)]
        split: bool,
        /// Apply splice to dare-dag.yaml and .dare/state.json.
        #[arg(long)]
        apply: bool,
        /// Exit 2 when level is HIGH or CRITICAL.
        #[arg(long)]
        strict: bool,
        /// Output format: human | json.
        #[arg(long = "format", default_value = "human")]
        format: String,
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
    /// Skills-pacote registry and lifecycle (microplanos 044–045).
    Skill {
        #[command(subcommand)]
        action: SkillCmd,
    },
    /// GraphRAG ingest / query / stats / viz / advanced (microplanos 041–043).
    Graph {
        #[command(subcommand)]
        action: GraphCmd,
    },
    /// Deterministic hooks list / run / validate (microplano 048).
    Hooks {
        #[command(subcommand)]
        action: HooksCmd,
    },
    /// Steering file discovery and resolution (microplano 048).
    Steering {
        #[command(subcommand)]
        action: SteeringCmd,
    },
    /// Deterministic Fix·Rate bench harness (microplano 049).
    Bench {
        /// Suite directory (default: fixtures/bench, relative to -d/cwd).
        #[arg(long)]
        suite: Option<PathBuf>,
        /// Baseline JSON for regression comparison.
        #[arg(long)]
        baseline: Option<PathBuf>,
        /// Fail when solve-rate drop exceeds N percentage points (0..=100).
        #[arg(long = "fail-on-regression", value_name = "N")]
        fail_on_regression: Option<u32>,
        /// Glob filter on case id.
        #[arg(long)]
        filter: Option<String>,
        /// Project directory (default: cwd).
        #[arg(short = 'd', long = "dir")]
        dir: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum HooksCmd {
    /// List hooks defs (embedded default or `.dare/hooks.yml` overlay).
    List {
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
    },
    /// Run hooks for a lifecycle event (trust gate + idempotent spawn).
    Run {
        /// Event id (`on-save`, `on-file-create`, `on-task-complete`, `pre-commit`).
        event: String,
        /// Optional project-relative file path for the event.
        #[arg(long)]
        file: Option<String>,
        /// Optional task id for the event.
        #[arg(long)]
        task: Option<String>,
        /// Bypass hooks.trusted=false for this invocation.
        #[arg(long)]
        trust: bool,
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
    },
    /// Validate hooks defs without executing actions.
    Validate {
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum SteeringCmd {
    /// List PROJECT-DNA, PATTERNS, and `.dare/steering/*.md` in precedence order.
    List {
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
    },
    /// Show steering blocks applicable to a project-relative file path.
    Show {
        /// Target file (project-relative).
        file: String,
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
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
    /// Install a skill into packages/skills (atomic).
    Add {
        /// Skill package name.
        name: String,
        /// Optional version pin.
        #[arg(long)]
        version: Option<String>,
        /// Install from a local archive (.tar/.tar.gz/.zip).
        #[arg(long)]
        from: Option<PathBuf>,
    },
    /// Remove an installed skill (blocked if reverse dependents exist).
    Remove {
        /// Skill package name.
        name: String,
    },
    /// Re-copy skill content and refresh the project manifest.
    Update {
        /// Skill package name.
        name: String,
        /// Optional archive source.
        #[arg(long)]
        from: Option<PathBuf>,
    },
    /// Pack an installed skill as tar.gz + sha256 (+ signature if keyed).
    Publish {
        /// Skill package name.
        name: String,
        /// Output directory (default: ./dist).
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum GraphCmd {
    /// Index project sources into the knowledge graph (contentHash + regex symbols).
    Ingest {
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
    },
    /// Hybrid keyword + BFS (+ optional semantic) query fused by RRF.
    Query {
        /// Search query string.
        query: String,
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
        /// Max hits to return.
        #[arg(long, default_value_t = 20)]
        limit: usize,
        /// BFS hop limit (default 2, max 5).
        #[arg(long, default_value_t = 2)]
        max_hops: usize,
        /// Max neighbors per node during BFS (default 50, max 200).
        #[arg(long, default_value_t = 50)]
        fanout: usize,
        /// Force keyword+BFS only (skip semantic channel even if compiled).
        #[arg(long = "no-semantic")]
        no_semantic: bool,
    },
    /// Show graph node/edge statistics.
    Stats {
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
    },
    /// Render a Mermaid subset of the graph.
    Viz {
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
        /// Write Mermaid to this path (project-relative or under root).
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
        /// Max nodes to include.
        #[arg(long, default_value_t = 80)]
        max_nodes: usize,
    },
    /// Report semantic feature / model cache status.
    Doctor {
        /// Project directory (default: cwd; unused by doctor report).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
    },
    /// Download / confirm the optional MiniLM embedding model (feature `semantic`).
    Enable {
        /// Project directory (default: cwd; unused by enable).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
        /// Skip interactive confirm / allow non-TTY download.
        #[arg(long)]
        yes: bool,
    },
    /// Keyword locate with hop decay (default decay 0.7).
    Locate {
        /// Search query string.
        query: String,
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
        /// Max hits to return.
        #[arg(long, default_value_t = 20)]
        limit: usize,
        /// BFS hop limit (default 2, max 5).
        #[arg(long, default_value_t = 2)]
        max_hops: usize,
        /// Max neighbors per node during BFS (default 50, max 200).
        #[arg(long, default_value_t = 50)]
        fanout: usize,
    },
    /// Resolve owners of a seed node (metadata owner + incoming contains).
    Owners {
        /// Seed node id.
        seed: String,
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
    },
    /// Blast-radius impact from one or more seeds (comma-separated).
    Impact {
        /// Seed node id(s), comma-separated.
        seeds: String,
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
        /// Max impacted ids to return.
        #[arg(long, default_value_t = 20)]
        limit: usize,
        /// BFS hop limit (default 2, max 5).
        #[arg(long, default_value_t = 2)]
        max_hops: usize,
        /// Max neighbors per node during BFS (default 50, max 200).
        #[arg(long, default_value_t = 50)]
        fanout: usize,
    },
    /// Shortest paths between two nodes.
    Trace {
        /// Source node id.
        #[arg(long)]
        from: String,
        /// Target node id.
        #[arg(long)]
        to: String,
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
        /// BFS hop limit (default 2, max 5).
        #[arg(long, default_value_t = 2)]
        max_hops: usize,
        /// Max neighbors per node during BFS (default 50, max 200).
        #[arg(long, default_value_t = 50)]
        fanout: usize,
        /// Max paths to return.
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Classify orphan requirements/code and stale nodes.
    Drift {
        /// Project directory (default: cwd).
        #[arg(long, short = 'd')]
        dir: Option<PathBuf>,
        /// Exit 7 when violations meet or exceed --threshold.
        #[arg(long)]
        strict: bool,
        /// Violation count threshold (default 1).
        #[arg(long, default_value_t = 1)]
        threshold: u32,
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
            Some(Commands::Refine {
                task_id,
                split,
                apply,
                strict,
                format,
            }) => run_refine_cmd(
                RefineCliArgs {
                    task_id,
                    split,
                    apply,
                    strict,
                    format,
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
                    SkillCmd::Add {
                        name,
                        version,
                        from,
                    } => SkillAction::Add {
                        name,
                        version,
                        from,
                    },
                    SkillCmd::Remove { name } => SkillAction::Remove { name },
                    SkillCmd::Update { name, from } => SkillAction::Update { name, from },
                    SkillCmd::Publish { name, out } => SkillAction::Publish { name, out },
                };
                run_skill(skill_action, &renderer)
            }
            Some(Commands::Graph { action }) => {
                let graph_action = match action {
                    GraphCmd::Ingest { dir } => GraphAction::Ingest { dir },
                    GraphCmd::Query {
                        query,
                        dir,
                        limit,
                        max_hops,
                        fanout,
                        no_semantic,
                    } => GraphAction::Query {
                        dir,
                        query,
                        limit,
                        max_hops,
                        fanout,
                        no_semantic,
                    },
                    GraphCmd::Stats { dir } => GraphAction::Stats { dir },
                    GraphCmd::Viz {
                        dir,
                        output,
                        max_nodes,
                    } => GraphAction::Viz {
                        dir,
                        output,
                        max_nodes,
                    },
                    GraphCmd::Doctor { dir } => GraphAction::Doctor { dir },
                    GraphCmd::Enable { dir, yes } => GraphAction::Enable { dir, yes },
                    GraphCmd::Locate {
                        query,
                        dir,
                        limit,
                        max_hops,
                        fanout,
                    } => GraphAction::Locate {
                        dir,
                        query,
                        limit,
                        max_hops,
                        fanout,
                    },
                    GraphCmd::Owners { seed, dir } => GraphAction::Owners { dir, seed },
                    GraphCmd::Impact {
                        seeds,
                        dir,
                        limit,
                        max_hops,
                        fanout,
                    } => GraphAction::Impact {
                        dir,
                        seeds,
                        limit,
                        max_hops,
                        fanout,
                    },
                    GraphCmd::Trace {
                        from,
                        to,
                        dir,
                        max_hops,
                        fanout,
                        limit,
                    } => GraphAction::Trace {
                        dir,
                        from,
                        to,
                        max_hops,
                        fanout,
                        limit,
                    },
                    GraphCmd::Drift {
                        dir,
                        strict,
                        threshold,
                    } => GraphAction::Drift {
                        dir,
                        strict,
                        threshold,
                    },
                };
                run_graph(graph_action, &renderer)
            }
            Some(Commands::Bench {
                suite,
                baseline,
                fail_on_regression,
                filter,
                dir,
            }) => run_bench_cmd(
                BenchCliOpts {
                    suite,
                    baseline,
                    fail_on_regression,
                    filter,
                    dir,
                },
                &renderer,
            ),
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
        Some(Commands::Init {
            name,
            stack,
            mcp,
            fullstack,
            transport,
            toolchain,
            non_interactive,
            force,
            check,
            dir,
        }) => run_init_cmd(InitCliOpts {
            name,
            dir,
            stack,
            mcp,
            fullstack,
            transport,
            toolchain,
            non_interactive,
            force,
            check,
        }),
        Some(Commands::Bootstrap {
            force,
            toolchain,
            check,
            dir,
        }) => run_bootstrap_cmd(BootstrapCliOpts {
            dir,
            toolchain,
            force,
            check,
        }),
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
        Some(Commands::Patterns {
            dir,
            check,
            modules,
            inject,
            ast,
        }) => run_patterns_cmd(dir, check, inject, ast, modules),
        Some(Commands::Hooks { action }) => match action {
            HooksCmd::List { dir } => run_hooks_list(dir),
            HooksCmd::Run {
                event,
                file,
                task,
                trust,
                dir,
            } => run_hooks_cmd(event, file, task, trust, dir),
            HooksCmd::Validate { dir } => run_hooks_validate(dir),
        },
        Some(Commands::Steering { action }) => match action {
            SteeringCmd::List { dir } => run_steering_list(dir),
            SteeringCmd::Show { file, dir } => run_steering_show(file, dir),
        },
        Some(Commands::Migrate {
            to,
            dir,
            check,
            ai,
            provider,
        }) => run_migrate_cmd(MigrateCliOpts {
            to,
            dir,
            check,
            ai,
            provider,
        }),
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
        Some(Commands::Refine { .. }) => unreachable!("refine handled in main"),
        Some(Commands::Guard { .. }) => unreachable!("guard handled in main"),
        Some(Commands::Dag { .. }) => unreachable!("dag handled in main"),
        Some(Commands::Execute { .. }) => unreachable!("execute handled in main"),
        Some(Commands::Skill { .. }) => unreachable!("skill handled in main"),
        Some(Commands::Graph { .. }) => unreachable!("graph handled in main"),
        Some(Commands::Bench { .. }) => unreachable!("bench handled in main"),
    }
}

fn ok_msg(msg: String) -> Result<(String, serde_json::Value), CoreError> {
    Ok((msg.clone(), serde_json::json!({ "message": msg })))
}

fn exit(code: i32) -> ExitCode {
    ExitCode::from(code as u8)
}
