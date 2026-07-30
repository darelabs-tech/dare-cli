//! HTTP server domain crate (dashboard + REST) — microplano 051.
//! MCP transport (optional feature `mcp`) — microplano 052.

mod app;
mod auth;
mod browser;
mod config;
mod error;
mod http_map;
mod middleware;
mod mode;
mod routes;
mod serve;
mod services;
mod state;
mod tasks_md;
mod telemetry;

#[cfg(feature = "mcp")]
pub mod mcp;

pub use app::create_app;
pub use auth::{MSG_UNAUTHORIZED, auth_middleware};
pub use browser::{is_allowed_browser_url, open_browser};
pub use config::{
    parse_server_config_from_env, ServerConfig, TokenSource, DEFAULT_BODY_LIMIT,
    DEFAULT_DASHBOARD_BIND, DEFAULT_DASHBOARD_PORT, DEFAULT_REST_BIND, DEFAULT_REST_PORT,
    CSP_DASHBOARD, ENV_BIND, ENV_BODY_LIMIT, ENV_LOG_TOKEN, ENV_PORT, ENV_PROJECT, ENV_TOKEN,
};
pub use error::{HttpError, HttpErrorBody};
pub use http_map::{MSG_GRAPH_DISABLED, MSG_INVALID_CONTEXT_TYPE};
pub use middleware::{MSG_BODY_TOO_LARGE, cors_layer, security_headers_layers};
pub use mode::AppMode;
pub use routes::rest_router;
pub use serve::serve;
pub use state::AppState;
pub use tasks_md::{
    get_task_view, put_task_status, TaskView, MSG_INVALID_STATUS, MSG_PATH_ESCAPE,
    MSG_TASK_NOT_FOUND, TASKS_REL,
};
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

    fn rest_app(dir: &tempfile::TempDir) -> (ServerConfig, axum::Router) {
        let root = ProjectRoot::new(dir.path()).expect("root");
        let cfg = ServerConfig {
            bind: IpAddr::from_str("127.0.0.1").unwrap(),
            port: 3000,
            project_root: ProjectRoot::new(dir.path()).expect("root"),
            token: "test-token-ok".to_string(),
            token_source: TokenSource::Generated,
            body_limit: DEFAULT_BODY_LIMIT,
            open_browser: false,
            log_token_value: false,
        };
        let state = AppState::new(
            root,
            cfg.token.clone(),
            DEFAULT_BODY_LIMIT,
            AppMode::Rest,
            "0.1.0-test",
        );
        let app = create_app(AppMode::Rest, &cfg, state);
        (cfg, app)
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
            headers
                .get("x-content-type-options")
                .and_then(|v| v.to_str().ok()),
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

    #[tokio::test]
    async fn tools_order() {
        let dir = tempfile::tempdir().unwrap();
        let (_cfg, app) = rest_app(&dir);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tools")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let names: Vec<&str> = json["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            vec![
                "health",
                "tools",
                "context_query",
                "blueprint",
                "dag",
                "tasks_get",
                "tasks_put",
                "graph_locate",
                "graph_map_requirement",
                "graph_traverse",
                "project",
                "steering",
            ]
        );
    }

    #[tokio::test]
    async fn context_bad_type() {
        let dir = tempfile::tempdir().unwrap();
        let (_cfg, app) = rest_app(&dir);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/context/query")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"type":"foo","query":"x"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], MSG_INVALID_CONTEXT_TYPE);
        assert_eq!(json["code"], "invalid_input");
    }

    #[tokio::test]
    async fn steering_env_403() {
        let dir = tempfile::tempdir().unwrap();
        let (_cfg, app) = rest_app(&dir);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/steering?file=.env")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["code"], "forbidden");
        assert!(json["error"].as_str().unwrap().contains(".env"));
    }

    #[tokio::test]
    async fn put_task_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let dare = dir.path().join("DARE");
        std::fs::create_dir_all(&dare).unwrap();
        std::fs::write(
            dare.join("TASKS.md"),
            "| id | title | status |\n| mp051-001 | Skeleton | ⏳ PENDING |\n",
        )
        .unwrap();
        let (_cfg, app) = rest_app(&dir);
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/tasks/mp051-001")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"status":"DONE"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["id"], "mp051-001");
        assert_eq!(json["status"], "DONE");
        let text = std::fs::read_to_string(dare.join("TASKS.md")).unwrap();
        assert!(text.contains("✅"));
        assert!(text.contains("DONE"));
    }

    #[tokio::test]
    async fn graph_empty_query_400() {
        let dir = tempfile::tempdir().unwrap();
        let (_cfg, app) = rest_app(&dir);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/graph/locate")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"query":"  "}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["code"], "invalid_input");
    }

    #[tokio::test]
    async fn path_id_escape_403() {
        let dir = tempfile::tempdir().unwrap();
        let (_cfg, app) = rest_app(&dir);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/tasks/a..b")
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
    async fn dashboard_put_tasks_404() {
        let (cfg, state, _dir) = test_cfg_state(false, DEFAULT_BODY_LIMIT);
        let app = create_app(AppMode::Dashboard, &cfg, state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/tasks/mp051-001")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"status":"DONE"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
