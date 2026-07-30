//! `GET /blueprint`.

use axum::extract::State;
use axum::Json;
use dare_core::SafeRelativePath;
use serde::Serialize;

use crate::error::HttpError;
use crate::http_map::{map_core_error, BLUEPRINT_MAX_BYTES, BLUEPRINT_REL};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct BlueprintResponse {
    pub path: String,
    pub content: String,
    pub bytes: usize,
}

pub async fn blueprint(
    State(state): State<AppState>,
) -> Result<Json<BlueprintResponse>, HttpError> {
    let rel = SafeRelativePath::new(BLUEPRINT_REL).map_err(map_core_error)?;
    let abs = state.root.resolve(&rel).map_err(map_core_error)?;
    let path = abs.as_path().as_std_path();
    if !path.is_file() {
        return Err(HttpError::not_found(format!(
            "file not found: {BLUEPRINT_REL}"
        )));
    }
    let meta = std::fs::metadata(path).map_err(|e| HttpError::internal(e.to_string()))?;
    if meta.len() > BLUEPRINT_MAX_BYTES {
        return Err(HttpError::invalid_input("blueprint exceeds size limit"));
    }
    let content =
        std::fs::read_to_string(path).map_err(|e| HttpError::internal(e.to_string()))?;
    let bytes = content.len();
    Ok(Json(BlueprintResponse {
        path: BLUEPRINT_REL.to_string(),
        content,
        bytes,
    }))
}
