//! `dare dag viz` — deterministic DAG visualization (microplano 027).

use std::path::PathBuf;
use std::process::ExitCode;

use clap::ValueEnum;
use dare_contracts::{load_dag, load_runtime_state};
use dare_core::fs::atomic_write;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use dare_dag::viz::{render, VizFormat, VizOptions};
use dare_dag::{DEFAULT_DAG_REL, STATE_REL};
use dare_project::find_project_root;
use serde_json::{json, Value};

use crate::commands::path_resolve::resolve_project_rel;
use crate::output::OutputRenderer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum CliVizFormat {
    Mermaid,
    Dot,
    Excalidraw,
}

impl Default for CliVizFormat {
    fn default() -> Self {
        Self::Mermaid
    }
}

impl From<CliVizFormat> for VizFormat {
    fn from(v: CliVizFormat) -> Self {
        match v {
            CliVizFormat::Mermaid => VizFormat::Mermaid,
            CliVizFormat::Dot => VizFormat::Dot,
            CliVizFormat::Excalidraw => VizFormat::Excalidraw,
        }
    }
}

pub fn run_dag_viz(
    dag: Option<PathBuf>,
    format: CliVizFormat,
    output: Option<PathBuf>,
    renderer: &OutputRenderer<'_>,
) -> ExitCode {
    match run_dag_viz_inner(dag, format, output) {
        Ok((human, data)) => {
            if let Err(e) = renderer.write_success(&human, data) {
                return ExitCode::from(renderer.write_error(&e) as u8);
            }
            ExitCode::SUCCESS
        }
        Err(e) => ExitCode::from(renderer.write_error(&e) as u8),
    }
}

fn run_dag_viz_inner(
    dag: Option<PathBuf>,
    format: CliVizFormat,
    output: Option<PathBuf>,
) -> CoreResult<(String, Value)> {
    let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
    let Some(root_path) = find_project_root(&cwd) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&root_path)?;
    let dag_rel = resolve_project_rel(&root, dag.as_deref(), DEFAULT_DAG_REL, true)?;
    let dag_display = dag_rel.as_str().to_string();

    let doc = load_dag(&root, &dag_rel)?;
    let state = SafeRelativePath::new(STATE_REL)
        .ok()
        .and_then(|rel| load_runtime_state(&root, &rel).ok());
    let opts = VizOptions {
        title_max: dare_dag::TITLE_MAX_DEFAULT,
        state,
    };
    let viz_format = VizFormat::from(format);
    let body = render(&doc, viz_format, &opts).map_err(CoreError::from)?;

    if let Some(out) = output {
        let out_rel = resolve_project_rel(&root, Some(out.as_path()), "", false)?;
        atomic_write(&root, &out_rel, body.as_bytes())?;
        let data = json!({
            "body": Value::Null,
            "dag": dag_display,
            "format": viz_format.as_str(),
            "outputPath": out_rel.as_str(),
        });
        let human = format!("wrote {}", out_rel.as_str());
        Ok((human, data))
    } else {
        let data = json!({
            "body": body,
            "dag": dag_display,
            "format": viz_format.as_str(),
            "outputPath": Value::Null,
        });
        Ok((body, data))
    }
}
