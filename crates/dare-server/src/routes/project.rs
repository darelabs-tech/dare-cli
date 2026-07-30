//! `GET /project` — read-only project snapshot (no dare-cli dependency).

use axum::extract::State;
use axum::Json;

use crate::routes::map_service_error;
use crate::services::{project_snapshot, ProjectSnapshot, ServiceCtx};
use crate::state::AppState;

pub async fn project(State(state): State<AppState>) -> Result<Json<ProjectSnapshot>, crate::error::HttpError> {
    let ctx = ServiceCtx::new((*state.root).clone());
    let snap = project_snapshot(&ctx).map_err(map_service_error)?;
    Ok(Json(snap))
}
