//! `GET /steering?file=`.

use axum::extract::{Query, State};
use axum::Json;
use dare_steering::SteeringShowReport;
use serde::Deserialize;

use crate::error::HttpError;
use crate::routes::map_service_error;
use crate::services::{steering_show, ServiceCtx};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SteeringQuery {
    pub file: Option<String>,
}

pub async fn steering(
    State(state): State<AppState>,
    Query(q): Query<SteeringQuery>,
) -> Result<Json<SteeringShowReport>, HttpError> {
    let file = q
        .file
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| HttpError::invalid_input("query param file is required"))?;
    let ctx = ServiceCtx::new((*state.root).clone());
    let report = steering_show(&ctx, file).map_err(map_service_error)?;
    Ok(Json(report))
}
