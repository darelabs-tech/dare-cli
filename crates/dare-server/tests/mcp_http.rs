//! MCP streamable-http auth tests (mp052-005).

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use dare_core::ProjectRoot;
use dare_server::mcp::{create_mcp_http_router, DEFAULT_MCP_HTTP_PORT};
use dare_server::{ServiceCtx, MSG_UNAUTHORIZED};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn http_force_auth_401() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = ProjectRoot::new(dir.path()).expect("root");
    let ctx = ServiceCtx::new(root);
    let token: Arc<str> = Arc::from("test-token-ok");
    let app = create_mcp_http_router(ctx, token, true);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
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

#[test]
fn mcp_http_defaults_not_rest_3000() {
    assert_eq!(DEFAULT_MCP_HTTP_PORT, 3100);
    assert_ne!(DEFAULT_MCP_HTTP_PORT, 3000);
}
