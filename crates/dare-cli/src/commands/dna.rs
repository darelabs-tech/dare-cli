//! `dare dna` — extract project conventions (microplano 037).

use std::path::PathBuf;

use dare_core::CoreResult;
use dare_project::{dna_report_to_json, format_dna_human, run_dna, DnaOptions};
use serde_json::Value;

/// Run DNA extraction / check.
pub fn run_dna_cmd(dir: Option<PathBuf>, check: bool, ast: bool) -> CoreResult<(String, Value)> {
    let start =
        dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let report = run_dna(&DnaOptions {
        dir: start,
        check,
        ast,
    })?;
    let human = format_dna_human(&report);
    let data = dna_report_to_json(&report);
    Ok((human, data))
}
