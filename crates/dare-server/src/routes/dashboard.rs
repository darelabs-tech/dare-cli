//! Dashboard HTML, static assets (rust-embed), and telemetry JSON.

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use rust_embed::Embed;

use crate::config::CSP_DASHBOARD;
use crate::error::HttpError;
use crate::state::AppState;
use crate::telemetry::build_telemetry_snapshot;

pub const MSG_PATH_ESCAPE: &str = "path escape forbidden";

#[derive(Embed)]
#[folder = "assets"]
struct DashboardAssets;

/// Read-only dashboard routes (mounted in Dashboard and Rest modes).
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(dashboard_html))
        .route("/assets/{*path}", get(serve_asset))
        .route("/api/telemetry", get(api_telemetry))
}

async fn dashboard_html() -> Response {
    match DashboardAssets::get("dashboard/index.html") {
        Some(file) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            );
            headers.insert(
                header::CONTENT_SECURITY_POLICY,
                HeaderValue::from_static(CSP_DASHBOARD),
            );
            headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
            headers.insert(
                header::X_CONTENT_TYPE_OPTIONS,
                HeaderValue::from_static("nosniff"),
            );
            (StatusCode::OK, headers, file.data.to_vec()).into_response()
        }
        None => HttpError::new(
            StatusCode::NOT_FOUND,
            "dashboard not found",
            "not_found",
        )
        .into_response(),
    }
}

async fn serve_asset(Path(path): Path<String>) -> Response {
    if let Err(err) = validate_asset_path(&path) {
        return err.into_response();
    }

    let embed_key = format!("dashboard/{path}");
    match DashboardAssets::get(&embed_key) {
        Some(file) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(content_type_for(&path)),
            );
            (StatusCode::OK, headers, file.data.to_vec()).into_response()
        }
        None => HttpError::new(StatusCode::NOT_FOUND, "asset not found", "not_found")
            .into_response(),
    }
}

async fn api_telemetry(State(state): State<AppState>) -> Result<Json<dare_contracts::TelemetrySnapshot>, HttpError> {
    build_telemetry_snapshot(state.root.as_ref()).map(Json).map_err(|e| {
        HttpError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            e.to_string(),
            "internal",
        )
    })
}

fn validate_asset_path(path: &str) -> Result<(), HttpError> {
    if path.contains("..") || path.starts_with('/') || path.contains('\\') {
        return Err(HttpError::new(
            StatusCode::FORBIDDEN,
            MSG_PATH_ESCAPE,
            "path_escape",
        ));
    }
    if path.is_empty() || path.split('/').any(|s| s.is_empty()) {
        return Err(HttpError::new(
            StatusCode::FORBIDDEN,
            MSG_PATH_ESCAPE,
            "path_escape",
        ));
    }
    let ext = path
        .rsplit_once('.')
        .map(|(_, e)| e)
        .unwrap_or("");
    if !matches!(
        ext,
        "js" | "css" | "svg" | "png" | "ico" | "woff2" | "map"
    ) {
        return Err(HttpError::new(
            StatusCode::FORBIDDEN,
            "forbidden",
            "forbidden",
        ));
    }
    Ok(())
}

fn content_type_for(path: &str) -> &'static str {
    match path.rsplit_once('.').map(|(_, e)| e) {
        Some("js") => "application/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("map") => "application/json",
        _ => "application/octet-stream",
    }
}
