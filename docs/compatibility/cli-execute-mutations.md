# CLI execute mutations (`dare execute --complete|--fail|--reset`)

> **DEC-030** · Microplano 029 · Source: `crates/dare-verify` + `crates/dare-cli/src/commands/execute.rs`

## Purpose

Mutate DAG runtime state after a task is implemented: mark **DONE** only if Ralph (build→test→lint) passes; mark **FAILED** with cascade; **reset** to PENDING while preserving attempt history.

Complements [`cli-execute-status.md`](cli-execute-status.md) (DEC-029: status/next/watch; no Start-on-next).

## Command

```bash
dare execute --complete <TASK_ID> [--output <TEXT>] [--dag PATH]
dare execute --fail <TASK_ID> [--reason <TEXT>] [--dag PATH]
dare execute --reset <TASK_ID> [--dag PATH]
```

Flags are mutually exclusive with `--status` / `--next` / `--watch` (and each other). Global: `--json` / `--no-color`.

| Flag | Default | Effect |
|------|---------|--------|
| `--complete <id>` | — | Auto-Start if PENDING → Ralph → DONE only if gates OK |
| `--output` | `Task completed.` | Persisted as `task.output` (truncated) |
| `--fail <id>` | — | Auto-Start if PENDING → FAILED + cascade skip |
| `--reason` | `Task failed.` | Persisted as `task.error` |
| `--reset <id>` | — | → PENDING; clears output/error; **keeps attempts** |
| `--dag` | `DARE/dare-dag.yaml` | Project jail |

## Ralph Loop

Order: **build → test → lint**. Per-gate timeout **600 s**. Stack from `dare.config.json` `backend` (fallback `rust-axum` if Cargo.toml present).

| Stack | Status |
|-------|--------|
| `rust-axum` / `rust` | Implemented (`cargo build/test/clippy --workspace`) |
| Other known IDs | `not implemented` → exit **4** |

### Gate failure

- Task stays **RUNNING** (not auto-FAILED)
- Writes `.dare/verification/<id>.json` with `ok: false`
- Process exit **1** (non-timeout) or **124** (timeout)
- **No** `DONE`

### Test harness

| `DARE_RALPH_MOCK` | Behavior |
|-------------------|----------|
| unset | Real `SystemProcessRunner` |
| `1` / `pass` | Three gates exit 0 (no spawn) |
| `fail` | Build exit 1 |
| `timeout` | Build timed out (exit 124) |

Test-only — not a CLI flag.

## Verification artifact

Path: `.dare/verification/<taskId>.json` (schema version **1**, camelCase). File-only ingest in this cycle (no GraphRAG).

## Exit codes

| Code | When |
|------|------|
| 0 | complete / fail / reset OK |
| 1 | Ralph gate failed (non-timeout) |
| 2 | Usage (exclusive flags) |
| 3 | DAG or task NotFound |
| 4 | InvalidInput / transition / stack / unsafe id |
| 5 | Io (lock / write) |
| **124** | Ralph gate timeout |

## Out of scope

- `--agent` / worktrees / budget (030+)
- `dare review` / mutation / formal / best-of-N (032/049)
- GraphRAG ingest (040+)

## Container

```bash
docker compose -f docker-compose.ci.yml config
```

Verified exit 0 in mp029-001 — no waiver.

## Local verify

```bash
DARE_RALPH_MOCK=pass dare execute --complete task-001
dare execute --fail task-001 --reason "x"
dare execute --reset task-001
cargo test -p dare-verify
cargo test -p dare-cli --test cli_smoke -- execute
```

## Related

- **DEC-030** · DEC-029 (status/next/watch)
- Runtime: [`dag-runtime.md`](dag-runtime.md)
- Capability: `dare-execute` → `cli_commands: ["execute"]`
