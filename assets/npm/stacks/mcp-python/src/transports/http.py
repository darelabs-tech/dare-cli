"""Streamable HTTP transport — modern MCP HTTP wire format.

Each client gets its own session keyed by the Mcp-Session-Id header,
generated on initialize. FastMCP wraps the uvicorn + Starlette plumbing.
"""
from mcp.server.fastmcp import FastMCP


def run(mcp: FastMCP, host: str = "0.0.0.0", port: int = 3001) -> None:
    mcp.settings.host = host
    mcp.settings.port = port
    mcp.run(transport="streamable-http")
