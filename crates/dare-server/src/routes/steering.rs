//! `GET /steering?file=`.

use axum::extract::{Query, State};
use axum::Json;
use dare_steering::{show_steering, SteeringShowReport};
use serde::Deserialize;

use crate::error::HttpError;
use crate::http_map::map_core_error;
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
    let report = show_steering(state.root.as_ref(), file).map_err(map_core_error)?;
    Ok(Json(report))
}
