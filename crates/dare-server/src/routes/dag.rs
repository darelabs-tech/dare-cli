//! `GET /dag`.

use axum::extract::State;
use axum::Json;
use dare_contracts::{load_dag, DagDocument};
use dare_core::SafeRelativePath;
use serde_json::Value;

use crate::error::HttpError;
use crate::http_map::{map_core_error, DAG_REL};
use crate::state::AppState;

pub async fn dag(State(state): State<AppState>) -> Result<Json<Value>, HttpError> {
    let rel = SafeRelativePath::new(DAG_REL).map_err(map_core_error)?;
    let doc = load_dag(state.root.as_ref(), &rel).map_err(map_core_error)?;
    let value = match doc {
        DagDocument::V21(d) => serde_json::to_value(d),
        DagDocument::Legacy(d) => serde_json::to_value(d),
    }
    .map_err(|e| HttpError::internal(e.to_string()))?;
    Ok(Json(value))
}
