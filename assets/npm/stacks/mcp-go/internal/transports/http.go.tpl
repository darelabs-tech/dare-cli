package transports

import (
	"fmt"

	mcpserver "github.com/mark3labs/mcp-go/server"

	srv "{{moduleName}}/internal/server"
)

// RunHTTP serves the MCP server over Streamable HTTP.
//
// NOTE: the streamable-http constructor name may vary across mark3labs/mcp-go
// releases. If `go build` fails after an SDK bump, check the server package
// for the current streamable HTTP server type.
func RunHTTP(host, port string) error {
	s := srv.New()
	addr := fmt.Sprintf("%s:%s", host, port)
	httpServer := mcpserver.NewStreamableHTTPServer(s)
	return httpServer.Start(addr)
}
