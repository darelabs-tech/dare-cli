"""SSE-over-HTTP transport — legacy MCP HTTP wire format.

FastMCP launches a uvicorn server under the hood and exposes the SSE
endpoints. For new deployments prefer the streamable HTTP transport
(transports/http.py).
"""
from mcp.server.fastmcp import FastMCP


def run(mcp: FastMCP, host: str = "0.0.0.0", port: int = 3001) -> None:
    mcp.settings.host = host
    mcp.settings.port = port
    mcp.run(transport="sse")
