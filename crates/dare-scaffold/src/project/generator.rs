//! `generateProjectStructure` — port of npm `utils/project-generator.ts`.

use std::path::{Path, PathBuf};

use dare_core::{CoreError, CoreResult};
use serde::Serialize;
use serde_json::{json, Value};

use super::bootstrap::{bootstrap_backend, bootstrap_frontend, bootstrap_mcp, BootstrapCtx};
use super::config::{
    assert_within_cwd, npm_slug, FrontendStack, GraphragBackend, McpLanguage, ProjectConfig,
    RustWorkspaceLayout, Structure,
};
use super::console::{Console, Tone};
use super::ide::{install_ide_files, write_dare_templates};
use super::tools::ToolRunner;
use super::writer::ProjectWriter;

/// Version written to `dare.config.json` (`getFrameworkVersion`).
pub const FRAMEWORK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Runtime knobs that are not part of the npm `ProjectConfig`.
pub struct GenerateEnv<'a> {
    pub cwd: &'a Path,
    pub runner: &'a mut dyn ToolRunner,
    pub console: &'a mut dyn Console,
    /// `DARE_RAILS_OFFLINE=1`.
    pub rails_offline: bool,
    /// Rust-only `--force`: skip the empty-directory guard.
    pub force: bool,
}

/// Paths created by a successful generation (for reports).
#[derive(Debug, Clone, Default)]
pub struct GenerateReport {
    pub created_files: Vec<PathBuf>,
}

/// Port of `generateProjectStructure`. On failure, removes what it created
/// (fix B-09) and returns the error.
pub fn generate_project_structure(
    cfg: &ProjectConfig,
    env: &mut GenerateEnv<'_>,
) -> CoreResult<GenerateReport> {
    let mut writer = ProjectWriter::new();
    match generate_inner(cfg, env, &mut writer) {
        Ok(()) => Ok(GenerateReport {
            created_files: writer.created_files().to_vec(),
        }),
        Err(err) => {
            writer.rollback();
            Err(err)
        }
    }
}

fn generate_inner(
    cfg: &ProjectConfig,
    env: &mut GenerateEnv<'_>,
    writer: &mut ProjectWriter,
) -> CoreResult<()> {
    let out = &cfg.output_dir;
    assert_within_cwd(env.cwd, out)?;
    writer.ensure_dir(out)?;

    if !env.force {
        assert_output_dir_is_empty(cfg)?;
    }
    run_stack_bootstrap(cfg, env, writer)?;

    writer.ensure_dir(&out.join("DARE"))?;
    writer.ensure_dir(&out.join("DARE/EXECUTION"))?;
    writer.write(&out.join("dare.config.json"), dare_config_json(cfg)?)?;
    merge_gitignore(cfg, writer)?;
    install_ide_files(writer, out, cfg)?;
    writer.write(
        &out.join("DARE/README.md"),
        generate_shared_config(&cfg.name),
    )?;
    write_dare_templates(writer, out)?;
    write_graphrag_config(cfg, writer)?;
    Ok(())
}

// ─── Empty-dir guard ─────────────────────────────────────────────────────────

/// `assertOutputDirIsEmpty` (ignores `.git` and `.gitkeep`).
fn assert_output_dir_is_empty(cfg: &ProjectConfig) -> CoreResult<()> {
    let out = &cfg.output_dir;
    let dirs: Vec<PathBuf> = match cfg.structure {
        Structure::McpServer | Structure::Frontend | Structure::Backend | Structure::Mvc => {
            vec![out.clone()]
        }
        Structure::Monorepo => {
            let mut v = Vec::new();
            if cfg.backend.is_some() {
                v.push(out.join("backend"));
            }
            if cfg.frontend.is_some() {
                v.push(out.join("frontend"));
            }
            v
        }
    };
    for dir in dirs {
        let Ok(read) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut entries: Vec<String> = read
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n != ".git" && n != ".gitkeep")
            .collect();
        entries.sort();
        if !entries.is_empty() {
            let preview = entries
                .iter()
                .take(8)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            let more = if entries.len() > 8 {
                format!(", … (+{})", entries.len() - 8)
            } else {
                String::new()
            };
            let d = dir.display();
            return Err(CoreError::invalid_input(format!(
                "Target directory is not empty: {d}\n  Found: {preview}{more}\n\nLikely cause: a previous `dare init` for this project failed midway and left files behind.\nResolve by either:\n  1) Removing the directory and trying again:  Remove-Item -Recurse -Force \"{d}\"\n  2) Choosing a different project name.\n  3) Running `dare bootstrap --force` if you want to reuse the existing directory and overwrite framework files."
            )));
        }
    }
    Ok(())
}

// ─── Stack bootstrap orchestration ──────────────────────────────────────────

fn crate_prefix(cfg: &ProjectConfig) -> String {
    let p = cfg.crate_prefix.as_deref().map(str::trim).unwrap_or("");
    if !p.is_empty() {
        return p.to_string();
    }
    let s = npm_slug(&cfg.name);
    if s.is_empty() {
        "app".into()
    } else {
        s
    }
}

fn rust_monorepo_dir(cfg: &ProjectConfig, crate_type: &str) -> PathBuf {
    match cfg.layout() {
        RustWorkspaceLayout::Multi => cfg
            .output_dir
            .join("crates")
            .join(format!("{}-{crate_type}", crate_prefix(cfg))),
        RustWorkspaceLayout::Single => cfg.output_dir.join("crates").join(crate_type),
    }
}

fn rust_crate_name(cfg: &ProjectConfig, crate_type: &str) -> String {
    if !cfg.is_rust_monorepo() {
        return cfg.name.clone();
    }
    match cfg.layout() {
        RustWorkspaceLayout::Single => crate_type.to_string(),
        RustWorkspaceLayout::Multi => format!("{}-{crate_type}", crate_prefix(cfg)),
    }
}

/// `runStackBootstrap`.
fn run_stack_bootstrap(
    cfg: &ProjectConfig,
    env: &mut GenerateEnv<'_>,
    writer: &mut ProjectWriter,
) -> CoreResult<()> {
    let out = &cfg.output_dir;
    let rust_mono = cfg.is_rust_monorepo();
    let is_monorepo = cfg.structure == Structure::Monorepo;
    let mut cx = BootstrapCtx {
        writer,
        runner: &mut *env.runner,
        console: &mut *env.console,
        rails_offline: env.rails_offline,
    };

    if matches!(
        cfg.structure,
        Structure::Backend | Structure::Monorepo | Structure::Mvc
    ) {
        if let Some(backend) = cfg.backend {
            let dir = if rust_mono {
                rust_monorepo_dir(cfg, "server")
            } else if is_monorepo {
                out.join("backend")
            } else {
                out.clone()
            };
            cx.writer.ensure_dir(&dir)?;
            bootstrap_backend(
                &mut cx,
                backend,
                &dir,
                &rust_crate_name(cfg, "server"),
                cfg.toolchain,
                is_monorepo,
                cfg.structure == Structure::Mvc,
            )?;
        }
    }

    if matches!(cfg.structure, Structure::Frontend | Structure::Monorepo) {
        if let Some(frontend) = cfg.frontend.filter(|f| *f != FrontendStack::None) {
            let dir = if rust_mono {
                rust_monorepo_dir(cfg, "web")
            } else if is_monorepo {
                out.join("frontend")
            } else {
                out.clone()
            };
            cx.writer.ensure_dir(&dir)?;
            bootstrap_frontend(
                &mut cx,
                frontend,
                &dir,
                &rust_crate_name(cfg, "web"),
                cfg.toolchain,
                is_monorepo,
            )?;
        }
    }

    if cfg.structure == Structure::McpServer {
        let lang = cfg.mcp_language.unwrap_or(McpLanguage::NodeTs);
        let transport = cfg.mcp_transport.map_or("stdio", |t| t.scaffold_value());
        bootstrap_mcp(&mut cx, lang, out, &cfg.name, transport)?;
    }

    if rust_mono {
        create_rust_fullstack_workspace(cfg, &mut cx)?;
    }

    cx.console
        .log(Tone::Green, "\n✓ Stack scaffold complete.\n");
    Ok(())
}

/// `createRustFullstackWorkspace`.
fn create_rust_fullstack_workspace(
    cfg: &ProjectConfig,
    cx: &mut BootstrapCtx<'_>,
) -> CoreResult<()> {
    let out = &cfg.output_dir;
    let layout = cfg.layout();
    let frontend_label = if cfg.frontend == Some(FrontendStack::RustLeptosCsr) {
        "rust-leptos-csr"
    } else {
        "rust-leptos"
    };
    cx.console.log(
        Tone::Cyan,
        &format!(
            "\n🦀 Creating unified Cargo workspace (rust-axum + {frontend_label}, {})...\n",
            layout.as_str()
        ),
    );
    let full_slug = {
        let s = npm_slug(&cfg.name);
        if s.is_empty() {
            "app".to_string()
        } else {
            s
        }
    };
    let slug = match layout {
        RustWorkspaceLayout::Multi => {
            let p = cfg.crate_prefix.as_deref().map(str::trim).unwrap_or("");
            if p.is_empty() {
                full_slug
            } else {
                p.to_string()
            }
        }
        RustWorkspaceLayout::Single => full_slug,
    };
    let members: Vec<String> = match layout {
        RustWorkspaceLayout::Single => vec!["crates/server".into(), "crates/web".into()],
        RustWorkspaceLayout::Multi => ["core", "server", "web", "cli"]
            .iter()
            .map(|t| format!("crates/{slug}-{t}"))
            .collect(),
    };
    let (web, server) = match layout {
        RustWorkspaceLayout::Single => ("crates/web".to_string(), "crates/server".to_string()),
        RustWorkspaceLayout::Multi => (
            format!("crates/{slug}-web"),
            format!("crates/{slug}-server"),
        ),
    };

    let mut lines = vec![
        "[workspace]".to_string(),
        "resolver = \"2\"".into(),
        "members = [".into(),
    ];
    lines.extend(members.iter().map(|m| format!("  \"{m}\",")));
    lines.extend([
        "]".to_string(),
        String::new(),
        "# CRÍTICO: NÃO definir [build] target global aqui.".into(),
        format!("# Este workspace mistura crates WASM ({web}/Leptos)"),
        format!("# com crates nativos ({server}/Axum)."),
        "# cargo-leptos gerencia wasm32-unknown-unknown internamente.".into(),
        String::new(),
        "# Shared dependencies — reference in crates via { workspace = true }".into(),
        "[workspace.dependencies]".into(),
        "tokio = { version = \"1\", features = [\"full\"] }".into(),
        "serde = { version = \"1.0\", features = [\"derive\"] }".into(),
        "serde_json = \"1.0\"".into(),
        "tracing = \"0.1\"".into(),
        "anyhow = \"1.0\"".into(),
        "thiserror = \"1.0\"".into(),
        "uuid = { version = \"1.10\", features = [\"v4\", \"serde\"] }".into(),
        String::new(),
    ]);
    cx.writer.write(&out.join("Cargo.toml"), lines.join("\n"))?;
    cx.writer.write(
        &out.join(".cargo/config.toml"),
        "# DARE: Do NOT add a global [build] target here.
# This workspace mixes Leptos WASM crates with native Axum crates.
# A global target breaks one or the other — cargo-leptos manages wasm32 internally.
",
    )?;

    if layout == RustWorkspaceLayout::Multi {
        let core = out.join("crates").join(format!("{slug}-core"));
        cx.writer.ensure_dir(&core.join("src"))?;
        cx.writer.write(
            &core.join("Cargo.toml"),
            [
                "[package]".to_string(),
                format!("name = \"{slug}-core\""),
                "version = \"0.1.0\"".into(),
                "edition = \"2021\"".into(),
                String::new(),
                "[lib]".into(),
                String::new(),
                "[dependencies]".into(),
                "serde = { workspace = true }".into(),
                "serde_json = { workspace = true }".into(),
                "thiserror = { workspace = true }".into(),
                "anyhow = { workspace = true }".into(),
                "tracing = { workspace = true }".into(),
                "uuid = { workspace = true }".into(),
                String::new(),
            ]
            .join("\n"),
        )?;
        cx.writer.write(
            &core.join("src/lib.rs"),
            format!("// {slug}-core — shared domain types and business logic\n"),
        )?;

        let cli = out.join("crates").join(format!("{slug}-cli"));
        cx.writer.ensure_dir(&cli.join("src"))?;
        cx.writer.write(
            &cli.join("Cargo.toml"),
            [
                "[package]".to_string(),
                format!("name = \"{slug}-cli\""),
                "version = \"0.1.0\"".into(),
                "edition = \"2021\"".into(),
                String::new(),
                "[[bin]]".into(),
                format!("name = \"{slug}-cli\""),
                "path = \"src/main.rs\"".into(),
                String::new(),
                "[dependencies]".into(),
                format!("{slug}-core = {{ path = \"../{slug}-core\" }}"),
                "clap = { version = \"4\", features = [\"derive\"] }".into(),
                "anyhow = { workspace = true }".into(),
                "tokio = { workspace = true }".into(),
                String::new(),
            ]
            .join("\n"),
        )?;
        cx.writer.write(
            &cli.join("src/main.rs"),
            [
                "use clap::Parser;".to_string(),
                String::new(),
                "#[derive(Parser)]".into(),
                format!("#[command(name = \"{slug}-cli\")]"),
                "struct Cli {}".into(),
                String::new(),
                "#[tokio::main]".into(),
                "async fn main() {".into(),
                "    let _cli = Cli::parse();".into(),
                "}".into(),
                String::new(),
            ]
            .join("\n"),
        )?;
    }

    let root_gitignore = out.join(".gitignore");
    let rust_ignore = "\n# Rust / Cargo\ntarget/\ndist/\nCargo.lock\n";
    match std::fs::read_to_string(&root_gitignore) {
        Ok(existing) => {
            if !existing.contains("target/") {
                cx.writer.write(
                    &root_gitignore,
                    format!("{}{rust_ignore}", existing.trim_end()),
                )?;
            }
        }
        Err(_) => cx.writer.write(&root_gitignore, rust_ignore)?,
    }

    cx.console
        .log(Tone::Green, "  ✓ Cargo workspace root created");
    for m in &members {
        cx.console.log(Tone::Gray, &format!("    {m}/"));
    }
    cx.console.log(
        Tone::Yellow,
        "\n  ⚠  Cargo.lock gitignored — commit it for binary deployments.\n",
    );
    Ok(())
}

// ─── dare.config.json ────────────────────────────────────────────────────────

/// Key order matches the npm object literal; `None` fields are dropped
/// (npm `fs.writeJSON` omits `undefined`).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DareConfigJson<'a> {
    name: &'a str,
    structure: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    backend: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frontend: Option<&'a str>,
    ide: &'a str,
    graphrag: Value,
    mcp: bool,
    toolchain: &'a str,
    version: &'a str,
    review: Value,
    refine: Value,
    verification: Value,
    hooks: Value,
    guard: Value,
    drift: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    mcp_transport: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mcp_language: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mcp_features: Option<Vec<&'a str>>,
}

/// Default `verification` block (`verification/config.ts` `DEFAULTS`).
pub fn default_verification_config() -> Value {
    json!({
        "enabled": false,
        "mutation": {
            "enabled": true,
            "minScore": 0.7,
            "incremental": true,
            "maxMutants": 200,
            "timeoutSeconds": 900
        },
        "failToPass": { "required": true },
        "antiTamper": { "enabled": true },
        "typeCheck": { "enabled": false },
        "loop": {
            "policy": "decay",
            "maxAttempts": 5,
            "maxDepth": 2,
            "saturationWindow": 3,
            "onSaturation": "fresh-start"
        },
        "bestOfN": { "default": 1, "max": 5, "budgetTokens": null },
        "prerank": { "enabled": false },
        "formal": {
            "enabled": false,
            "backend": "dafny",
            "modules": [],
            "maxRepairIterations": 5,
            "proofTimeoutSeconds": 120,
            "antiBypass": true
        }
    })
}

/// `dare.config.json` body (2-space JSON + trailing newline, like `fs.writeJSON`).
pub fn dare_config_json(cfg: &ProjectConfig) -> CoreResult<String> {
    let is_mcp = cfg.structure == Structure::McpServer;
    let body = DareConfigJson {
        name: &cfg.name,
        structure: cfg.structure.as_str(),
        backend: cfg.backend.map(|b| b.as_str()),
        frontend: cfg.frontend.map(|f| f.as_str()),
        ide: cfg.ide.as_str(),
        graphrag: json!({
            "backend": cfg.graphrag.as_str(),
            "semantic": { "enabled": false, "model": "all-MiniLM-L6-v2", "rrfK": 60 }
        }),
        mcp: cfg.mcp,
        toolchain: cfg.toolchain.as_str(),
        version: FRAMEWORK_VERSION,
        review: json!({ "onComplete": true, "strict": false }),
        refine: json!({ "thresholds": { "low": 5, "med": 12, "high": 20 } }),
        verification: default_verification_config(),
        hooks: json!({ "on": {}, "trusted": false }),
        guard: json!({
            "enabled": false,
            "onExecute": true,
            "unicode": "strip",
            "trustedPaths": [".dare/steering/**", "DARE/TASKS.md"],
            "signing": { "enabled": false }
        }),
        drift: json!({
            "enabled": false,
            "maxOrphanReqs": 0,
            "maxOrphanCode": 0,
            "failOnStale": false,
            "ignore": ["**/index.ts", "**/*.generated.*", "**/bin/**"]
        }),
        mcp_transport: is_mcp
            .then(|| cfg.mcp_transport.map(|t| t.as_str()))
            .flatten(),
        mcp_language: is_mcp
            .then(|| cfg.mcp_language.map(|l| l.as_str()))
            .flatten(),
        mcp_features: is_mcp
            .then(|| {
                cfg.mcp_features
                    .as_ref()
                    .map(|fs| fs.iter().map(|f| f.as_str()).collect())
            })
            .flatten(),
    };
    let s = serde_json::to_string_pretty(&body)
        .map_err(|e| CoreError::internal(format!("serialize dare.config.json: {e}")))?;
    Ok(format!("{s}\n"))
}

// ─── .gitignore, README, dare-graph.yml ─────────────────────────────────────

fn merge_gitignore(cfg: &ProjectConfig, writer: &mut ProjectWriter) -> CoreResult<()> {
    let mut extras = vec![
        "# DARE Framework",
        ".dare/",
        "*.db",
        "*.db-shm",
        "*.db-wal",
        "DARE/.canvas.md",
    ];
    if cfg.structure == Structure::McpServer && cfg.mcp_language == Some(McpLanguage::Python) {
        extras.push(".venv/\n__pycache__/\n*.py[cod]");
    }
    let dare_extras = extras.join("\n");
    let path = cfg.output_dir.join(".gitignore");
    match std::fs::read_to_string(&path) {
        Ok(existing) => {
            if !existing.contains("# DARE Framework") {
                writer.write(&path, format!("{}\n{dare_extras}\n", existing.trim_end()))?;
            }
            Ok(())
        }
        Err(_) => writer.write(
            &path,
            [
                "node_modules/",
                "dist/",
                "build/",
                ".env",
                ".env.local",
                "logs/",
                "*.log",
                &dare_extras,
            ]
            .join("\n")
                + "\n",
        ),
    }
}

/// `generateSharedConfig` → `DARE/README.md`.
pub fn generate_shared_config(project_name: &str) -> String {
    format!(
        "# {project_name} — DARE Framework

## Design → Architect → Review → Execute

### Estrutura
- `DARE/` — documentação de design e execução
- `DARE/EXECUTION/` — tasks executadas e telemetria
- `templates/` — templates de documentação DARE

### Ralph Loop
Antes de marcar qualquer task como DONE:
1. Build ✅
2. Test ✅
3. Lint ✅

### Fluxo de Trabalho
1. `/generate-design` — cria DESIGN.md
2. `/generate-blueprint` — cria BLUEPRINT.md
3. `/generate-tasks` — cria TASKS.md
4. `/execute-task` — executa cada task com Ralph Loop
"
    )
}

const GRAPH_NODES_EDGES: &str = "
# Node types tracked in the graph
nodes:
  - task
  - file
  - schema
  - endpoint
  - component
  - entity
  - concept
  - code_symbol
  - requirement

# Relationship types
edges:
  - depends_on
  - implements
  - uses
  - references
  - related_to
  - contains
  - extends
  - affects
  - derives_from
";

/// `writeGraphragConfig` content.
pub fn graphrag_config(backend: GraphragBackend) -> String {
    let head = match backend {
        GraphragBackend::Sqlite => {
            "# DARE Knowledge Graph — SQLite
backend: sqlite

sqlite:
  path: .dare/graph.db
"
        }
        GraphragBackend::Neo4j => {
            "# DARE Knowledge Graph — Neo4j
backend: neo4j

neo4j:
  url: http://localhost:7474   # Neo4j HTTP API endpoint (not Bolt)
  database: dare               # database name
  username: neo4j
  password: password
  # auth: \"Bearer <token>\"     # alternative to username/password
"
        }
        GraphragBackend::Json => {
            "# DARE Knowledge Graph — JSON
backend: json

json:
  path: .dare/graph.json
"
        }
    };
    format!("{head}{GRAPH_NODES_EDGES}")
}

fn write_graphrag_config(cfg: &ProjectConfig, writer: &mut ProjectWriter) -> CoreResult<()> {
    let dest = cfg.output_dir.join("dare-graph.yml");
    if dest.exists() {
        return Ok(());
    }
    writer.write(&dest, graphrag_config(cfg.graphrag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::config::{BackendStack, Ide, McpTransport, ToolchainMode};
    use crate::project::console::MemoryConsole;
    use crate::project::tools::RecordingToolRunner;

    fn cfg(root: &Path, structure: Structure) -> ProjectConfig {
        ProjectConfig {
            name: "api-nest-validations".into(),
            structure,
            backend: None,
            frontend: None,
            mcp_transport: None,
            mcp_language: None,
            mcp_features: None,
            ide: Ide::ClaudeCode,
            graphrag: GraphragBackend::Sqlite,
            mcp: true,
            toolchain: ToolchainMode::Auto,
            rust_workspace_layout: None,
            crate_prefix: None,
            output_dir: root.join("api-nest-validations"),
        }
    }

    fn run(
        c: &ProjectConfig,
        cwd: &Path,
        tools: &[&str],
    ) -> (CoreResult<GenerateReport>, RecordingToolRunner) {
        let mut runner = RecordingToolRunner::with_tools(tools);
        let mut console = MemoryConsole::default();
        let mut env = GenerateEnv {
            cwd,
            runner: &mut runner,
            console: &mut console,
            rails_offline: false,
            force: false,
        };
        let res = generate_project_structure(c, &mut env);
        (res, runner)
    }

    #[test]
    fn monorepo_nestjs_vue_matches_npm_layout() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut c = cfg(tmp.path(), Structure::Monorepo);
        c.backend = Some(BackendStack::NodeNestjs);
        c.frontend = Some(FrontendStack::Vue);
        let (res, runner) = run(&c, tmp.path(), &["npm"]);
        res.expect("generate");
        let root = &c.output_dir;
        assert!(root.join("backend/src/app.module.ts").is_file());
        assert!(root.join("backend/tsconfig.json").is_file());
        assert!(
            !root.join("src").exists(),
            "backend must not land in the root"
        );
        assert_eq!(runner.commands[0].cwd, root.join("frontend"));
        assert!(root.join("CLAUDE.md").is_file());
        assert!(root.join(".claude/settings.json").is_file());
        assert!(
            !root.join(".cursorrules").exists(),
            "claude-code installs only Claude"
        );
        assert!(!root.join("Dockerfile").exists());
        assert!(root.join("DARE/EXECUTION").is_dir());
        assert!(root.join("templates/DESIGN-template.md").is_file());
        let cfg_json: Value =
            serde_json::from_str(&std::fs::read_to_string(root.join("dare.config.json")).unwrap())
                .unwrap();
        assert_eq!(cfg_json["structure"], "monorepo");
        assert_eq!(cfg_json["backend"], "node-nestjs");
        assert_eq!(cfg_json["frontend"], "vue");
        let gi = std::fs::read_to_string(root.join(".gitignore")).unwrap();
        assert!(gi.starts_with("node_modules/\n"));
    }

    #[test]
    fn config_json_key_order_matches_npm() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut c = cfg(tmp.path(), Structure::McpServer);
        c.mcp_language = Some(McpLanguage::Go);
        c.mcp_transport = Some(McpTransport::HttpStream);
        let s = dare_config_json(&c).unwrap();
        let keys: Vec<&str> = s
            .lines()
            .filter(|l| l.starts_with("  \"") && !l.starts_with("   "))
            .map(|l| l.trim().split('"').nth(1).unwrap())
            .collect();
        assert_eq!(
            keys,
            [
                "name",
                "structure",
                "ide",
                "graphrag",
                "mcp",
                "toolchain",
                "version",
                "review",
                "refine",
                "verification",
                "hooks",
                "guard",
                "drift",
                "mcpTransport",
                "mcpLanguage"
            ]
        );
        assert!(s.ends_with("}\n"));
    }

    #[test]
    fn failure_rolls_back_only_created_paths() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut c = cfg(tmp.path(), Structure::Frontend);
        c.frontend = Some(FrontendStack::Vue);
        // native npm missing → error after the output dir was created.
        c.toolchain = ToolchainMode::Native;
        let (res, _) = run(&c, tmp.path(), &[]);
        assert!(res.is_err());
        assert!(!c.output_dir.exists());
    }

    #[test]
    fn rust_multi_workspace() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut c = cfg(tmp.path(), Structure::Monorepo);
        c.backend = Some(BackendStack::RustAxum);
        c.frontend = Some(FrontendStack::RustLeptosCsr);
        c.rust_workspace_layout = Some(RustWorkspaceLayout::Multi);
        c.crate_prefix = Some("ars".into());
        c.ide = Ide::Cursor;
        let (res, _) = run(&c, tmp.path(), &["cargo"]);
        res.expect("generate");
        let root = &c.output_dir;
        for m in ["ars-core", "ars-server", "ars-web", "ars-cli"] {
            assert!(
                root.join("crates").join(m).join("Cargo.toml").is_file(),
                "{m}"
            );
        }
        let ws = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
        assert!(ws.contains("  \"crates/ars-web\",\n"));
        let server = std::fs::read_to_string(root.join("crates/ars-server/Cargo.toml")).unwrap();
        assert!(server.contains("ars_server"), "{server}");
    }
}
