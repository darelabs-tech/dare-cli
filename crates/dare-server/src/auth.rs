//! Bearer auth + loopback exemption rules.

use std::net::SocketAddr;

use axum::extract::{ConnectInfo, Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::error::HttpError;
use crate::state::AppState;

pub const MSG_UNAUTHORIZED: &str = "unauthorized";

/// Auth middleware: loopback exempt unless `force_auth`; non-loopback requires Bearer.
///
/// Oneshot without `ConnectInfo` is treated as loopback unless `force_auth`.
/// Present Bearer that does not match → 401 even on loopback.
pub async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, HttpError> {
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0);

    let is_loopback = match peer {
        Some(addr) => addr.ip().is_loopback(),
        None => !state.force_auth,
    };
    let require_auth = state.force_auth || !is_loopback;

    match extract_bearer(request.headers().get(axum::http::header::AUTHORIZATION)) {
        None => {
            if require_auth {
                return Err(HttpError::unauthorized(MSG_UNAUTHORIZED));
            }
        }
        Some(provided) => {
            if !constant_time_eq(provided.as_bytes(), state.token.as_bytes()) {
                return Err(HttpError::unauthorized(MSG_UNAUTHORIZED));
            }
        }
    }

    Ok(next.run(request).await)
}

fn extract_bearer(value: Option<&axum::http::HeaderValue>) -> Option<&str> {
    let raw = value?.to_str().ok()?;
    let token = raw.strip_prefix("Bearer ").or_else(|| raw.strip_prefix("bearer "))?;
    if token.is_empty() {
        return None;
    }
    Some(token)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
