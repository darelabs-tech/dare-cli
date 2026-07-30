//! Graph POST routes: locate / traverse / map-requirement.

use axum::extract::State;
use axum::Json;
use dare_graph::{
    bfs_expand, load_graph_config, locate, open_graph, LocateOptions, NodeType, RankedHit,
    DEFAULT_FANOUT, DEFAULT_LIMIT, DEFAULT_MAX_HOPS, LOCATE_DECAY,
};
use serde::{Deserialize, Serialize};

use crate::error::HttpError;
use crate::http_map::{map_core_error, MSG_GRAPH_DISABLED};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocateBody {
    pub query: String,
    pub max_hops: Option<usize>,
    pub fanout: Option<usize>,
    pub limit: Option<usize>,
    pub decay: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraverseBody {
    pub seeds: Vec<String>,
    pub max_hops: Option<usize>,
    pub fanout: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocateResponse {
    pub schema_version: u32,
    pub hits: Vec<RankedHit>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraverseResponse {
    pub schema_version: u32,
    pub nodes: Vec<String>,
}

fn open_project_graph(root: &dare_core::ProjectRoot) -> Result<dare_graph::GraphHandle, HttpError> {
    let cfg = load_graph_config(root, None).map_err(|_| {
        HttpError::graph_unavailable(MSG_GRAPH_DISABLED)
    })?;
    open_graph(root, &cfg).map_err(|_| HttpError::graph_unavailable(MSG_GRAPH_DISABLED))
}

fn locate_opts(body: LocateBody) -> Result<LocateOptions, HttpError> {
    let query = body.query.trim().to_string();
    if query.is_empty() {
        return Err(HttpError::invalid_input("query must not be empty"));
    }
    Ok(LocateOptions {
        query,
        max_hops: body.max_hops.unwrap_or(DEFAULT_MAX_HOPS),
        fanout: body.fanout.unwrap_or(DEFAULT_FANOUT),
        limit: body.limit.unwrap_or(DEFAULT_LIMIT),
        decay: body.decay.unwrap_or(LOCATE_DECAY),
    })
}

pub async fn graph_locate(
    State(state): State<AppState>,
    body: Result<Json<LocateBody>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<LocateResponse>, HttpError> {
    let Json(body) = body.map_err(|e| HttpError::invalid_input(e.to_string()))?;
    let opts = locate_opts(body)?;
    let g = open_project_graph(state.root.as_ref())?;
    let hits = locate(&g, &opts).map_err(map_core_error)?;
    Ok(Json(LocateResponse {
        schema_version: 1,
        hits,
    }))
}

pub async fn graph_traverse(
    State(state): State<AppState>,
    body: Result<Json<TraverseBody>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<TraverseResponse>, HttpError> {
    let Json(body) = body.map_err(|e| HttpError::invalid_input(e.to_string()))?;
    if body.seeds.is_empty() || body.seeds.len() > 32 {
        return Err(HttpError::invalid_input(
            "seeds must contain 1..=32 entries",
        ));
    }
    for s in &body.seeds {
        let t = s.trim();
        if t.is_empty() || t.len() > 256 {
            return Err(HttpError::invalid_input(
                "each seed must be non-empty and <= 256 chars",
            ));
        }
    }
    let g = open_project_graph(state.root.as_ref())?;
    let nodes = bfs_expand(
        &g,
        &body.seeds,
        body.max_hops.unwrap_or(DEFAULT_MAX_HOPS),
        body.fanout.unwrap_or(DEFAULT_FANOUT),
    )
    .map_err(map_core_error)?;
    Ok(Json(TraverseResponse {
        schema_version: 1,
        nodes,
    }))
}

pub async fn graph_map_requirement(
    State(state): State<AppState>,
    body: Result<Json<LocateBody>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<LocateResponse>, HttpError> {
    let Json(body) = body.map_err(|e| HttpError::invalid_input(e.to_string()))?;
    let opts = locate_opts(body)?;
    let g = open_project_graph(state.root.as_ref())?;
    let all = locate(&g, &opts).map_err(map_core_error)?;
    let req = NodeType::Requirement.as_str();
    let filtered: Vec<RankedHit> = all
        .iter()
        .filter(|h| h.node_type == req)
        .cloned()
        .collect();
    let hits = if filtered.is_empty() { all } else { filtered };
    Ok(Json(LocateResponse {
        schema_version: 1,
        hits,
    }))
}
