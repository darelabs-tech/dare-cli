//! `dare update` — dry-run update plan (microplano 021); apply deferred to 022.

use std::path::PathBuf;

use dare_core::{CoreError, CoreResult, ProjectRoot};
use dare_project::find_project_root;
use dare_update::{
    format_human, load_desired_manifest_v2_embedded, parse_harness_target, plan_to_json,
    plan_update, UpdatePlanOptions,
};
use serde_json::Value;

/// Run `dare update`. Apply without `--dry-run` is stubbed until microplano 022.
pub fn run_update(
    dry_run: bool,
    target: Option<String>,
    dir: Option<PathBuf>,
) -> CoreResult<(String, Value)> {
    if !dry_run {
        return Err(CoreError::internal(
            "dare update apply is not implemented; use --dry-run (see microplano 022)",
        ));
    }

    let start =
        dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let Some(root_path) = find_project_root(&start) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&root_path)?;

    let harness = match target.as_deref() {
        Some(s) => Some(parse_harness_target(s)?),
        None => None,
    };

    let manifest = load_desired_manifest_v2_embedded()?;
    let plan = plan_update(
        &root,
        &manifest,
        &UpdatePlanOptions {
            target: harness,
            cli_version: env!("CARGO_PKG_VERSION").into(),
        },
    )?;

    let human = format_human(&plan);
    let data = plan_to_json(&plan)?;
    Ok((human, data))
}
