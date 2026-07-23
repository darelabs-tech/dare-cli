# CLI execute status / next / watch (`dare execute`)

> **DEC-029** · Microplano 028 · Source: `crates/dare-dag/src/execution.rs` + `crates/dare-cli/src/commands/execute.rs`

## Purpose

Read-oriented orchestration surface for a DARE DAG: print runtime status, list the next ready tasks (with composed prompts), or watch status in a loop. Uses `ensure_state` + canvas refresh for `--status` / `--next`. **Does not** start tasks (`transition(Start)`), complete/fail/reset, or run Ralph (**029+**).

## Command

```bash
dare execute [--status|--next|--watch] [--dag PATH] [--interval SECS] [--max-ticks N]
# global flags:
dare execute --json --no-color
```

| Flag | Default | Effect |
|------|---------|--------|
| *(none)* / `--status` | status | Snapshot counts + canvas path; ensure state + write `DARE/.canvas.md` |
| `--next` | — | Ready tasks at **min rank** (id lexico); composed prompts; **no** Start |
| `--watch` | — | Poll status; **zero writes** to `.dare/state.json` and canvas |
| `--dag` | `DARE/dare-dag.yaml` | DAG path (project jail via `resolve_project_rel`) |
| `--interval` | `2` | Watch poll seconds (`0` allowed; smokes use `--max-ticks 1`) |
| `--max-ticks` | unlimited | Stop watch after N ticks (CI / smoke) |

`--status`, `--next`, and `--watch` are mutually exclusive (clap conflicts → exit **2**).

## Canonical messages

| Constant | Exact string |
|----------|----------------|
| `MSG_EMPTY` | `Empty DAG — no tasks.` |
| `MSG_RESOLVED` | `✅ All tasks resolved.` |
| `MSG_BLOCKED` | `Blocked — no executable tasks` |

## Outcomes (`data.outcome`)

| Outcome | When |
|---------|------|
| `status` | `--status` / watch tick snapshot |
| `ready` | `--next` with ≥1 ready at min rank |
| `resolved` | no PENDING/RUNNING left |
| `blocked` | 0 ready and ≥1 PENDING (domain; after `ensure_state` cascade, FAILED parents usually yield SKIPPED children → often `resolved`) |
| `waiting` | 0 ready and ≥1 RUNNING |
| `empty` | DAG has zero tasks |

## Exit codes

| Code | When |
|------|------|
| 0 | OK — including empty / resolved / blocked / waiting |
| 2 | Usage (exclusive flags, clap) |
| 3 | DAG NotFound |
| 4 | InvalidInput / Config / cycle / missing dep / jail / no root |
| 5 | Io (lock / write state or canvas) |

## Mutations

| Action | `.dare/state.json` | `DARE/.canvas.md` | `transition(Start)` |
|--------|--------------------|-------------------|---------------------|
| `--status` | `ensure_state` (merge + cascade + save) | write | **No** |
| `--next` | same | write | **No** |
| `--watch` | soft-load only | **no write** | **No** |

## `--json` shapes (excerpt)

**Status / watch tick**

```json
{
  "action": "status",
  "outcome": "status",
  "dag": "DARE/dare-dag.yaml",
  "canvasPath": "DARE/.canvas.md",
  "counts": { "done": 0, "running": 0, "pending": 1, "failed": 0, "skipped": 0, "total": 1 },
  "tasks": [{ "id": "task-001", "title": "…", "status": "PENDING", "rank": 0, "complexity": "LOW" }]
}
```

**Next**

```json
{
  "action": "next",
  "outcome": "ready",
  "dag": "DARE/dare-dag.yaml",
  "rank": 0,
  "ready": [{ "id": "task-001", "title": "…", "rank": 0, "complexity": "LOW", "specFile": "…", "prompt": "…" }],
  "blocked": false,
  "resolved": false
}
```

With `--json --watch`, one success envelope is emitted **per tick**.

## Out of scope (029+)

- `--complete` / `--fail` / `--reset`
- Ralph Loop gates
- `--agent` / worktrees / budget (030+)
- Real agent drivers (031+)

## Container

```bash
docker compose -f docker-compose.ci.yml config
```

Verified exit 0 in mp028-001 — no waiver; no new image for execute status/next/watch.

## Local verify

```bash
dare execute --status --dag DARE/dare-dag-028.yaml
dare execute --next --dag DARE/dare-dag-028.yaml
dare execute --watch --max-ticks 1 --interval 0
cargo test -p dare-dag -- execution
cargo test -p dare-cli --test cli_smoke -- execute
```

## Related

- Decision log: **DEC-029**
- Runtime ranks/state: [`dag-runtime.md`](dag-runtime.md) (DEC-027)
- DAG viz: [`cli-dag-viz.md`](cli-dag-viz.md) (DEC-028)
- Capability matrix: `dare-execute` → `cli_commands: ["execute"]`
