//! `dare mcp serve` — MCP transport CLI (microplano 052 / mp052-004).

use std::net::IpAddr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use dare_core::{CoreError, CoreResult, ProjectRoot};
use dare_server::{mcp::serve_stdio, ServiceCtx, ENV_PROJECT};

use crate::output::OutputRenderer;

/// Format `MSG_UNKNOWN_TRANSPORT` for a given transport string.
pub fn msg_unknown_transport(transport: &str) -> String {
    format!("unknown transport: {transport} (expected stdio|streamable-http)")
}

pub const DEFAULT_MCP_TRANSPORT: &str = "stdio";
pub const DEFAULT_MCP_HTTP_BIND: &str = "127.0.0.1";
pub const DEFAULT_MCP_HTTP_PORT: u16 = 3100;
pub const ENV_MCP_HTTP_BIND: &str = "DARE_MCP_HTTP_BIND";
pub const ENV_MCP_HTTP_PORT: &str = "DARE_MCP_HTTP_PORT";

/// Message when streamable-http is selected before mp052-005 lands.
pub const MSG_STREAMABLE_HTTP_NOT_IMPLEMENTED: &str = "streamable-http not implemented yet";

/// CLI args for `dare mcp serve`.
pub struct McpServeCliOpts {
    pub transport: String,
    pub bind: Option<String>,
    pub port: Option<u16>,
    pub dir: Option<PathBuf>,
}

/// Run `dare mcp serve [--transport] [--bind] [--port] [-d]`.
pub fn run_mcp_serve_cmd(opts: McpServeCliOpts, renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_mcp_serve(opts) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let code = renderer.write_error(&e);
            ExitCode::from(code as u8)
        }
    }
}

fn run_mcp_serve(opts: McpServeCliOpts) -> CoreResult<()> {
    let transport = opts.transport.trim();
    let transport = if transport.is_empty() {
        DEFAULT_MCP_TRANSPORT
    } else {
        transport
    };

    match transport {
        "stdio" => serve_stdio_blocking(opts.dir),
        "streamable-http" => serve_streamable_http_cli(opts),
        other => Err(CoreError::usage(msg_unknown_transport(other))),
    }
}

fn resolve_project_root(dir: Option<PathBuf>) -> CoreResult<ProjectRoot> {
    let project_path = std::env::var(ENV_PROJECT)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .or(dir)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    ProjectRoot::new(&project_path)
}

fn serve_stdio_blocking(dir: Option<PathBuf>) -> CoreResult<()> {
    let root = resolve_project_root(dir)?;
    let ctx = ServiceCtx::new(root);
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CoreError::io(format!("tokio runtime: {e}")))?;
    rt.block_on(serve_stdio(ctx))
}

fn serve_streamable_http_cli(opts: McpServeCliOpts) -> CoreResult<()> {
    let (_bind, _port) = resolve_http_bind_port(opts.bind.as_deref(), opts.port)?;
    let root = resolve_project_root(opts.dir)?;
    let _ctx = ServiceCtx::new(root);

    // Prefer calling `serve_streamable_http` when mp052-005 has added it.
    serve_streamable_http_or_stub(_ctx, _bind, _port)
}

fn resolve_http_bind_port(
    bind_override: Option<&str>,
    port_override: Option<u16>,
) -> CoreResult<(IpAddr, u16)> {
    let bind_str = bind_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            std::env::var(ENV_MCP_HTTP_BIND)
                .ok()
                .filter(|s| !s.trim().is_empty())
        })
        .unwrap_or_else(|| DEFAULT_MCP_HTTP_BIND.to_string());
    let bind = IpAddr::from_str(bind_str.trim())
        .map_err(|_| CoreError::invalid_input(format!("invalid bind address: {bind_str}")))?;

    let port = if let Some(p) = port_override {
        p
    } else if let Ok(raw) = std::env::var(ENV_MCP_HTTP_PORT) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            DEFAULT_MCP_HTTP_PORT
        } else {
            trimmed
                .parse::<u16>()
                .map_err(|_| CoreError::invalid_input(format!("invalid port: {raw}")))?
        }
    } else {
        DEFAULT_MCP_HTTP_PORT
    };
    if port == 0 {
        return Err(CoreError::invalid_input("port must be in 1..=65535"));
    }
    Ok((bind, port))
}

/// Call `dare_server::mcp::serve_streamable_http` when present; otherwise exit-4 stub.
///
/// mp052-005 owns the real HTTP serve implementation. This helper keeps the CLI
/// wired so a future worktree that already has the symbol can switch by updating
/// this function body only.
fn serve_streamable_http_or_stub(
    _ctx: ServiceCtx,
    _bind: IpAddr,
    _port: u16,
) -> CoreResult<()> {
    // When `serve_streamable_http` exists in dare-server::mcp (mp052-005), prefer:
    //   rt.block_on(dare_server::mcp::serve_streamable_http(ctx, bind, port, token, force_auth))
    Err(CoreError::invalid_input(MSG_STREAMABLE_HTTP_NOT_IMPLEMENTED))
}
