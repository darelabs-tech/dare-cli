# CLI execute agent (`dare execute --agent`)

> **DEC-031** · Microplano 030 · Source: `crates/dare-agent` + `crates/dare-cli/src/commands/execute_agent.rs`

## Purpose

Run the autonomous agent loop with a **mock/noop** driver, isolated git worktrees, token budget, and **fixed** retry policy. On Done, optionally chain Ralph/`--complete` (029).

Complements [`cli-execute-status.md`](cli-execute-status.md) (DEC-029) and [`cli-execute-mutations.md`](cli-execute-mutations.md) (DEC-030).

## Commands

```bash
dare execute --agent [--driver mock|noop] [--task ID] [--budget-tokens N] [--policy fixed] [--dag PATH]
dare execute --cleanup-worktrees
```

Flags are mutually exclusive with `--status` / `--next` / `--watch` / `--complete` / `--fail` / `--reset` (and each other). Global: `--json` / `--no-color`.

| Flag | Default | Effect |
|------|---------|--------|
| `--agent` | — | Run agent loop |
| `--driver` | `mock` | `mock` / `noop` only; others → exit **4** `driver not implemented` |
| `--task` | first ready | Task id; missing → exit **3** |
| `--budget-tokens` | `0` | `0` = unlimited; finite budget exhaust → exit **1** |
| `--policy` | `fixed` | Only `fixed`; `decay` → exit **4** |
| `--cleanup-worktrees` | — | Remove orphans under `.dare/agent-worktrees/` |
| `--dag` | `DARE/dare-dag.yaml` | Project jail (ignored for cleanup path selection of root) |

## Env

| Var | Values | Effect |
|-----|--------|--------|
| `DARE_AGENT_MOCK` | `success` (default) / `fail` / `timeout` | Mock driver mode |
| `DARE_AGENT_SKIP_RALPH` | `1` / `true` | On Done, skip Ralph/complete (exit 0) |
| `DARE_RALPH_MOCK` | `pass` / `fail` / `timeout` | Used when Ralph runs after Done (029) |

## Worktrees

| Item | Value |
|------|-------|
| Root | `.dare/agent-worktrees/` |
| Branch | `dare/agent-{taskId}-{attempt}` |
| Rel path | `.dare/agent-worktrees/{taskId}-{attempt}` |

Requires a git repository (`.git`). Auto-cleanup after each attempt; orphans removable via `--cleanup-worktrees`.

## Policy `fixed`

| Agent status | Decision |
|--------------|----------|
| Success | **Done** → Ralph/complete unless `DARE_AGENT_SKIP_RALPH` |
| Failure | **Continue** if `attempt < 5`, else **Stop** |
| Timeout / Cancelled | **Stop** |

`MAX_AGENT_ATTEMPTS = 5`.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Done (+ Ralph ok) / empty ready / cleanup ok |
| 1 | Stop / budget exhausted / Ralph gate fail / internal |
| 2 | Usage (exclusive flags) |
| 3 | Task not found |
| 4 | No git / unknown driver / bad policy / invalid input |
| 5 | (reserved / IO per 004) |
| 124 | Agent mock timeout **or** Ralph timeout |
| **6** | **Not used** (guard deferred to 034) |

## JSON (`--json`)

Success / report envelope `data.action`:

- `agent` — fields: `taskId`, `driver`, `policy`, `decision`, `attempts`, `budget`, `worktreePath`, `result`, `ralphSkipped`
- `cleanup-worktrees` — `{ action, removed }`

## Local verify

```bash
cargo test -p dare-agent
cargo test -p dare-cli --test cli_smoke -- execute_agent
```

## Out of scope (030)

- Real drivers (**031**)
- Decay / REPLAN (**033**)
- Guard exit 6 (**034**)
- Review in loop (**032**)
