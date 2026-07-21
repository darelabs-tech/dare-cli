//! `dare validate` — read-only DAG validation (microplano 020).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use dare_dag::{format_human, report_to_json, validate_path, ValidateOptions, DEFAULT_DAG_REL};
use dare_project::find_project_root;
use serde_json::Value;

use crate::output::OutputRenderer;

pub fn run_validate(dag: Option<PathBuf>, strict: bool, renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_validate_inner(dag, strict) {
        Ok((human, data, ok)) => {
            if let Err(e) = renderer.write_report(&human, data, ok) {
                return exit_err(renderer, &e);
            }
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => exit_err(renderer, &e),
    }
}

fn exit_err(renderer: &OutputRenderer<'_>, e: &CoreError) -> ExitCode {
    let code = renderer.write_error(e);
    ExitCode::from(code as u8)
}

fn run_validate_inner(dag: Option<PathBuf>, strict: bool) -> CoreResult<(String, Value, bool)> {
    let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
    let Some(root_path) = find_project_root(&cwd) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&root_path)?;
    let rel = resolve_dag_rel(&root, dag.as_deref())?;
    let opts = ValidateOptions { strict };
    let report = validate_path(&root, &rel, &opts)?;
    let human = format_human(&report);
    let data = report_to_json(&report)?;
    Ok((human, data, report.ok))
}

fn resolve_dag_rel(root: &ProjectRoot, dag: Option<&Path>) -> CoreResult<SafeRelativePath> {
    let Some(dag) = dag else {
        return SafeRelativePath::new(DEFAULT_DAG_REL);
    };
    if dag.is_absolute() {
        let dag_canon = if dag.exists() {
            std::fs::canonicalize(dag).map_err(|e| CoreError::io(e.to_string()))?
        } else {
            dag.to_path_buf()
        };
        let root_std = root.as_path().as_std_path();
        let rel = dag_canon
            .strip_prefix(root_std)
            .map_err(|_| CoreError::invalid_input("dag path is outside project root"))?;
        let s = rel.to_string_lossy().replace('\\', "/");
        if s.is_empty() {
            return Err(CoreError::invalid_input("invalid dag path"));
        }
        // existence check deferred to load_dag
        if !dag.exists() {
            return Err(CoreError::not_found(format!(
                "dag file not found: {}",
                dag.display()
            )));
        }
        return SafeRelativePath::new(&s);
    }
    let joined = root.as_path().as_std_path().join(dag);
    if !joined.exists() {
        return Err(CoreError::not_found(format!(
            "dag file not found: {}",
            dag.display()
        )));
    }
    let s = dag.to_string_lossy().replace('\\', "/");
    SafeRelativePath::new(&s)
}
