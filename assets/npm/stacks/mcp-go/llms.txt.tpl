# {{projectName}}

DARE-shaped MCP server (Go, mark3labs/mcp-go). Default transport: **{{defaultTransport}}**. BETA.

## Setup

```bash
cp .env.example .env
go mod tidy
go run ./cmd/server
```

## Commands

- `go run ./cmd/server` — start server (transport via --transport or MCP_TRANSPORT env)
- `go test ./...` — tests
- `golangci-lint run` — lint
- `govulncheck ./...` — vuln scan
- `npx @modelcontextprotocol/inspector go run ./cmd/server` — MCP Inspector

## Tools (registered in internal/server/server.go)

| Name | Input | Output | Description |
|---|---|---|---|
| `echo` | `text: string` | `string` | Returns input verbatim. Smoke test. |

## Prompts

| Name | Args | Description |
|---|---|---|
| `summarize` | `text` | Summarize the input text. Placeholder. |

## Transports

Choose with `--transport stdio|sse|http` or `MCP_TRANSPORT`:

- `stdio` (default) — local agents. No port.
- `sse` — SSE-over-HTTP on MCP_PORT (default 3001).
- `http` — Streamable HTTP on MCP_PORT (default 3001).

## Architecture

- `cmd/server/main.go` — entrypoint, flag parsing, transport dispatch.
- `internal/server/server.go` — builds the mcp-go server with tools + prompts.
- `internal/tools/`, `internal/prompts/` — pure functions, unit-tested.
- `internal/transports/` — one runner per transport.

## CLI inventory

`go run ./cmd/server --json --list-tools` prints the tool registry as JSON (M-03).

## Note on beta status

`mark3labs/mcp-go` is community-led. If Anthropic ships an official Go SDK,
migrate `internal/server` + `internal/transports` to it; the pure
tools/prompts functions stay unchanged.

## DARE skills

See `.dare/skills.yml`.
