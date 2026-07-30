//! `POST /context/query`.

use axum::extract::State;
use axum::Json;
use dare_core::fs::read_to_string;
use dare_core::SafeRelativePath;
use dare_graph::{load_graph_config, locate, open_graph, LocateOptions, LOCATE_DECAY};
use serde::{Deserialize, Serialize};

use crate::error::HttpError;
use crate::http_map::{
    map_core_error, BLUEPRINT_REL, DAG_REL, MSG_GRAPH_DISABLED, MSG_INVALID_CONTEXT_TYPE,
};
use crate::state::AppState;
use crate::tasks_md::TASKS_REL;

#[derive(Debug, Deserialize)]
pub struct ContextQueryRequest {
    #[serde(rename = "type")]
    pub kind: String,
    pub query: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextQueryResponse {
    pub schema_version: u32,
    #[serde(rename = "type")]
    pub kind: String,
    pub query: String,
    pub hits: Vec<ContextHit>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextHit {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub snippet: String,
}

fn clip_snippet(s: &str) -> String {
    let t = s.trim();
    if t.chars().count() <= 280 {
        t.to_string()
    } else {
        t.chars().take(280).collect()
    }
}

pub async fn context_query(
    State(state): State<AppState>,
    body: Result<Json<ContextQueryRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<ContextQueryResponse>, HttpError> {
    let Json(req) = body.map_err(|e| HttpError::invalid_input(e.to_string()))?;
    let kind = req.kind.trim();
    if !matches!(kind, "architecture" | "task" | "dependency") {
        return Err(HttpError::invalid_input(MSG_INVALID_CONTEXT_TYPE));
    }
    let query = req.query.trim().to_string();
    if query.is_empty() || query.chars().count() > 512 {
        return Err(HttpError::invalid_input(
            "query must be 1..=512 characters after trim",
        ));
    }

    let root = state.root.as_ref();
    let mut warnings = Vec::new();
    let hits = match kind {
        "architecture" => search_blueprint(root, &query)?,
        "task" => search_tasks(root, &query)?,
        "dependency" => search_dependency(root, &query, &mut warnings)?,
        _ => unreachable!(),
    };

    Ok(Json(ContextQueryResponse {
        schema_version: 1,
        kind: kind.to_string(),
        query,
        hits,
        warnings,
    }))
}

fn search_blueprint(
    root: &dare_core::ProjectRoot,
    query: &str,
) -> Result<Vec<ContextHit>, HttpError> {
    let rel = SafeRelativePath::new(BLUEPRINT_REL).map_err(map_core_error)?;
    let text = match read_to_string(root, &rel) {
        Ok(t) => t,
        Err(dare_core::CoreError::NotFound(_)) => return Ok(Vec::new()),
        Err(e) => return Err(map_core_error(e)),
    };
    let q = query.to_ascii_lowercase();
    let mut hits = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.to_ascii_lowercase().contains(&q) {
            hits.push(ContextHit {
                id: format!("blueprint#{}", i + 1),
                label: line.trim().chars().take(80).collect(),
                kind: "architecture".to_string(),
                snippet: clip_snippet(line),
            });
        }
    }
    Ok(hits)
}

fn search_tasks(
    root: &dare_core::ProjectRoot,
    query: &str,
) -> Result<Vec<ContextHit>, HttpError> {
    let rel = SafeRelativePath::new(TASKS_REL).map_err(map_core_error)?;
    let text = match read_to_string(root, &rel) {
        Ok(t) => t,
        Err(dare_core::CoreError::NotFound(_)) => return Ok(Vec::new()),
        Err(e) => return Err(map_core_error(e)),
    };
    let q = query.to_ascii_lowercase();
    let mut hits = Vec::new();
    for line in text.lines() {
        if line.to_ascii_lowercase().contains(&q) {
            let id = line
                .split('|')
                .nth(1)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("task")
                .to_string();
            hits.push(ContextHit {
                id: id.clone(),
                label: id,
                kind: "task".to_string(),
                snippet: clip_snippet(line),
            });
        }
    }
    Ok(hits)
}

fn search_dependency(
    root: &dare_core::ProjectRoot,
    query: &str,
    warnings: &mut Vec<String>,
) -> Result<Vec<ContextHit>, HttpError> {
    match try_graph_locate(root, query) {
        Ok(hits) => Ok(hits),
        Err(_) => {
            warnings.push(MSG_GRAPH_DISABLED.to_string());
            search_dag_depends(root, query)
        }
    }
}

fn try_graph_locate(
    root: &dare_core::ProjectRoot,
    query: &str,
) -> Result<Vec<ContextHit>, HttpError> {
    let cfg = load_graph_config(root, None).map_err(|_| {
        HttpError::graph_unavailable(MSG_GRAPH_DISABLED)
    })?;
    let g = open_graph(root, &cfg).map_err(|_| {
        HttpError::graph_unavailable(MSG_GRAPH_DISABLED)
    })?;
    let opts = LocateOptions {
        query: query.to_string(),
        decay: LOCATE_DECAY,
        ..LocateOptions::default()
    };
    let ranked = locate(&g, &opts).map_err(map_core_error)?;
    Ok(ranked
        .into_iter()
        .map(|h| ContextHit {
            id: h.id,
            label: h.label,
            kind: h.node_type,
            snippet: String::new(),
        })
        .collect())
}

fn search_dag_depends(
    root: &dare_core::ProjectRoot,
    query: &str,
) -> Result<Vec<ContextHit>, HttpError> {
    let rel = SafeRelativePath::new(DAG_REL).map_err(map_core_error)?;
    let text = match read_to_string(root, &rel) {
        Ok(t) => t,
        Err(dare_core::CoreError::NotFound(_)) => return Ok(Vec::new()),
        Err(e) => return Err(map_core_error(e)),
    };
    let q = query.to_ascii_lowercase();
    let mut hits = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let lower = line.to_ascii_lowercase();
        if lower.contains(&q) {
            hits.push(ContextHit {
                id: format!("dag#line{}", i + 1),
                label: clip_snippet(line),
                kind: "dependency".to_string(),
                snippet: clip_snippet(line),
            });
        }
    }
    Ok(hits)
}
