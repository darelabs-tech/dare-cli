//! `GET|PUT /tasks/{id}`.

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;

use crate::error::HttpError;
use crate::http_map::map_core_error;
use crate::state::AppState;
use crate::tasks_md::{
    get_task_view, put_task_status, reject_path_escape_id, validate_task_id, TaskView,
    MSG_PATH_ESCAPE,
};

#[derive(Debug, Deserialize)]
pub struct PutTaskBody {
    pub status: String,
}

fn check_id(id: &str) -> Result<(), HttpError> {
    if let Err(e) = reject_path_escape_id(id) {
        let _ = e;
        return Err(HttpError::path_escape(MSG_PATH_ESCAPE));
    }
    validate_task_id(id).map_err(map_core_error)
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TaskView>, HttpError> {
    check_id(&id)?;
    let view = get_task_view(state.root.as_ref(), &id).map_err(map_core_error)?;
    Ok(Json(view))
}

pub async fn put_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    body: Result<Json<PutTaskBody>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<TaskView>, HttpError> {
    check_id(&id)?;
    let Json(body) = body.map_err(|e| HttpError::invalid_input(e.to_string()))?;
    let view = put_task_status(state.root.as_ref(), &id, body.status.trim()).map_err(map_core_error)?;
    Ok(Json(view))
}
