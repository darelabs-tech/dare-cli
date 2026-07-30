//! MCP transport (microplano 052). Tools + handler (mp052-003); stdio serve (mp052-004).

mod error_map;
mod handler;
mod serve_stdio;
mod tools;

pub use error_map::{map_core_error, map_http_error};
pub use handler::McpHandler;
pub use serve_stdio::serve_stdio;
pub use tools::{dispatch, tool_definitions, TOOL_NAMES};

/// Returns `true` when the `mcp` Cargo feature is enabled (compile-time gate smoke).
pub fn mcp_feature_ok() -> bool {
    true
}
