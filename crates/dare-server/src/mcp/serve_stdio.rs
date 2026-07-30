//! MCP stdio transport serve (microplano 052 / mp052-004).

use dare_core::{CoreError, CoreResult};
use rmcp::ServiceExt;

use crate::services::ServiceCtx;

use super::handler::McpHandler;

/// Serve MCP over stdin/stdout until the peer disconnects.
///
/// Uses rmcp 3.x `ServiceExt::serve` with [`rmcp::transport::stdio`].
pub async fn serve_stdio(ctx: ServiceCtx) -> CoreResult<()> {
    let handler = McpHandler::new(ctx);
    let running = handler
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| CoreError::io(format!("mcp stdio serve failed: {e}")))?;
    let _quit = running
        .waiting()
        .await
        .map_err(|e| CoreError::io(format!("mcp stdio wait failed: {e}")))?;
    Ok(())
}
