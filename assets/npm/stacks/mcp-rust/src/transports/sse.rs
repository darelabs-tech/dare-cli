//! SSE-over-HTTP transport.
//!
//! rmcp exposes an SSE server behind the `transport-sse-server` feature in
//! recent releases; the surface is pre-1.0. This wrapper keeps the bind logic
//! in one place. If the SDK feature name or constructor changes, only this
//! file needs editing.
use std::net::SocketAddr;

use crate::server::build_service;

pub async fn run(host: &str, port: u16) -> anyhow::Result<()> {
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!("mcp/sse listening on {addr}");

    // Build the service so capabilities are validated even if the concrete
    // SSE wiring below is adjusted for your rmcp version.
    let _service = build_service();

    // Minimal axum app that documents where the SSE endpoints live. Replace
    // the route bodies with rmcp's SSE server integration for your version
    // (e.g. rmcp::transport::sse_server::SseServer::serve(addr)).
    let app = axum::Router::new()
        .route("/sse", axum::routing::get(|| async { "MCP SSE endpoint — wire rmcp SseServer here" }))
        .route(
            "/messages",
            axum::routing::post(|| async { "MCP message endpoint — wire rmcp SseServer here" }),
        )
        .layer(tower_http::cors::CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
