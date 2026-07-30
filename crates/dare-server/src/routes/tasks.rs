//! `GET|PUT /tasks/{id}`.

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;

use crate::error::HttpError;
use crate::routes::map_service_error;
use crate::services::{task_get, task_put, ServiceCtx};
use crate::state::AppState;
use crate::tasks_md::TaskView;

#[derive(Debug, Deserialize)]
pub struct PutTaskBody {
    pub status: String,
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TaskView>, HttpError> {
    let ctx = ServiceCtx::new((*state.root).clone());
    let view = task_get(&ctx, &id).map_err(map_service_error)?;
    Ok(Json(view))
}

pub async fn put_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    body: Result<Json<PutTaskBody>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<TaskView>, HttpError> {
    let Json(body) = body.map_err(|e| HttpError::invalid_input(e.to_string()))?;
    let ctx = ServiceCtx::new((*state.root).clone());
    let view = task_put(&ctx, &id, &body.status).map_err(map_service_error)?;
    Ok(Json(view))
}
