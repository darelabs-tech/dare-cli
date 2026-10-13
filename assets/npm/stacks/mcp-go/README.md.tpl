# {{projectName}}

DARE-shaped MCP server in Go using `mark3labs/mcp-go`.
Default transport: **{{defaultTransport}}**.

> ⚠️ **Beta.** `mark3labs/mcp-go` is the community Go SDK. Migrate to an
> official Anthropic Go SDK if/when one ships.

## Setup

```bash
cp .env.example .env
go mod tidy
go run ./cmd/server          # uses {{defaultTransport}} transport by default
```

## Test with MCP Inspector

```bash
npx @modelcontextprotocol/inspector go run ./cmd/server
```

## Tools

- `echo` — returns its input. Canonical smoke test.

## Prompts

- `summarize` — placeholder prompt template taking a `text` argument.

## Transports

Pick at runtime via `--transport stdio|sse|http` or `MCP_TRANSPORT`:

| Transport | When to use |
|---|---|
| `stdio` (default) | Local CLI agents |
| `sse`             | Web clients via SSE |
| `http`            | Streamable HTTP |

## Commands

- `go run ./cmd/server` — start server
- `go test ./...` — tests
- `golangci-lint run` — lint
- `govulncheck ./...` — vuln scan

## DARE Method

See `.dare/skills.yml` and `llms.txt`.
