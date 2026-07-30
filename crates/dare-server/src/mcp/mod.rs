//! MCP transport (microplano 052). Tools + handler (mp052-003); stdio (mp052-004); streamable-http (mp052-005).

mod error_map;
mod handler;
mod http;
mod serve_stdio;
mod tools;

pub use error_map::{map_core_error, map_http_error};
pub use handler::McpHandler;
pub use http::{
    create_mcp_http_router, serve_streamable_http, DEFAULT_MCP_HTTP_BIND, DEFAULT_MCP_HTTP_PORT,
    ENV_MCP_HTTP_BIND, ENV_MCP_HTTP_PORT,
};
pub use serve_stdio::serve_stdio;
pub use tools::{dispatch, tool_definitions, TOOL_NAMES};

/// Returns `true` when the `mcp` Cargo feature is enabled (compile-time gate smoke).
pub fn mcp_feature_ok() -> bool {
    true
}
