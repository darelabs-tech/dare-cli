//! Shared REST server startup for `dare server --protocol rest` and `dare-mcp-server`.

use std::path::PathBuf;

use dare_core::{CoreError, CoreResult};
use dare_server::{parse_server_config_from_env, serve, AppMode};

/// BLUEPRINT-052 §0.1 `MSG_ALIAS_DEPRECATED` (ADR-004 transition alias).
pub const MSG_ALIAS_DEPRECATED: &str = "dare-mcp-server is deprecated: it serves legacy REST only. Use 'dare server --protocol rest' or 'dare mcp serve' for MCP.";

/// CLI overrides for REST mode (`--bind`, `--port`, `-d`).
pub struct RestServerOpts {
    pub bind: Option<String>,
    pub port: Option<u16>,
    pub dir: Option<PathBuf>,
}

/// Start Axum in `AppMode::Rest` (never MCP/JSON-RPC).
pub fn run_rest_server(opts: RestServerOpts) -> CoreResult<()> {
    let root_path = opts
        .dir
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let cfg = parse_server_config_from_env(
        AppMode::Rest,
        opts.bind.as_deref(),
        opts.port,
        &root_path,
        false,
    )?;
    serve_until_ctrl_c(AppMode::Rest, cfg)
}

fn serve_until_ctrl_c(mode: AppMode, cfg: dare_server::ServerConfig) -> CoreResult<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CoreError::io(format!("tokio runtime: {e}")))?;
    rt.block_on(async move {
        serve(mode, cfg, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
    })
}
