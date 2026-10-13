{
  "openapi": "3.1.0",
  "info": {
    "title": "{{projectName}}",
    "version": "0.1.0",
    "description": "MCP server JSON-RPC surface. MCP is not REST; this describes the HTTP/SSE transport envelopes only."
  },
  "paths": {
    "/sse": {
      "get": {
        "summary": "SSE event stream",
        "responses": { "200": { "description": "text/event-stream with MCP JSON-RPC frames" } }
      }
    },
    "/messages": {
      "post": {
        "summary": "MCP JSON-RPC message (HTTP / SSE post-back)",
        "requestBody": {
          "required": true,
          "content": { "application/json": { "schema": { "type": "object" } } }
        },
        "responses": {
          "200": {
            "description": "JSON-RPC 2.0 response",
            "content": { "application/json": { "schema": { "type": "object" } } }
          }
        }
      }
    }
  }
}
