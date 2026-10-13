//! `dare bootstrap` — re-run the stack scaffold on an existing project.
//!
//! Projects created by the npm-parity `dare init` (cycle 011, `structure` key
//! in `dare.config.json`) follow npm `commands/bootstrap.ts`. Projects created
//! by Rust 4.0 init (`stack` key) keep the microplano 047 skeleton re-apply.

use std::path::PathBuf;

use dare_core::fs::{atomic_write, read_to_string};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use dare_scaffold::project::bootstrap::{
    bootstrap_backend, bootstrap_frontend, bootstrap_mcp, BootstrapCtx,
};
use dare_scaffold::project::console::{Console, StdConsole, Tone};
use dare_scaffold::project::tools::SystemToolRunner;
use dare_scaffold::project::writer::ProjectWriter;
use dare_scaffold::project::{BackendStack, FrontendStack, McpLanguage, ToolchainMode};
use dare_scaffold::{
    run_scaffold, ConflictPolicy, FrontendKind, ScaffoldRequest, Toolchain, Transport,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const BOOTSTRAP_REPORT_SCHEMA: u32 = 1;
const CONFIG_REL: &str = "dare.config.json";

pub const MSG_MISSING_CONFIG: &str = "dare.config.json not found";
pub const MSG_MISSING_STACK_FIELD: &str = "dare.config.json missing stack";

/// Resolved bootstrap domain request (BLUEPRINT-047 §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapRequest {
    pub toolchain_override: Option<Toolchain>,
    pub force: bool,
    pub check: bool,
}

/// Bootstrap execution report (schemaVersion 1, camelCase JSON).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapReport {
    pub schema_version: u32,
    pub mode: String,
    pub project_root: String,
    pub stack_id: String,
    pub toolchain: Toolchain,
    pub created: Vec<String>,
    pub replaced: Vec<String>,
    pub skipped: Vec<String>,
    pub rolled_back: bool,
    pub check: bool,
}

pub struct BootstrapCliOpts {
    pub dir: Option<PathBuf>,
    pub toolchain: Option<String>,
    pub force: bool,
    pub check: bool,
    pub json: bool,
    pub color: bool,
}

fn parse_toolchain(input: &str) -> CoreResult<Toolchain> {
    match input.to_ascii_lowercase().as_str() {
        "none" => Ok(Toolchain::None),
        "docker" => Ok(Toolchain::Docker),
        other => Err(CoreError::invalid_input(format!(
            "unknown toolchain: {other}"
        ))),
    }
}

fn toolchain_to_config_str(toolchain: Toolchain) -> &'static str {
    match toolchain {
        Toolchain::None => "none",
        Toolchain::Docker => "docker",
    }
}

fn load_config_value(root: &ProjectRoot) -> CoreResult<Value> {
    let rel = SafeRelativePath::new(CONFIG_REL)?;
    let raw = match read_to_string(root, &rel) {
        Ok(s) => s,
        Err(CoreError::NotFound(_)) => return Err(CoreError::not_found(MSG_MISSING_CONFIG)),
        Err(e) => return Err(e),
    };
    serde_json::from_str(&raw).map_err(|e| CoreError::config(format!("invalid {CONFIG_REL}: {e}")))
}

fn parse_stack_id(config: &Value) -> CoreResult<String> {
    config
        .get("stack")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| CoreError::invalid_input(MSG_MISSING_STACK_FIELD))
}

fn parse_project_name(config: &Value, root: &ProjectRoot) -> String {
    config
        .get("projectName")
        .or_else(|| config.get("project_name"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            root.as_path()
                .file_name()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "project".to_string())
        })
}

fn parse_config_toolchain(config: &Value) -> CoreResult<Toolchain> {
    match config.get("toolchain").and_then(|v| v.as_str()) {
        Some(s) => parse_toolchain(s),
        None => Ok(Toolchain::None),
    }
}

fn parse_config_frontend(config: &Value) -> CoreResult<Option<FrontendKind>> {
    match config.get("frontend").and_then(|v| v.as_str()) {
        Some(s) => match s.to_ascii_lowercase().as_str() {
            "react" => Ok(Some(FrontendKind::React)),
            "vue" => Ok(Some(FrontendKind::Vue)),
            other => Err(CoreError::invalid_input(format!(
                "unknown frontend: {other}"
            ))),
        },
        None => Ok(None),
    }
}

fn parse_config_transport(config: &Value) -> CoreResult<Option<Transport>> {
    match config.get("transport").and_then(|v| v.as_str()) {
        Some(s) => match s.to_ascii_lowercase().as_str() {
            "stdio" => Ok(Some(Transport::Stdio)),
            "http" => Ok(Some(Transport::Http)),
            "sse" => Ok(Some(Transport::Sse)),
            other => Err(CoreError::invalid_input(format!(
                "unknown transport: {other}"
            ))),
        },
        None => Ok(None),
    }
}

fn persist_toolchain(root: &ProjectRoot, toolchain: Toolchain) -> CoreResult<()> {
    let rel = SafeRelativePath::new(CONFIG_REL)?;
    let raw = read_to_string(root, &rel)?;
    let mut config: Value = serde_json::from_str(&raw)
        .map_err(|e| CoreError::config(format!("invalid {CONFIG_REL}: {e}")))?;
    if let Value::Object(ref mut map) = config {
        map.insert(
            "toolchain".to_string(),
            Value::String(toolchain_to_config_str(toolchain).to_string()),
        );
    }
    let out = serde_json::to_string_pretty(&config)
        .map_err(|e| CoreError::io(format!("serialize config: {e}")))?;
    atomic_write(root, &rel, out.as_bytes())
}

/// Re-apply scaffold artefacts from existing `dare.config.json` (BLUEPRINT-047 §5.2).
pub fn run_bootstrap(root: &ProjectRoot, req: &BootstrapRequest) -> CoreResult<BootstrapReport> {
    let config = load_config_value(root)?;
    let stack_id = parse_stack_id(&config)?;
    let project_name = parse_project_name(&config, root);
    let config_toolchain = parse_config_toolchain(&config)?;
    let toolchain = req.toolchain_override.unwrap_or(config_toolchain);
    let frontend = parse_config_frontend(&config)?;
    let transport = parse_config_transport(&config)?;

    let (force, conflict_policy) = if req.force {
        (true, ConflictPolicy::FailFast)
    } else {
        (false, ConflictPolicy::SkipExisting)
    };

    let scaffold_req = ScaffoldRequest {
        project_name,
        stack_id: stack_id.clone(),
        toolchain,
        transport,
        frontend,
        conflict_policy,
        force,
        check: req.check,
    };

    let scaffold_report = run_scaffold(root, &scaffold_req)?;

    if req.toolchain_override.is_some() && !req.check {
        persist_toolchain(root, toolchain)?;
    }

    Ok(BootstrapReport {
        schema_version: BOOTSTRAP_REPORT_SCHEMA,
        mode: "bootstrap".to_string(),
        project_root: root.as_path().to_string(),
        stack_id,
        toolchain,
        created: scaffold_report.created,
        replaced: scaffold_report.replaced,
        skipped: scaffold_report.skipped,
        rolled_back: scaffold_report.rolled_back,
        check: req.check,
    })
}

pub fn bootstrap_report_to_json(report: &BootstrapReport) -> CoreResult<String> {
    serde_json::to_string_pretty(report)
        .map_err(|e| CoreError::io(format!("serialize bootstrap report: {e}")))
}

// ─── npm-parity path (`structure` in dare.config.json) ───────────────────────

/// npm `detectConflicts` sentinels.
const CONFLICT_SENTINELS: &[&str] = &[
    "vendor",
    "composer.lock",
    "node_modules",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "Cargo.lock",
    "target",
];

fn str_field<'a>(config: &'a Value, key: &str) -> Option<&'a str> {
    config.get(key).and_then(Value::as_str)
}

fn parse_frontend_id(f: &str) -> CoreResult<FrontendStack> {
    FrontendStack::parse(f)
        .ok_or_else(|| CoreError::invalid_input(format!("Unknown frontend stack: {f}")))
}

/// Port of npm `bootstrap.ts` action body.
pub fn run_npm_bootstrap(
    root: &std::path::Path,
    config: &Value,
    toolchain_flag: Option<&str>,
    force: bool,
    check: bool,
    console: &mut dyn Console,
) -> CoreResult<Value> {
    let mut toolchain = str_field(config, "toolchain")
        .and_then(ToolchainMode::parse)
        .unwrap_or(ToolchainMode::Auto);
    if let Some(flag) = toolchain_flag {
        toolchain = ToolchainMode::parse(flag).ok_or_else(|| {
            CoreError::invalid_input(format!(
                "❌ Invalid --toolchain \"{flag}\". Use: auto | native | docker."
            ))
        })?;
    }
    let project_name = str_field(config, "name")
        .map(str::to_string)
        .unwrap_or_else(|| {
            root.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "project".into())
        });
    if !force {
        let conflicts: Vec<&str> = CONFLICT_SENTINELS
            .iter()
            .copied()
            .filter(|f| root.join(f).exists())
            .collect();
        if !conflicts.is_empty() {
            let list: String = conflicts.iter().map(|f| format!("\n   - {f}")).collect();
            return Err(CoreError::invalid_input(format!(
                "❌ Refusing to run — found framework artifacts that the scaffold would overwrite:{list}\n\n  Use --force to run anyway (may overwrite existing files)."
            )));
        }
    }
    let structure = str_field(config, "structure").unwrap_or_default();
    let backend = str_field(config, "backend");
    let frontend = str_field(config, "frontend");
    let mcp_language = str_field(config, "mcpLanguage");
    let label = backend.or(frontend).or(mcp_language).unwrap_or("undefined");
    let data = json!({
        "schemaVersion": BOOTSTRAP_REPORT_SCHEMA,
        "mode": "bootstrap",
        "projectRoot": root.to_string_lossy().replace('\\', "/"),
        "structure": structure,
        "backend": backend,
        "frontend": frontend,
        "mcpLanguage": mcp_language,
        "toolchain": toolchain.as_str(),
        "check": check,
    });
    if check {
        return Ok(data);
    }
    console.log(
        Tone::BlueBold,
        &format!("\n🚀 Running scaffold for {structure}/{label}\n"),
    );

    let mut writer = ProjectWriter::new();
    let mut runner = SystemToolRunner;
    let mut cx = BootstrapCtx {
        writer: &mut writer,
        runner: &mut runner,
        console,
        rails_offline: std::env::var("DARE_RAILS_OFFLINE").as_deref() == Ok("1"),
    };
    let result = match (structure, backend, frontend) {
        ("mcp-server", _, _) => McpLanguage::parse(mcp_language.unwrap_or("node-ts"))
            .ok_or_else(|| CoreError::invalid_input("Unknown MCP language"))
            // npm bootstrap passes no transport → scaffolder default stdio.
            .and_then(|lang| bootstrap_mcp(&mut cx, lang, root, &project_name, "stdio")),
        ("frontend", _, Some(f)) => parse_frontend_id(f)
            .and_then(|f| bootstrap_frontend(&mut cx, f, root, &project_name, toolchain, false)),
        ("backend" | "monorepo" | "mvc", Some(b), _) => {
            let run = |cx: &mut BootstrapCtx<'_>| -> CoreResult<()> {
                let stack = BackendStack::parse(b).ok_or_else(|| {
                    CoreError::invalid_input(format!("Unknown backend stack: {b}"))
                })?;
                let dir = if structure == "monorepo" {
                    root.join("backend")
                } else {
                    root.to_path_buf()
                };
                cx.writer.ensure_dir(&dir)?;
                bootstrap_backend(
                    cx,
                    stack,
                    &dir,
                    &project_name,
                    toolchain,
                    false,
                    structure == "mvc",
                )?;
                if let (true, Some(f)) = (structure == "monorepo", frontend) {
                    let fdir = root.join("frontend");
                    cx.writer.ensure_dir(&fdir)?;
                    bootstrap_frontend(
                        cx,
                        parse_frontend_id(f)?,
                        &fdir,
                        &project_name,
                        toolchain,
                        false,
                    )?;
                }
                Ok(())
            };
            run(&mut cx)
        }
        _ => {
            return Err(CoreError::invalid_input(
                "❌ dare.config.json has no stack to bootstrap.",
            ))
        }
    };
    result
        .map(|()| data)
        .map_err(|e| CoreError::invalid_input(format!("❌ Bootstrap failed: {}", e.message())))
}

pub fn run_bootstrap_cmd(opts: BootstrapCliOpts) -> CoreResult<(String, Value)> {
    let start = opts
        .dir
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let npm_config = std::fs::read_to_string(start.join(CONFIG_REL))
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .filter(|c| c.get("structure").is_some());
    if let Some(config) = npm_config {
        let mut console = StdConsole::new(opts.color, opts.json);
        let data = run_npm_bootstrap(
            &start,
            &config,
            opts.toolchain.as_deref(),
            opts.force,
            opts.check,
            &mut console,
        )?;
        let human = if opts.check {
            "bootstrap --check: plan only (no writes)".to_string()
        } else {
            "\n✅ Bootstrap complete. Your DARE artifacts (.cursor/, DARE/, etc.) were preserved.\n"
                .to_string()
        };
        return Ok((human, data));
    }

    let toolchain_override = opts.toolchain.as_deref().map(parse_toolchain).transpose()?;

    let req = BootstrapRequest {
        toolchain_override,
        force: opts.force,
        check: opts.check,
    };

    let root = ProjectRoot::new(&start)?;
    let report = run_bootstrap(&root, &req)?;
    let json_str = bootstrap_report_to_json(&report)?;
    let data: Value = serde_json::from_str(&json_str)
        .map_err(|e| CoreError::io(format!("parse bootstrap report json: {e}")))?;
    let human = format!(
        "bootstrap: stack={} check={}",
        report.stack_id, report.check
    );
    Ok((human, data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ErrorKind;
    use std::fs;
    use tempfile::tempdir;

    fn seed_config(dir: &std::path::Path, project_name: &str, stack: &str, toolchain: &str) {
        let config = serde_json::json!({
            "schemaVersion": 1,
            "projectName": project_name,
            "stack": stack,
            "toolchain": toolchain,
        });
        fs::write(
            dir.join(CONFIG_REL),
            serde_json::to_string_pretty(&config).unwrap(),
        )
        .unwrap();
    }

    fn bootstrap_req(force: bool, check: bool) -> BootstrapRequest {
        BootstrapRequest {
            toolchain_override: None,
            force,
            check,
        }
    }

    #[test]
    fn bootstrap_idempotent() {
        let dir = tempdir().expect("tempdir");
        seed_config(dir.path(), "demo-app", "go-gin", "none");
        let root = ProjectRoot::new(dir.path()).expect("project root");

        let first = run_bootstrap(&root, &bootstrap_req(false, false)).expect("first bootstrap");
        assert!(!first.created.is_empty(), "first run should create files");
        assert_eq!(first.mode, "bootstrap");

        let second = run_bootstrap(&root, &bootstrap_req(false, false)).expect("second bootstrap");
        assert!(
            second.created.is_empty(),
            "second run should not create files: {:?}",
            second.created
        );
    }

    #[test]
    fn bootstrap_force_replaces() {
        let dir = tempdir().expect("tempdir");
        seed_config(dir.path(), "demo-app", "go-gin", "none");
        let root = ProjectRoot::new(dir.path()).expect("project root");

        run_bootstrap(&root, &bootstrap_req(false, false)).expect("initial bootstrap");

        let rel = SafeRelativePath::new("README.md").expect("safe path");
        atomic_write(&root, &rel, b"# stale content").expect("seed stale ax file");

        let req = BootstrapRequest {
            toolchain_override: Some(Toolchain::Docker),
            force: true,
            check: false,
        };
        let report = run_bootstrap(&root, &req).expect("force bootstrap");
        assert!(
            !report.replaced.is_empty(),
            "force bootstrap should replace existing files"
        );

        let config_raw = fs::read_to_string(dir.path().join(CONFIG_REL)).expect("read config");
        assert!(
            config_raw.contains("\"docker\""),
            "toolchain override must persist on disk: {config_raw}"
        );
        let readme = fs::read_to_string(dir.path().join("README.md")).expect("read readme");
        assert!(
            !readme.contains("stale content"),
            "force should replace stale AX content"
        );
    }

    #[test]
    fn npm_bootstrap_backend_and_conflicts() {
        use dare_scaffold::project::console::MemoryConsole;
        let dir = tempdir().expect("tempdir");
        let config = serde_json::json!({
            "name": "shop", "structure": "backend", "backend": "go-gin", "toolchain": "auto"
        });
        let mut c = MemoryConsole::default();
        run_npm_bootstrap(dir.path(), &config, None, false, false, &mut c).expect("bootstrap");
        assert!(dir.path().join("cmd/server/main.go").is_file());
        assert!(c
            .lines
            .iter()
            .any(|l| l.contains("Running scaffold for backend/go-gin")));

        fs::create_dir_all(dir.path().join("vendor")).unwrap();
        let err = run_npm_bootstrap(dir.path(), &config, None, false, false, &mut c).unwrap_err();
        assert!(err.message().contains("   - vendor"), "{}", err.message());
        let err =
            run_npm_bootstrap(dir.path(), &config, Some("none"), true, false, &mut c).unwrap_err();
        assert!(err.message().contains("Invalid --toolchain"));
    }

    #[test]
    fn bootstrap_missing_config() {
        let dir = tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("project root");

        let err = run_bootstrap(&root, &bootstrap_req(false, false)).expect_err("missing config");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert_eq!(err.message(), MSG_MISSING_CONFIG);
    }

    #[test]
    fn bootstrap_check_zero_write() {
        let dir = tempdir().expect("tempdir");
        seed_config(dir.path(), "demo-app", "go-gin", "none");
        let root = ProjectRoot::new(dir.path()).expect("project root");

        let before: Vec<_> = fs::read_dir(dir.path())
            .expect("read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();

        let report = run_bootstrap(&root, &bootstrap_req(false, true)).expect("check bootstrap");
        assert!(report.check);
        assert!(report.created.is_empty());
        assert!(report.replaced.is_empty());
        assert!(!report.skipped.is_empty());

        let after: Vec<_> = fs::read_dir(dir.path())
            .expect("read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(before, after, "check mode must not write files");
    }
}
