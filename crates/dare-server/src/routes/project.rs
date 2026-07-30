//! `GET /project` — read-only project snapshot (no dare-cli dependency).

use axum::extract::State;
use axum::Json;
use dare_core::SafeRelativePath;
use serde::Serialize;
use serde_json::Value;

use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshot {
    pub schema_version: u32,
    pub root: String,
    pub dare_dir_present: bool,
    pub config_present: bool,
    pub backend: Option<String>,
    pub graph_present: bool,
}

fn read_backend(root: &dare_core::ProjectRoot) -> Option<String> {
    let rel = SafeRelativePath::new("dare.config.json").ok()?;
    let abs = root.resolve(&rel).ok()?;
    let path = abs.as_path().as_std_path();
    if !path.is_file() {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let v: Value = serde_json::from_slice(&bytes).ok()?;
    v.get("ide")
        .or_else(|| v.get("backend"))
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
}

pub async fn project(State(state): State<AppState>) -> Json<ProjectSnapshot> {
    let root_path = state.root.as_path().as_std_path();
    let dare_dir_present = root_path.join("DARE").is_dir();
    let config_present = root_path.join("dare.config.json").is_file();
    let mut graph_present = root_path.join("dare-graph.yml").is_file();
    if !graph_present {
        graph_present = root_path.join("DARE").join("dare-graph.yml").is_file();
    }
    let backend = read_backend(state.root.as_ref());
    Json(ProjectSnapshot {
        schema_version: 1,
        root: state.root.as_path().as_str().to_string(),
        dare_dir_present,
        config_present,
        backend,
        graph_present,
    })
}
