package transports

import (
	"fmt"

	mcpserver "github.com/mark3labs/mcp-go/server"

	srv "{{moduleName}}/internal/server"
)

// RunSSE serves the MCP server over SSE-over-HTTP.
func RunSSE(host, port string) error {
	s := srv.New()
	addr := fmt.Sprintf("%s:%s", host, port)
	sse := mcpserver.NewSSEServer(s)
	return sse.Start(addr)
}
