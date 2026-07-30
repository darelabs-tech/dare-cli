//! `GET /dag`.

use axum::extract::State;
use axum::Json;
use serde_json::Value;

use crate::error::HttpError;
use crate::routes::map_service_error;
use crate::services::{dag_load_json, ServiceCtx};
use crate::state::AppState;

pub async fn dag(State(state): State<AppState>) -> Result<Json<Value>, HttpError> {
    let ctx = ServiceCtx::new((*state.root).clone());
    let value = dag_load_json(&ctx).map_err(map_service_error)?;
    Ok(Json(value))
}
