//! `dare bootstrap` — re-apply scaffold on existing greenfield project (microplano 047).

use std::path::PathBuf;

use dare_core::{CoreError, CoreResult, ProjectRoot};
use dare_scaffold::Toolchain;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const BOOTSTRAP_REPORT_SCHEMA: u32 = 1;

#[allow(dead_code)] // used by bootstrap FS pipeline (mp047-005)
pub const MSG_MISSING_CONFIG: &str = "dare.config.json not found";
#[allow(dead_code)] // used by bootstrap FS pipeline (mp047-005)
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
}

fn parse_toolchain(input: &str) -> CoreResult<Toolchain> {
    match input.to_ascii_lowercase().as_str() {
        "none" => Ok(Toolchain::None),
        "docker" => Ok(Toolchain::Docker),
        other => Err(CoreError::invalid_input(format!("unknown toolchain: {other}"))),
    }
}

/// Stub bootstrap runner — full FS pipeline deferred to mp047-005.
pub fn run_bootstrap(root: &ProjectRoot, req: &BootstrapRequest) -> CoreResult<BootstrapReport> {
    let _ = req;
    Ok(BootstrapReport {
        schema_version: BOOTSTRAP_REPORT_SCHEMA,
        mode: "bootstrap".to_string(),
        project_root: root.as_path().to_string(),
        stack_id: String::new(),
        toolchain: req
            .toolchain_override
            .unwrap_or(Toolchain::None),
        created: Vec::new(),
        replaced: Vec::new(),
        skipped: Vec::new(),
        rolled_back: false,
        check: req.check,
    })
}

pub fn bootstrap_report_to_json(report: &BootstrapReport) -> CoreResult<String> {
    serde_json::to_string_pretty(report)
        .map_err(|e| CoreError::io(format!("serialize bootstrap report: {e}")))
}

pub fn run_bootstrap_cmd(opts: BootstrapCliOpts) -> CoreResult<(String, Value)> {
    let start = opts
        .dir
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let toolchain_override = opts
        .toolchain
        .as_deref()
        .map(parse_toolchain)
        .transpose()?;

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
