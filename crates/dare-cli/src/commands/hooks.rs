//! `dare hooks list|run|validate` — deterministic hooks CLI (microplano 048).

use std::path::PathBuf;

use dare_config::{load_effective, CliOverrides, EnvOverrides, DEFAULT_CONFIG_REL};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath, SystemProcessRunner};
use dare_hooks::{
    list_hooks, run_hooks, validate_hooks, HookEvent, HooksListReport, HooksRunReport,
    HooksValidateReport, RunHooksRequest,
};
use serde_json::Value;

/// List hooks defs for the project (`embedded` or `.dare/hooks.yml` overlay).
pub fn run_hooks_list(dir: Option<PathBuf>) -> CoreResult<(String, Value)> {
    let root = resolve_root(dir)?;
    let cfg = load_cfg(&root)?;
    let report = list_hooks(&root, &cfg)?;
    let human = format_list_human(&report);
    let data = serde_json::to_value(&report).map_err(|e| CoreError::internal(e.to_string()))?;
    Ok((human, data))
}

/// Validate hooks defs; `ok=false` → `InvalidInput` (exit 4).
pub fn run_hooks_validate(dir: Option<PathBuf>) -> CoreResult<(String, Value)> {
    let root = resolve_root(dir)?;
    let cfg = load_cfg(&root)?;
    let report = validate_hooks(&root, &cfg)?;
    if !report.ok {
        let msg = if report.errors.is_empty() {
            "hooks validate failed".to_string()
        } else {
            report.errors.join("; ")
        };
        return Err(CoreError::invalid_input(msg));
    }
    let human = format_validate_human(&report);
    let data = serde_json::to_value(&report).map_err(|e| CoreError::internal(e.to_string()))?;
    Ok((human, data))
}

/// Run hooks for `event`; unknown event → usage (exit 2); any `failed` → internal (exit 1).
pub fn run_hooks_cmd(
    event: String,
    file: Option<String>,
    task: Option<String>,
    trust: bool,
    dir: Option<PathBuf>,
) -> CoreResult<(String, Value)> {
    let root = resolve_root(dir)?;
    let cfg = load_cfg(&root)?;
    let parsed = HookEvent::parse(&event)?;
    let req = RunHooksRequest {
        event: parsed,
        file: file.as_deref(),
        task: task.as_deref(),
        trust_flag: trust,
    };
    let report = run_hooks(&root, &cfg, &req, &SystemProcessRunner)?;
    if report.results.iter().any(|r| r.status == "failed") {
        return Err(CoreError::internal("hook action failed"));
    }
    let human = format_run_human(&report);
    let data = serde_json::to_value(&report).map_err(|e| CoreError::internal(e.to_string()))?;
    Ok((human, data))
}

fn resolve_root(dir: Option<PathBuf>) -> CoreResult<ProjectRoot> {
    let path =
        dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    ProjectRoot::new(&path)
}

fn load_cfg(root: &ProjectRoot) -> CoreResult<dare_contracts::DareConfig> {
    let rel = SafeRelativePath::new(DEFAULT_CONFIG_REL)?;
    load_effective(
        root,
        &rel,
        &EnvOverrides::default(),
        &CliOverrides::default(),
    )
}

fn format_list_human(report: &HooksListReport) -> String {
    let mut lines = vec![format!(
        "hooks list: {} hook(s) source={} enabled={} trusted={}",
        report.hooks.len(),
        report.source,
        report.enabled,
        report.trusted
    )];
    for h in &report.hooks {
        lines.push(format!("  {}  actions=[{}]", h.event, h.actions.join(", ")));
    }
    lines.join("\n")
}

fn format_validate_human(report: &HooksValidateReport) -> String {
    let mut lines = vec![format!(
        "hooks validate: ok source={}",
        report.source
    )];
    for w in &report.warnings {
        lines.push(format!("warning: {w}"));
    }
    lines.join("\n")
}

fn format_run_human(report: &HooksRunReport) -> String {
    let mut lines = vec![format!(
        "hooks run: event={} results={}",
        report.event,
        report.results.len()
    )];
    for r in &report.results {
        let code = r
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".into());
        let reason = r.reason.as_deref().unwrap_or("-");
        lines.push(format!(
            "  {}  status={}  exit={}  reason={}",
            r.action, r.status, code, reason
        ));
    }
    lines.join("\n")
}
