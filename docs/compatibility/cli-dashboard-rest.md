# CLI dashboard + REST (`dare dashboard` / `dare server`)

> **DEC-052** · Microplano 051 · Library: `crates/dare-server` · CLI: `commands/dashboard.rs`, `commands/server.rs`

## Purpose

Local HTTP surfaces for DARE telemetry and legacy REST compatibility. Domain logic lives in **`dare-server`** (Axum); the CLI is a thin clap/I/O shell over `parse_server_config_from_env` + `serve`. Shared factory: **`create_app(mode, cfg, state)`** — both modes mount `/health` + dashboard; **`AppMode::Rest`** additionally mounts the legacy REST router.

Per **[ADR-004](../adr/ADR-004-rest-compativel-e-mcp-real.md)**: REST compatible HTTP **≠** MCP (JSON-RPC/stdio/SSE). MCP real is microplano **052**.

| Surface | Role |
|---------|------|
| `dare dashboard` | Read-only telemetry UI (`AppMode::Dashboard`); default port **4100**; opens browser unless `--no-open` |
| `dare server --protocol rest` | Legacy REST HTTP (`AppMode::Rest`); default port **3000**; no browser open |

## Commands

```text
dare dashboard [--port <u16>] [--no-open] [-d <dir>]
dare server --protocol rest [--bind <addr>] [--port <u16>] [-d <dir>]
```

| Flag | Default | Effect |
|------|---------|--------|
| `--port` | dashboard **4100** / server **3000** | Listen port (`1..=65535`; `0` → InvalidInput) |
| `--bind` | `127.0.0.1` | Server only; bind address |
| `--no-open` | false | Dashboard only; skip browser open after bind |
| `--protocol` | required | Server only; v1 accepts **`rest`** only (else Usage exit **2**) |
| `-d` / `--dir` | cwd | Project root (also honors `DARE_PROJECT_PATH`) |

Bind defaults: **`127.0.0.1`** for both modes (`DEFAULT_DASHBOARD_BIND` / `DEFAULT_REST_BIND`).

## Environment (`DARE_MCP_*`)

Env names preserve TS 3.18.1 compatibility (historical `dare-mcp-server` was Express REST, not MCP — ADR-004).

| Env | Role |
|-----|------|
| `DARE_MCP_BIND` | Bind override (CLI `--bind` wins when set) |
| `DARE_MCP_PORT` | Port override (CLI `--port` wins when set) |
| `DARE_MCP_TOKEN` | Bearer token (≥ 8 chars); else UUID v4 generated |
| `DARE_MCP_BODY_LIMIT` | Request body cap (default **1_048_576**; supports `k`/`mb`/`mib`; range 1024…16 MiB) |
| `DARE_MCP_LOG_TOKEN` | `1`/`true` → log token **value** at startup; default logs only `token=set\|generated` |
| `DARE_PROJECT_PATH` | Project root when `-d` omitted |

Priority: CLI bind/port overrides > env > mode defaults.

## Auth

Bearer + **loopback exempt**:

| Peer | `Authorization: Bearer <token>` | Result |
|------|----------------------------------|--------|
| Loopback | absent | OK |
| Loopback | present but wrong | **401** `{error,code:"unauthorized"}` |
| Non-loopback | absent / malformed / wrong | **401** |
| Non-loopback | exact match (constant-time) | OK |

Loopback: `SocketAddr::ip().is_loopback()` via `ConnectInfo`. Oneshot tests without `ConnectInfo` are treated as loopback unless `force_auth` (test helper).

## Key routes

Shared (both modes):

| Method | Path | Notes |
|--------|------|-------|
| GET | `/health` | Smoke; `{ok,protocol:"rest",…}` |
| GET | `/dashboard` | HTML UI (+ static embed) |
| GET | `/api/telemetry` | `TelemetrySnapshot` JSON |

REST legacy (`AppMode::Rest` only) — frozen `/tools` announcement (12 entries):

| Method | Path |
|--------|------|
| GET | `/tools` |
| POST | `/context/query` |
| GET | `/blueprint` |
| GET | `/dag` |
| GET/PUT | `/tasks/{id}` |
| POST | `/graph/locate` |
| POST | `/graph/traverse` |
| POST | `/graph/map-requirement` |
| GET | `/project` |
| GET | `/steering` |

Body over limit → **413** `{error:"request body too large",code:"body_too_large"}`.

## Compatibility vs TS 3.18.1

| Topic | Class | Notes |
|-------|-------|-------|
| Auth loopback exempt; Bearer off-loopback | A | Parity with Mestre §6.1 |
| Body limit 1 MiB → **413** | B | TS may return 400; Rust freezes **413** |
| `map-requirement` via `locate` + filter `requirement` (fallback full locate) | B | No identical dedicated TS API in Rust graph |
| Binary alias `dare-mcp-server` | C | Deferred; surface is `dare server --protocol rest` (ADR-004 transition window required) |
| Shared Axum `create_app` | A/B | Rewrite stack (Axum vs Express); contract paths preserved |

## Capability

`dare-dashboard` → `cli_commands: ["dashboard", "server"]` in `assets/capability-matrix.yml` (existing id; no duplicate).

## Examples

```bash
dare dashboard
dare dashboard --port 4100 --no-open -d .
dare server --protocol rest
dare server --protocol rest --bind 127.0.0.1 --port 3000 -d .
```

## Out of scope (051)

MCP JSON-RPC/stdio/SSE (**052**), OAuth/TLS, WebSocket, Docker phase, binary alias `dare-mcp-server` (COULD / ADR-004 window).
