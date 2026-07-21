# CLI info (`dare info`)

> **DEC-018** · Microplano 017 · Source: `crates/dare-cli/src/commands/info.rs`

## Purpose

Read-only diagnosis of the native CLI install and the current project: version, platform, project root, embedded assets integrity, config/DARE/state presence, graph path, backend/IDE, and approximate TASKS progress. **Zero filesystem mutations.**

## Command

```bash
dare info [--root <path>]
dare info --json [--root <path>]
```

| Flag | Effect |
|------|--------|
| `--root <path>` | Start directory for project-root walk (default: cwd) |
| `--json` | Envelope via output renderer (004); `data` = InfoReport schema 1 |
| `--no-color` | Global; no ANSI in human mode |

## JSON schema version 1 (frozen)

CamelCase fields:

| Field | Type | Notes |
|-------|------|-------|
| `schemaVersion` | number | Always `1` |
| `version` | string | `CARGO_PKG_VERSION` |
| `platform` | `{os,arch,family}` | `std::env::consts` |
| `projectRoot` | string\|null | Absolute display path |
| `assetsOk` | bool | `verify_embedded_assets` |
| `assetsError` | string\|null | |
| `configPresent` | bool | `dare.config.json` |
| `graphPath` | string\|null | |
| `graphPresent` | bool | |
| `backend` | string\|null | `ide` preferred, else `backend` |
| `tasks` | `{source,done,pending,totalMarked}` | Heuristic |
| `dareDirPresent` | bool | |
| `statePresent` | bool | `.dare/state.json` |

Bump requires ADR + migration note.

## Project root markers (walk-up)

Stops at first ancestor containing any of:

- file `dare.config.json`
- directory `DARE/`
- file `Cargo.toml`

## Disk contracts (read-only)

| Path | Use |
|------|-----|
| `dare.config.json` | Marker + `ide`/`backend` |
| `DARE/TASKS.md` or `DARE/TASKS-*.md` | Progress heuristic |
| `dare-graph.yml` or `DARE/dare-graph.yml` | Graph path |
| `.dare/state.json` | Presence only |

## TASKS progress heuristic

1. Prefer `DARE/TASKS.md` if present.
2. Else list `DARE/TASKS-*.md`, **sort lexicographically**, pick first.
3. `done` = count of `✅` + count of `DONE`; `pending` = count of `⏳` + count of `PENDING` (may double-count rows that contain both emoji and word — documented v1 behavior).

Formal DAG state counting is out of scope (microplan 026+).

## Zero mutations

`collect_info` must not create, modify, or delete files. Unit test compares directory listing before/after. Human output includes `mode: read-only (zero mutations)`.

## Local verify (container)

```bash
docker compose -f docker-compose.ci.yml config
```

Inherits microplan 003/015 images.

## Tests

```bash
cargo test -p dare-cli -- info
cargo test -p dare-cli --test cli_smoke -- info
```

## Related

- **DEC-018** (decision log)
- Output envelope: [`cli-output-and-errors.md`](cli-output-and-errors.md)
- Path safety: ProjectRoot / SafeRelativePath (005)
- Assets: [`assets-inventory.md`](assets-inventory.md)
