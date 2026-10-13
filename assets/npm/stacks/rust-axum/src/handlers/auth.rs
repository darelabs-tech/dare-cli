//! Auth handlers — POST /auth/login, GET /auth/me.
use axum::{extract::State, http::HeaderMap, Json};
use serde::{Deserialize, Serialize};

use crate::{
    errors::AppError,
    middleware::auth::extract_bearer,
    services::auth_service,
    services::user_service,
    SharedState,
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub role: String,
}

pub async fn login(
    State(state): State<SharedState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let (token, expires_in) = auth_service::login(&state, &body.email, &body.password).await?;
    Ok(Json(LoginResponse { access_token: token, token_type: "Bearer", expires_in }))
}

pub async fn me(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<MeResponse>, AppError> {
    let claims = extract_bearer(&state.cfg, &headers)?;
    let user = user_service::find_by_id(&state, claims.sub).await?;
    Ok(Json(MeResponse { id: user.id, email: user.email, role: user.role }))
}
