"""stdio transport — the default for local agents."""
from mcp.server.fastmcp import FastMCP


def run(mcp: FastMCP) -> None:
    mcp.run(transport="stdio")
