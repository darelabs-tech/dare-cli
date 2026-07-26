//! `dare steering list|show` — deterministic steering inspection (microplano 048).

use std::path::PathBuf;

use dare_core::{CoreError, CoreResult, ProjectRoot};
use dare_steering::{list_steering, show_steering, SteeringListReport, SteeringShowReport};
use serde_json::Value;

/// List steering sources under project root (`DARE/PROJECT-DNA.md`, `PATTERNS.md`, `.dare/steering/*.md`).
pub fn run_steering_list(dir: Option<PathBuf>) -> CoreResult<(String, Value)> {
    let root = resolve_root(dir)?;
    let report = list_steering(&root)?;
    let human = format_list_human(&report);
    let data = serde_json::to_value(&report).map_err(|e| CoreError::internal(e.to_string()))?;
    Ok((human, data))
}

/// Show steering blocks applicable to `file` (project-relative path).
pub fn run_steering_show(file: String, dir: Option<PathBuf>) -> CoreResult<(String, Value)> {
    let root = resolve_root(dir)?;
    let report = show_steering(&root, &file)?;
    let human = format_show_human(&report);
    let data = serde_json::to_value(&report).map_err(|e| CoreError::internal(e.to_string()))?;
    Ok((human, data))
}

fn resolve_root(dir: Option<PathBuf>) -> CoreResult<ProjectRoot> {
    let path =
        dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    ProjectRoot::new(&path)
}

fn format_list_human(report: &SteeringListReport) -> String {
    let mut lines = vec![format!(
        "steering list: {} file(s)",
        report.files.len()
    )];
    for f in &report.files {
        let glob = f.glob.as_deref().unwrap_or("-");
        lines.push(format!(
            "  {}  scope={}  glob={}  priority={}",
            f.path, f.scope, glob, f.priority
        ));
    }
    for w in &report.warnings {
        lines.push(format!("warning: {w}"));
    }
    lines.join("\n")
}

fn format_show_human(report: &SteeringShowReport) -> String {
    let mut lines = vec![format!(
        "steering show: {} → {} block(s)",
        report.target,
        report.blocks.len()
    )];
    for b in &report.blocks {
        let glob = b.glob.as_deref().unwrap_or("-");
        lines.push(format!(
            "--- {} (scope={} glob={} priority={}) ---",
            b.path, b.scope, glob, b.priority
        ));
        if !b.body.is_empty() {
            lines.push(b.body.trim_end().to_string());
        }
    }
    lines.join("\n")
}
