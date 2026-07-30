# CLI MCP (`dare mcp serve`)

> **DEC-053** · Microplano 052 · Library: `crates/dare-server` (`mcp` feature) · CLI: `commands/mcp.rs` · Alias: `src/bin/dare_mcp_server.rs`

## Purpose

Real **Model Context Protocol** (JSON-RPC) transport for DARE domain tools. Domain logic lives in shared **`dare-server` services**; MCP is a separate surface from REST/dashboard (**051** / DEC-052). SDK: **`rmcp =3.0.1`**.

Per **[ADR-004](../adr/ADR-004-rest-compativel-e-mcp-real.md)**: REST compatible HTTP **≠** MCP. Ports stay distinct — REST **3000** vs MCP streamable-http **3100**.

| Surface | Role |
|---------|------|
| `dare mcp serve` | MCP over **stdio** (default) or **streamable-http** |
| `dare-mcp-server` | Transition **alias** — serves legacy **REST only** + deprecation on stderr (**never** silent MCP) |

## Commands

```text
dare mcp serve [--transport stdio|streamable-http] [-d <dir>]
               [--bind <ip>] [--port <u16>]   # streamable-http only
```

| Flag | Default | Effect |
|------|---------|--------|
| `--transport` | **`stdio`** | `stdio` \| `streamable-http` (else Usage exit **2**) |
| `--bind` | `127.0.0.1` | streamable-http only; bind address |
| `--port` | **3100** | streamable-http only (`1..=65535`; `0` → InvalidInput) |
| `-d` / `--dir` | cwd | Project root (also honors `DARE_PROJECT_PATH`) |

## Exit codes

| Code | When |
|------|------|
| **0** | Serve until EOF / Ctrl+C |
| **2** | Usage / unknown transport |
| **3** | Project root missing / NotFound |
| **4** | Invalid bind/port/args (InvalidInput) |
| **5** | Bind IO / serve failure |

## Environment (streamable-http)

| Env | Role |
|-----|------|
| `DARE_MCP_HTTP_BIND` | Bind override (CLI `--bind` wins when set) |
| `DARE_MCP_HTTP_PORT` | Port override (CLI `--port` wins when set) |
| `DARE_MCP_TOKEN` | Bearer token for non-loopback HTTP (same family as 051) |
| `DARE_PROJECT_PATH` | Project root when `-d` omitted |

Priority: CLI bind/port > `DARE_MCP_HTTP_*` > defaults (`127.0.0.1`:**3100**).

Do **not** confuse with `DARE_MCP_BIND` / `DARE_MCP_PORT` used by REST/dashboard (**051**) — those remain the legacy REST env names.

## Tools (`tools/list` frozen order)

Ten tools (Blueprint §0.4). Order is stable and contract-frozen:

1. `project`
2. `blueprint`
3. `dag`
4. `task_get`
5. `task_put`
6. `context_query`
7. `graph_locate`
8. `graph_traverse`
9. `graph_map_requirement`
10. `steering_show`

Names are snake_case domain ids (**not** REST path strings). Auth on streamable-http mirrors 051: Bearer + loopback exempt.

## Alias `dare-mcp-server` (ADR-004)

Binary always:

1. Prints deprecation on **stderr** (before listen / even on `--help` paths that reach `main`)
2. Serves **`AppMode::Rest`** (legacy Express-compatible REST) — **never** MCP/JSON-RPC

Canonical message:

```text
dare-mcp-server is deprecated: it serves legacy REST only. Use 'dare server --protocol rest' or 'dare mcp serve' for MCP.
```

Silent MCP substitution is **forbidden**.

## Compatibility vs TS 3.18.1 / REST 051

| Topic | Class | Notes |
|-------|-------|-------|
| REST wire 051 after services refactor | A | Regression suite must stay green; auth parity with Mestre |
| MCP tool names ≠ `/tools` REST announcement | B | Protocol-specific ids; map documented here |
| Alias serves REST (name ≠ MCP) | B | ADR-004 transition window |
| Tool result envelope `schemaVersion` | B | New MCP envelope |
| Resources / Prompts MCP | C | Out of scope 052 |
| Future alias removal | C | Post-1.0 |

## Capability

`dare-mcp` → `cli_commands: ["mcp"]` in `assets/capability-matrix.yml`.

## Examples

```bash
dare mcp serve
dare mcp serve --transport stdio -d .
dare mcp serve --transport streamable-http
dare mcp serve --transport streamable-http --bind 127.0.0.1 --port 3100 -d .
# deprecated REST alias (stderr warning):
dare-mcp-server --port 3000
```

## Out of scope (052)

Self-update / package managers (**053**), OAuth/TLS, WebSocket, MCP Resources/Prompts, cloud-hosted MCP, rewriting REST/dashboard (**051**).
