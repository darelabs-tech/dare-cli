// Package transports — one runner per MCP transport.
package transports

import (
	mcpserver "github.com/mark3labs/mcp-go/server"

	srv "{{moduleName}}/internal/server"
)

// RunStdio serves over stdin/stdout — the default for local agents.
func RunStdio() error {
	s := srv.New()
	return mcpserver.ServeStdio(s)
}
