//! JSON HTTP errors `{error, code}`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HttpErrorBody {
    pub error: String,
    pub code: String,
}

#[derive(Debug, Clone)]
pub struct HttpError {
    pub status: StatusCode,
    pub body: HttpErrorBody,
}

impl HttpError {
    pub fn new(status: StatusCode, error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            status,
            body: HttpErrorBody {
                error: error.into(),
                code: code.into(),
            },
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message, "unauthorized")
    }

    pub fn body_too_large(message: impl Into<String>) -> Self {
        Self::new(StatusCode::PAYLOAD_TOO_LARGE, message, "body_too_large")
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}
