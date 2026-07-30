//! Streamable HTTP MCP transport (microplano 052 / mp052-005).

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::Router;
use dare_core::{CoreError, CoreResult};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use tokio::net::TcpListener;

use crate::auth::auth_middleware;
use crate::config::DEFAULT_BODY_LIMIT;
use crate::mode::AppMode;
use crate::services::ServiceCtx;
use crate::state::AppState;

use super::handler::McpHandler;

/// Default bind for MCP streamable-http (distinct from REST `:3000`).
pub const DEFAULT_MCP_HTTP_BIND: &str = "127.0.0.1";
/// Default port for MCP streamable-http (RF-15 / T-08).
pub const DEFAULT_MCP_HTTP_PORT: u16 = 3100;
/// Env override for MCP HTTP bind (do not reuse `DARE_MCP_BIND` / REST).
pub const ENV_MCP_HTTP_BIND: &str = "DARE_MCP_HTTP_BIND";
/// Env override for MCP HTTP port (do not reuse `DARE_MCP_PORT` / REST).
pub const ENV_MCP_HTTP_PORT: &str = "DARE_MCP_HTTP_PORT";

/// Build the Axum router that serves MCP streamable-http under `/mcp`.
///
/// Auth mirrors 051: loopback exempt unless `force_auth`; non-loopback / force
/// requires Bearer. Unauthorized responses use Class A `{error,code}` JSON via
/// [`crate::auth::auth_middleware`].
pub fn create_mcp_http_router(
    ctx: ServiceCtx,
    token: Arc<str>,
    force_auth: bool,
) -> Router {
    create_mcp_http_router_with_config(ctx, token, force_auth, StreamableHttpServerConfig::default())
}

fn create_mcp_http_router_with_config(
    ctx: ServiceCtx,
    token: Arc<str>,
    force_auth: bool,
    mut config: StreamableHttpServerConfig,
) -> Router {
    if config.max_request_body_bytes < DEFAULT_BODY_LIMIT {
        config.max_request_body_bytes = DEFAULT_BODY_LIMIT;
    }

    let factory_ctx = ctx.clone();
    let service: StreamableHttpService<McpHandler, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(McpHandler::new(factory_ctx.clone())),
            Arc::new(LocalSessionManager::default()),
            config,
        );

    let mut state = AppState::new(
        ctx.root,
        token,
        DEFAULT_BODY_LIMIT,
        AppMode::Rest,
        env!("CARGO_PKG_VERSION"),
    );
    state.force_auth = force_auth;

    Router::new()
        .nest_service("/mcp", service)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

/// Serve MCP over streamable-http on `bind:port` until Ctrl+C (graceful shutdown).
///
/// Defaults used by CLI/env elsewhere: [`DEFAULT_MCP_HTTP_BIND`]:[`DEFAULT_MCP_HTTP_PORT`]
/// (`127.0.0.1:3100`). Does **not** bind REST port 3000.
pub async fn serve_streamable_http(
    ctx: ServiceCtx,
    bind: IpAddr,
    port: u16,
    token: Arc<str>,
    force_auth: bool,
) -> CoreResult<()> {
    if port == 0 {
        return Err(CoreError::invalid_input("port must be in 1..=65535"));
    }

    let addr = SocketAddr::new(bind, port);
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| CoreError::io(format!("bind {addr}: {e}")))?;
    let local = listener
        .local_addr()
        .map_err(|e| CoreError::io(format!("local_addr: {e}")))?;

    let config = StreamableHttpServerConfig::default()
        .with_max_request_body_bytes(DEFAULT_BODY_LIMIT)
        .with_allowed_hosts([
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "::1".to_string(),
            bind.to_string(),
            format!("{bind}:{}", local.port()),
        ]);
    let shutdown_ct = config.cancellation_token.clone();

    let app = create_mcp_http_router_with_config(ctx, token, force_auth, config);

    eprintln!(
        "dare-server mcp streamable-http listening bind={} port={}",
        bind,
        local.port()
    );

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        let _ = tokio::signal::ctrl_c().await;
        shutdown_ct.cancel();
    })
    .await
    .map_err(|e| CoreError::io(format!("mcp streamable-http serve: {e}")))?;

    Ok(())
}
