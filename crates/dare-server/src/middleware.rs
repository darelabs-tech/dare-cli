//! Security headers, CORS, and body-limit error mapping.

use axum::extract::Request;
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::config::CSP_DASHBOARD;
use crate::error::HttpErrorBody;

pub const MSG_BODY_TOO_LARGE: &str = "request body too large";

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            let Ok(s) = origin.to_str() else {
                return false;
            };
            s.starts_with("http://127.0.0.1:")
                || s.starts_with("http://localhost:")
                || s.starts_with("http://[::1]:")
        }))
        .allow_methods(Any)
        .allow_headers(Any)
}

pub fn security_headers_layers() -> (
    SetResponseHeaderLayer<HeaderValue>,
    SetResponseHeaderLayer<HeaderValue>,
    SetResponseHeaderLayer<HeaderValue>,
) {
    (
        SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ),
        SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ),
        SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(CSP_DASHBOARD),
        ),
    )
}

/// Map empty/generic 413 from `RequestBodyLimitLayer` to JSON `{error,code}`.
pub async fn map_body_too_large(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    if response.status() == StatusCode::PAYLOAD_TOO_LARGE {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(HttpErrorBody {
                error: MSG_BODY_TOO_LARGE.to_string(),
                code: "body_too_large".to_string(),
            }),
        )
            .into_response();
    }
    response
}
