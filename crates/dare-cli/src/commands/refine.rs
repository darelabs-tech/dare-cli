//! `dare refine` — complexity score + optional sub-DAG splice (microplano 033).

use std::process::ExitCode;

use dare_core::{CoreError, CoreResult, ProjectRoot};
use dare_dag::{
    format_refine_human, refine_report_to_json, run_refine_default, strict_should_exit_2,
    RefineOptions, DEFAULT_DAG_REL, MSG_STRICT,
};
use dare_project::find_project_root;
use serde_json::Value;

use crate::output::OutputRenderer;

pub struct RefineCliArgs {
    pub task_id: String,
    pub split: bool,
    pub apply: bool,
    pub strict: bool,
    pub format: String,
}

pub fn run_refine_cmd(args: RefineCliArgs, renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_refine_inner(args) {
        Ok((human, data, exit2)) => {
            if let Err(e) = renderer.write_report(&human, data, !exit2) {
                return exit_err(renderer, &e);
            }
            if exit2 {
                ExitCode::from(2)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => exit_err(renderer, &e),
    }
}

fn exit_err(renderer: &OutputRenderer<'_>, e: &CoreError) -> ExitCode {
    let code = renderer.write_error(e);
    ExitCode::from(code as u8)
}

fn run_refine_inner(args: RefineCliArgs) -> CoreResult<(String, Value, bool)> {
    let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
    let Some(root_path) = find_project_root(&cwd) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&root_path)?;

    let format = args.format.to_ascii_lowercase();
    if format != "human" && format != "json" {
        return Err(CoreError::invalid_input(
            "invalid --format (expected human|json)",
        ));
    }

    let opts = RefineOptions {
        task_id: args.task_id,
        split: args.split || args.apply,
        apply: args.apply,
        strict: args.strict,
        dag_rel: DEFAULT_DAG_REL.to_string(),
    };

    let report = run_refine_default(&root, &opts)?;
    let exit2 = strict_should_exit_2(&report, args.strict);
    let data = refine_report_to_json(&report)?;

    let mut human = match format.as_str() {
        "json" => dare_core::to_canonical_json_string(&data)? + "\n",
        _ => format_refine_human(&report),
    };
    if exit2 {
        human.push_str(MSG_STRICT);
        human.push('\n');
    }

    Ok((human, data, exit2))
}
