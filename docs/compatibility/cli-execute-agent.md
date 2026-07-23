# CLI execute agent (`dare execute --agent`)

> **DEC-031** · **DEC-037** · Microplanos 030–031 · Source: `crates/dare-agent` + `crates/dare-cli/src/commands/execute_agent.rs`

## Purpose

Run the autonomous agent loop with a **mock/noop** or **real CLI** driver (`codex` / `claude` / `cursor` / `antigravity`), isolated git worktrees, token budget, and **fixed** retry policy. On Done, optionally chain Ralph/`--complete` (029). Guard preflight (034) runs before the loop.

Complements [`cli-execute-status.md`](cli-execute-status.md) (DEC-029) and [`cli-execute-mutations.md`](cli-execute-mutations.md) (DEC-030).

## Commands

```bash
dare execute --agent [--driver mock|noop|codex|claude|cursor|antigravity] \
  [--task ID] [--budget-tokens N] [--policy fixed] [--dag PATH]
dare execute --cleanup-worktrees
```

Flags are mutually exclusive with `--status` / `--next` / `--watch` / `--complete` / `--fail` / `--reset` (and each other). Global: `--json` / `--no-color`.

| Flag | Default | Effect |
|------|---------|--------|
| `--agent` | — | Run agent loop |
| `--driver` | `mock` | `mock` \| `noop` \| `codex` \| `claude` \| `cursor` \| `antigravity`; unknown → exit **4** `driver not implemented` |
| `--task` | first ready | Task id; missing → exit **3** |
| `--budget-tokens` | `0` | `0` = unlimited; finite budget exhaust → exit **1** |
| `--policy` | `fixed` | Only `fixed`; `decay` → exit **4** |
| `--cleanup-worktrees` | — | Remove orphans under `.dare/agent-worktrees/` |
| `--dag` | `DARE/dare-dag.yaml` | Project jail (ignored for cleanup path selection of root) |

## Drivers

| Id (`--driver`) | Kind | Runtime |
|-----------------|------|---------|
| `mock` | Test | In-process; modes via `DARE_AGENT_MOCK` |
| `noop` | Test | Always success, empty summary |
| `codex` | Real | Codex CLI JSONL (`codex exec --json …`) |
| `claude` | Real | Claude Code CLI (text; **not** Anthropic SDK) |
| `cursor` | Real | Cursor Agent CLI (`cursor-agent --print`) |
| `antigravity` | Real | Antigravity CLI (`antigravity agent --print`) |

### Agent driver ids vs `dare-ai` ProviderId

Agent `--driver` ids are **not** the same as `dare design --ai --provider` / `dare-ai` `ProviderId` (DEC-025). Same host tools, different namespaces (Classe B):

| Agent `--driver` | `dare-ai` ProviderId | Notes |
|------------------|----------------------|-------|
| `codex` | `codex` | Same string; distinct crates (`dare-agent` vs `dare-ai`) |
| `claude` | `claude-code` | **≠** — agent short id vs enrich provider id |
| `cursor` | `cursor-cli` | **≠** |
| `antigravity` | `antigravity-cli` | **≠** |
| `mock` / `noop` | `mock` (enrich only) | Agent has both `mock` and `noop`; enrich has `mock` only |

## Env

| Var | Values / format | Effect |
|-----|-----------------|--------|
| `DARE_AGENT_MOCK` | `success` (default) / `fail` / `timeout` | Mock driver mode |
| `DARE_AGENT_SKIP_RALPH` | `1` / `true` | On Done, skip Ralph/complete (exit 0) |
| `DARE_RALPH_MOCK` | `pass` / `fail` / `timeout` | Used when Ralph runs after Done (029) |
| `DARE_CODEX_COMMAND` | whitespace-split argv (no shell) | Full override of Codex program+args |
| `DARE_CLAUDE_COMMAND` | whitespace-split argv (no shell) | Full override of Claude program+args |
| `DARE_CURSOR_COMMAND` | whitespace-split argv (no shell) | Full override of Cursor program+args |
| `DARE_ANTIGRAVITY_COMMAND` | whitespace-split argv (no shell) | Full override of Antigravity program+args |

Env **absent** → default argv below. Env **present** → replaces program and args entirely. Empty / whitespace-only override → InvalidInput exit **4** (`command override must not be empty`). Same variable names as enrich (024); parsed inside `dare-agent` (no crate dep on `dare-ai`).

### Default argv (real drivers)

| Driver | Program | Default args |
|--------|---------|--------------|
| `codex` | `codex` | `exec --json --sandbox read-only --ask-for-approval never` |
| `claude` | `claude` | `-p --output-format text` |
| `cursor` | `cursor-agent` | `--print` |
| `antigravity` | `antigravity` | `agent --print` |

- Prompt is passed on **stdin** (UTF-8).
- Timeout per run: **20 min** (`AGENT_DRIVER_TIMEOUT`); maps to `AgentRunStatus::Timeout` → CLI exit **124**.
- Codex default sandbox is **read-only**; no TTY `--require-approval` in this cycle.
- `doctor` reports `ok=false` when the executable is missing (does not abort the process). `run` with missing exe → internal `"executable not found: {program}"` → CLI exit **1**.

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
| 1 | Stop / budget exhausted / Ralph gate fail / **executable not found** / internal |
| 2 | Usage (exclusive flags) |
| 3 | Task not found |
| 4 | No git / unknown driver / bad policy / invalid input (incl. empty `DARE_*_COMMAND`) |
| 5 | (reserved / IO per 004) |
| **6** | Guard FAIL (preflight 034) |
| 124 | Driver / mock timeout **or** Ralph timeout |

## JSON (`--json`)

Success / report envelope `data.action`:

- `agent` — fields: `taskId`, `driver`, `policy`, `decision`, `attempts`, `budget`, `worktreePath`, `result`, `ralphSkipped`
- `cleanup-worktrees` — `{ action, removed }`

## CI / compose

Microplano 031 reuses the existing CI compose (`docker-compose.ci.yml` + `Dockerfile.rust`) — no new image. **mp031-001** validated `docker compose -f docker-compose.ci.yml config` (exit 0); no waiver required. Smokes use fake binary / env override (no real Codex/Claude/Cursor/Antigravity CLIs in CI).

## Local verify

```bash
cargo test -p dare-agent
cargo test -p dare-cli --test cli_smoke -- execute_agent
```

## Out of scope

- Decay / REPLAN (**033**)
- Review in loop (**032** already separate CLI)
- Anthropic SDK / Claude API directa
- Best-of-N / approval TTY (**049**)
- `dare ai` (**050**)
