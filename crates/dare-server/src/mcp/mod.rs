//! MCP transport (microplano 052). Tools + handler (mp052-003); serve in later tasks.

mod error_map;
mod handler;
mod tools;

pub use error_map::{map_core_error, map_http_error};
pub use handler::McpHandler;
pub use tools::{dispatch, tool_definitions, TOOL_NAMES};

/// Returns `true` when the `mcp` Cargo feature is enabled (compile-time gate smoke).
pub fn mcp_feature_ok() -> bool {
    true
}
