//! `dare discover` — brownfield detection and install.

use std::path::{Path, PathBuf};

use dare_core::CoreResult;
use dare_project::{
    detect, format_human, format_install_human, install, install_report_to_json, report_to_json,
    InstallOptions,
};
use serde_json::Value;

/// Run discover: `--check` → detect only; otherwise install.
pub fn run_discover(
    dir: Option<PathBuf>,
    check: bool,
    force: bool,
    dry_run: bool,
    strict_conflicts: bool,
) -> CoreResult<(String, Value)> {
    let start: PathBuf =
        dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    if check {
        let report = detect(Path::new(&start))?;
        let human = format_human(&report);
        let data = report_to_json(&report);
        return Ok((human, data));
    }

    let opts = InstallOptions {
        force,
        dry_run,
        strict_conflicts,
    };
    let report = install(Path::new(&start), &opts)?;
    let human = format_install_human(&report);
    let data = install_report_to_json(&report);
    Ok((human, data))
}
