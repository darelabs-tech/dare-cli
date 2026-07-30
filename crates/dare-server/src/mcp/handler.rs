//! MCP `ServerHandler` holding [`ServiceCtx`] (microplano 052).

use std::future::Future;

use dare_core::ProjectRoot;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ErrorData, JsonObject,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};

use crate::services::ServiceCtx;

use super::tools::{self, TOOL_NAMES};

/// In-process MCP server handler backed by shared domain services.
#[derive(Debug, Clone)]
pub struct McpHandler {
    ctx: ServiceCtx,
}

impl McpHandler {
    pub(crate) fn new(ctx: ServiceCtx) -> Self {
        Self { ctx }
    }

    pub fn from_root(root: ProjectRoot) -> Self {
        Self::new(ServiceCtx::new(root))
    }

    /// Frozen tool names (same order as `tools/list`).
    pub fn tool_names() -> &'static [&'static str] {
        &TOOL_NAMES
    }

    /// In-process `tools/list` (no peer / transport required).
    pub fn list_tools_now(&self) -> ListToolsResult {
        ListToolsResult::with_all_items(tools::tool_definitions())
    }

    /// In-process `tools/call` (no peer / transport required).
    pub fn call_tool_now(
        &self,
        name: &str,
        arguments: Option<JsonObject>,
    ) -> Result<CallToolResult, ErrorData> {
        tools::dispatch(&self.ctx, name, arguments.as_ref())
    }
}

impl ServerHandler for McpHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("DARE MCP tools (project, dag, tasks, graph, steering)")
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        std::future::ready(Ok(self.list_tools_now()))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        tools::tool_definitions()
            .into_iter()
            .find(|t| t.name.as_ref() == name)
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResponse, ErrorData>> + Send + '_ {
        let name = request.name.to_string();
        let result = self.call_tool_now(&name, request.arguments);
        std::future::ready(result.map(CallToolResponse::from))
    }
}
