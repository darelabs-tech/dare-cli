//! `dare update` — plan (`--dry-run`) or apply (microplano 022).

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use dare_core::{CoreError, CoreResult, ProjectRoot};
use dare_project::find_project_root;
use dare_update::{
    apply_report_to_json, apply_update, format_apply_human, format_human,
    load_desired_manifest_v2_embedded, parse_harness_target, plan_to_json, plan_update,
    ApplyOptions, AskContext, AskFn, UpdatePlanOptions,
};
use serde_json::Value;

/// Run `dare update`: dry-run plans; otherwise applies with keep/replace policy.
pub fn run_update(
    dry_run: bool,
    yes: bool,
    force: bool,
    target: Option<String>,
    dir: Option<PathBuf>,
) -> CoreResult<(String, Value)> {
    let start =
        dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    if !start.exists() || !start.is_dir() {
        return Err(CoreError::not_found(format!(
            "directory not found: {}",
            start.display()
        )));
    }

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

    if dry_run {
        // `--force` is ignored on dry-run (zero writes).
        let human = format_human(&plan);
        let data = plan_to_json(&plan)?;
        return Ok((human, data));
    }

    let interactive = io::stdin().is_terminal() && io::stdout().is_terminal() && !yes && !force;
    let ask: Option<AskFn> = if interactive {
        Some(stdin_batch_ask())
    } else {
        None
    };

    let report = apply_update(
        &root,
        &plan,
        ApplyOptions {
            yes,
            force,
            interactive,
            ask,
            cli_version: env!("CARGO_PKG_VERSION").into(),
        },
    )?;

    let human = format_apply_human(&report);
    let data = apply_report_to_json(&report)?;
    Ok((human, data))
}

fn stdin_batch_ask() -> AskFn {
    Box::new(|ctx: &AskContext| {
        let n = ctx.customized_paths.len();
        let _ = write!(io::stderr(), "Replace all {n} customized files? [y/N]: ");
        let _ = io::stderr().flush();
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => false,
            Ok(_) => {
                let t = line.trim();
                t == "y" || t == "Y" || t == "yes"
            }
            Err(_) => false,
        }
    })
}
