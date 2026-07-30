//! `GET /health` smoke endpoint.

use axum::extract::State;
use axum::Json;
use serde::Serialize;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub version: String,
    pub protocol: &'static str,
    pub mode: &'static str,
}

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        version: state.version.clone(),
        protocol: "rest",
        mode: state.mode.as_str(),
    })
}
