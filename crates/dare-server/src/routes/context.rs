//! `POST /context/query`.

use axum::extract::State;
use axum::Json;
use serde::Deserialize;

use crate::error::HttpError;
use crate::routes::map_service_error;
use crate::services::{context_query as svc_context_query, ContextQueryResponse, ServiceCtx};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ContextQueryRequest {
    #[serde(rename = "type")]
    pub kind: String,
    pub query: String,
}

pub async fn context_query(
    State(state): State<AppState>,
    body: Result<Json<ContextQueryRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<ContextQueryResponse>, HttpError> {
    let Json(req) = body.map_err(|e| HttpError::invalid_input(e.to_string()))?;
    let ctx = ServiceCtx::new((*state.root).clone());
    let resp = svc_context_query(&ctx, &req.kind, &req.query).map_err(map_service_error)?;
    Ok(Json(resp))
}
