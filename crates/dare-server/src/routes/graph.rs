//! Graph POST routes: locate / traverse / map-requirement.

use axum::extract::State;
use axum::Json;
use dare_graph::RankedHit;
use serde::{Deserialize, Serialize};

use crate::error::HttpError;
use crate::routes::map_service_error;
use crate::services::{
    graph_locate as svc_locate, graph_map_requirement as svc_map,
    graph_traverse as svc_traverse, locate_defaults, ServiceCtx,
};
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

pub async fn graph_locate(
    State(state): State<AppState>,
    body: Result<Json<LocateBody>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<LocateResponse>, HttpError> {
    let Json(body) = body.map_err(|e| HttpError::invalid_input(e.to_string()))?;
    let opts = locate_defaults(
        body.query.trim().to_string(),
        body.max_hops,
        body.fanout,
        body.limit,
        body.decay,
    );
    let ctx = ServiceCtx::new((*state.root).clone());
    let hits = svc_locate(&ctx, opts).map_err(map_service_error)?;
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
    let ctx = ServiceCtx::new((*state.root).clone());
    let nodes = svc_traverse(
        &ctx,
        &body.seeds,
        body.max_hops
            .unwrap_or(dare_graph::DEFAULT_MAX_HOPS),
        body.fanout.unwrap_or(dare_graph::DEFAULT_FANOUT),
    )
    .map_err(map_service_error)?;
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
    let opts = locate_defaults(
        body.query.trim().to_string(),
        body.max_hops,
        body.fanout,
        body.limit,
        body.decay,
    );
    let ctx = ServiceCtx::new((*state.root).clone());
    let hits = svc_map(&ctx, opts).map_err(map_service_error)?;
    Ok(Json(LocateResponse {
        schema_version: 1,
        hits,
    }))
}
