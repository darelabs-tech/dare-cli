//! Shared Axum router factory.

use axum::routing::get;
use axum::Router;
use tower_http::limit::RequestBodyLimitLayer;

use crate::auth::auth_middleware;
use crate::config::ServerConfig;
use crate::middleware::{cors_layer, map_body_too_large, security_headers_layers};
use crate::mode::AppMode;
use crate::routes;
use crate::state::AppState;

/// Build the shared app router for the given mode (Fase A: `GET /health` only).
pub fn create_app(mode: AppMode, cfg: &ServerConfig, mut state: AppState) -> Router {
    state.mode = mode;
    let body_limit = cfg.body_limit;
    let (nosniff, frame, csp) = security_headers_layers();

    Router::new()
        .route("/health", get(routes::health))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(nosniff)
        .layer(frame)
        .layer(csp)
        .layer(cors_layer())
        .layer(RequestBodyLimitLayer::new(body_limit))
        // Outer: rewrite empty 413 from RequestBodyLimitLayer into JSON.
        .layer(axum::middleware::from_fn(map_body_too_large))
        .with_state(state)
}
