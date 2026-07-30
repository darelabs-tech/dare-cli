//! HTTP server domain crate (dashboard + REST) — microplano 051.

mod app;
mod auth;
mod config;
mod error;
mod middleware;
mod mode;
mod routes;
mod state;
mod telemetry;

pub use app::create_app;
pub use auth::{MSG_UNAUTHORIZED, auth_middleware};
pub use config::{
    parse_server_config_from_env, ServerConfig, TokenSource, DEFAULT_BODY_LIMIT,
    DEFAULT_DASHBOARD_BIND, DEFAULT_DASHBOARD_PORT, DEFAULT_REST_BIND, DEFAULT_REST_PORT,
    CSP_DASHBOARD, ENV_BIND, ENV_BODY_LIMIT, ENV_LOG_TOKEN, ENV_PORT, ENV_PROJECT, ENV_TOKEN,
};
pub use error::{HttpError, HttpErrorBody};
pub use middleware::{MSG_BODY_TOO_LARGE, cors_layer, security_headers_layers};
pub use mode::AppMode;
pub use routes::dashboard::MSG_PATH_ESCAPE;
pub use state::AppState;
pub use telemetry::build_telemetry_snapshot;

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use std::str::FromStr;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use dare_core::ProjectRoot;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    fn test_cfg_state(
        force_auth: bool,
        body_limit: usize,
    ) -> (ServerConfig, AppState, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let cfg = ServerConfig {
            bind: IpAddr::from_str("127.0.0.1").unwrap(),
            port: 4100,
            project_root: ProjectRoot::new(dir.path()).expect("root"),
            token: "test-token-ok".to_string(),
            token_source: TokenSource::Generated,
            body_limit,
            open_browser: false,
            log_token_value: false,
        };
        let mut state = AppState::new(
            root,
            cfg.token.clone(),
            body_limit,
            AppMode::Dashboard,
            "0.1.0-test",
        );
        state.force_auth = force_auth;
        (cfg, state, dir)
    }

    #[tokio::test]
    async fn health_ok() {
        let (cfg, state, _dir) = test_cfg_state(false, DEFAULT_BODY_LIMIT);
        let app = create_app(AppMode::Dashboard, &cfg, state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["protocol"], "rest");
        assert_eq!(json["mode"], "dashboard");
        assert_eq!(json["version"], "0.1.0-test");
    }

    #[tokio::test]
    async fn auth_force_require_401() {
        let (cfg, state, _dir) = test_cfg_state(true, DEFAULT_BODY_LIMIT);
        let app = create_app(AppMode::Dashboard, &cfg, state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], MSG_UNAUTHORIZED);
        assert_eq!(json["code"], "unauthorized");
    }

    #[tokio::test]
    async fn body_too_large_413() {
        let limit = 1024usize;
        let (cfg, state, _dir) = test_cfg_state(false, limit);
        let app = create_app(AppMode::Dashboard, &cfg, state);
        let oversized = vec![b'x'; limit + 1];
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/health")
                    .header("content-type", "application/octet-stream")
                    .header("content-length", oversized.len().to_string())
                    .body(Body::from(oversized))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], MSG_BODY_TOO_LARGE);
        assert_eq!(json["code"], "body_too_large");
    }

    #[tokio::test]
    async fn headers_present() {
        let (cfg, state, _dir) = test_cfg_state(false, DEFAULT_BODY_LIMIT);
        let app = create_app(AppMode::Dashboard, &cfg, state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert_eq!(
            headers.get("x-content-type-options").and_then(|v| v.to_str().ok()),
            Some("nosniff")
        );
        assert_eq!(
            headers.get("x-frame-options").and_then(|v| v.to_str().ok()),
            Some("DENY")
        );
        assert_eq!(
            headers
                .get("content-security-policy")
                .and_then(|v| v.to_str().ok()),
            Some(CSP_DASHBOARD)
        );
    }

    #[tokio::test]
    async fn dashboard_html_200() {
        let (cfg, state, _dir) = test_cfg_state(false, DEFAULT_BODY_LIMIT);
        let app = create_app(AppMode::Dashboard, &cfg, state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/dashboard")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let ct = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(ct.starts_with("text/html"), "content-type={ct}");
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8_lossy(&bytes);
        assert!(body.contains("<!DOCTYPE html>") || body.contains("<html"));
    }

    #[tokio::test]
    async fn assets_traversal_403() {
        let (cfg, state, _dir) = test_cfg_state(false, DEFAULT_BODY_LIMIT);
        let app = create_app(AppMode::Dashboard, &cfg, state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/assets/%2e%2e/Cargo.toml")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], MSG_PATH_ESCAPE);
        assert_eq!(json["code"], "path_escape");
    }

    #[tokio::test]
    async fn assets_exe_forbidden() {
        let (cfg, state, _dir) = test_cfg_state(false, DEFAULT_BODY_LIMIT);
        let app = create_app(AppMode::Dashboard, &cfg, state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/assets/evil.exe")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["code"], "forbidden");
    }

    #[tokio::test]
    async fn telemetry_maps_keys() {
        let (cfg, state, _dir) = test_cfg_state(false, DEFAULT_BODY_LIMIT);
        let app = create_app(AppMode::Rest, &cfg, state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/telemetry")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json.get("dag").is_some());
        assert!(json.get("gates").is_some());
        assert!(json.get("cost").is_some());
        assert!(json.get("bestOfN").is_some());
        assert!(json.get("guard").is_some());
        assert!(json.get("drift").is_some());
        assert!(json["dag"].get("tasksTotal").is_some());
        assert!(json["drift"].get("available").is_some());
    }
}
