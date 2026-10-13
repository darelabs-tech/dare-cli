//! Streamable HTTP transport.
//!
//! rmcp exposes a streamable-http server behind a feature flag in recent
//! releases (pre-1.0). This wrapper centralizes the bind logic; swap the route
//! bodies for rmcp's StreamableHttpService integration for your version.
use std::net::SocketAddr;

use crate::server::build_service;

pub async fn run(host: &str, port: u16) -> anyhow::Result<()> {
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!("mcp/http listening on {addr}");

    let _service = build_service();

    let app = axum::Router::new()
        .route(
            "/messages",
            axum::routing::post(|| async { "MCP streamable-http endpoint — wire rmcp here" })
                .get(|| async { "MCP streamable-http endpoint — wire rmcp here" }),
        )
        .layer(tower_http::cors::CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
