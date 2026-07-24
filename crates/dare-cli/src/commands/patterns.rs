//! `dare patterns` - deterministic pattern mining (microplano 038).

use std::path::PathBuf;

use dare_core::CoreResult;
use dare_project::patterns::{
    format_human as format_patterns_human, report_to_json as patterns_report_to_json, run_patterns,
    PatternsOptions,
};
use serde_json::Value;

/// Parse comma-separated `--modules` into trimmed ids.
pub fn parse_modules_csv(raw: Option<String>) -> CoreResult<Vec<String>> {
    let Some(s) = raw else {
        return Ok(Vec::new());
    };
    let parts: Vec<String> = s
        .split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        return Err(dare_core::CoreError::invalid_input(
            "--modules requires at least one module id",
        ));
    }
    Ok(parts)
}

/// Run pattern mining / check.
pub fn run_patterns_cmd(
    dir: Option<PathBuf>,
    check: bool,
    inject: bool,
    ast: bool,
    modules: Option<String>,
) -> CoreResult<(String, Value)> {
    let start =
        dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let modules = parse_modules_csv(modules)?;
    let report = run_patterns(&PatternsOptions {
        dir: start,
        check,
        inject,
        ast,
        modules,
    })?;
    let human = format_patterns_human(&report);
    let data = patterns_report_to_json(&report);
    Ok((human, data))
}
