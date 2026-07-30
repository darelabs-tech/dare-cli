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

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message, "invalid_input")
    }

    pub fn path_escape(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message, "path_escape")
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message, "forbidden")
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message, "not_found")
    }

    pub fn graph_unavailable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, message, "graph_unavailable")
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message, "internal")
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}
