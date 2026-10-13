//! RFC 7807 Problem Details — uniform error shape across the API.
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid credentials")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal: {0}")]
    Internal(#[from] anyhow::Error),
    #[error("database: {0}")]
    Db(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, title) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not Found"),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "Bad Request"),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "Conflict"),
            AppError::Internal(_) | AppError::Db(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
        };
        let body = json!({
            "type": format!("urn:problem:{}", status.as_u16()),
            "title": title,
            "status": status.as_u16(),
            "detail": self.to_string(),
        });
        (status, Json(body)).into_response()
    }
}
