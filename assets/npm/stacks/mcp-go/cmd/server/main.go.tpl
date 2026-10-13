// {{projectName}} — MCP server entrypoint.
package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"

	"github.com/joho/godotenv"

	"{{moduleName}}/internal/server"
	"{{moduleName}}/internal/transports"
)

func main() {
	_ = godotenv.Load()

	defaultTransport := os.Getenv("MCP_TRANSPORT")
	if defaultTransport == "" {
		defaultTransport = "{{defaultTransport}}"
	}

	transport := flag.String("transport", defaultTransport, "stdio | sse | http")
	jsonOut := flag.Bool("json", false, "emit JSON output (use with --list-tools)")
	listTools := flag.Bool("list-tools", false, "list registered tools as JSON and exit")
	flag.Parse()

	// --json --list-tools short-circuits before starting any transport (M-03).
	if *listTools && *jsonOut {
		out, _ := json.MarshalIndent(server.InventoryJSON(), "", "  ")
		fmt.Println(string(out))
		return
	}

	host := envOr("MCP_HOST", "0.0.0.0")
	port := envOr("MCP_PORT", "3001")

	var err error
	switch *transport {
	case "stdio":
		err = transports.RunStdio()
	case "sse":
		err = transports.RunSSE(host, port)
	case "http":
		err = transports.RunHTTP(host, port)
	default:
		fmt.Fprintf(os.Stderr, "invalid transport %q (valid: stdio, sse, http)\n", *transport)
		os.Exit(1)
	}
	if err != nil {
		fmt.Fprintf(os.Stderr, "[mcp] fatal: %v\n", err)
		os.Exit(1)
	}
}

func envOr(key, def string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return def
}
