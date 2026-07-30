//! `GET /blueprint`.

use axum::extract::State;
use axum::Json;

use crate::error::HttpError;
use crate::routes::map_service_error;
use crate::services::{read_blueprint, BlueprintDoc, ServiceCtx};
use crate::state::AppState;

pub async fn blueprint(State(state): State<AppState>) -> Result<Json<BlueprintDoc>, HttpError> {
    let ctx = ServiceCtx::new((*state.root).clone());
    let doc = read_blueprint(&ctx).map_err(map_service_error)?;
    Ok(Json(doc))
}
