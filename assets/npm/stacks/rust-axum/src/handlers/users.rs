//! Users handlers — GET /users, POST /users.
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    errors::AppError,
    middleware::auth::extract_bearer,
    services::user_service,
    SharedState,
};

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 { 1 }
fn default_limit() -> i64 { 20 }

#[derive(Debug, Serialize)]
pub struct UserOut {
    pub id: uuid::Uuid,
    pub email: String,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserPage {
    pub items: Vec<UserOut>,
    pub total: i64,
    pub page: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    #[serde(default = "default_role")]
    pub role: String,
}

fn default_role() -> String { "USER".to_string() }

pub async fn list(
    State(state): State<SharedState>,
    Query(q): Query<PageQuery>,
    headers: HeaderMap,
) -> Result<Json<UserPage>, AppError> {
    let _ = extract_bearer(&state.cfg, &headers)?;
    let page = q.page.max(1);
    let limit = q.limit.clamp(1, 100);
    let (items, total) = user_service::page(&state, page, limit).await?;
    Ok(Json(UserPage {
        items: items.into_iter().map(|u| UserOut {
            id: u.id, email: u.email, role: u.role, created_at: u.created_at,
        }).collect(),
        total,
        page,
    }))
}

pub async fn create(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<UserOut>, AppError> {
    let claims = extract_bearer(&state.cfg, &headers)?;
    if claims.role != "ADMIN" {
        return Err(AppError::Forbidden);
    }
    let user = user_service::create(&state, &body.email, &body.password, &body.role).await?;
    Ok(Json(UserOut { id: user.id, email: user.email, role: user.role, created_at: user.created_at }))
}
