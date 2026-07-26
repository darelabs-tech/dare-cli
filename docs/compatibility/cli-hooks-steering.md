# CLI hooks and steering (`dare hooks` / `dare steering`)

> **DEC-049** · Microplano 048 · Libraries: `crates/dare-hooks`, `crates/dare-steering` · CLI: `commands/hooks.rs`, `commands/steering.rs`

## Purpose

Deterministic, non-LLM hooks with a trust gate, and read-only steering resolution by scope/glob/priority.

| Command | Role |
|---------|------|
| `dare hooks list` | List event→actions from embed or `.dare/hooks.yml` |
| `dare hooks run <event>` | Execute allowlisted actions (requires trust) |
| `dare hooks validate` | Validate defs; zero writes |
| `dare steering list` | List DNA / PATTERNS / `.dare/steering/*.md` |
| `dare steering show <file>` | Blocks applicable to a project-relative path |

## Hook events (closed)

| Event | Typical use |
|-------|-------------|
| `on-save` | PostToolUse / save |
| `on-file-create` | New file |
| `on-task-complete` | Task DONE |
| `pre-commit` | Git hook |

Unknown event → exit **2**.

## Allowlist actions + spawn

Program is always `current_exe` (the running `dare` binary). No shell concat.

| Action id | argv |
|-----------|------|
| `dare-validate` | `validate` |
| `dare-review` | `review` |
| `graph-register` | `graph ingest` |
| `lint` | `guard` *(Class B vs TS stack linter)* |
| `test` | `info` *(Class B smoke)* |

Default SoT: embedded `assets/hooks/default-hooks.yml`. Optional overlay `.dare/hooks.yml` **replaces** the hooks array (schemaVersion must be `1`).

## Trust gate

| Source | Effect |
|--------|--------|
| Default | `hooks.trusted = false` |
| `dare.config.json` → `hooks.trusted: true` | Allows `run` without flag |
| CLI `--trust` | Allows `run` for this invocation |
| Both absent/false | `run` does **not** execute → exit **2** + `HOOKS_TRUST` |
| `hooks.enabled: false` | `run` → exit **2** + `HOOKS_DISABLED` |

## Idempotency

SHA-256 of canonical JSON (`schemaVersion`, `event`, `action`, `file`, `task`). Markers under `.dare/hooks-idempotency/{hash}.ok` (cap **512**, prune by mtime). Re-run with same digest → `skipped` / `idempotent` (no duplicate spawn).

## Exit codes (hooks)

| Code | When |
|------|------|
| 0 | list/validate ok; run all ok/skipped |
| 1 | Internal / action spawn failed |
| 2 | Unknown event, `HOOKS_TRUST`, `HOOKS_DISABLED`, usage |
| 3 | NotFound |
| 4 | InvalidInput / bad overlay (`validate` `ok=false`) |
| 5 | Io |

## Steering

Sources (when present): `DARE/PROJECT-DNA.md` (priority 0), `DARE/PATTERNS.md` (priority 1), `.dare/steering/*.md` (frontmatter `scope` / `glob` / `priority`).

Sort: priority ASC, path ASC (posix).

**Security:** basename `.env` or `.env.*` is never eligible → exit **4** (`steering target excluded: .env* paths are not eligible`) before read.

## Examples

```bash
dare hooks list --json
dare hooks validate --json
dare hooks run on-save --file src/lib.rs          # exit 2 without trust
dare hooks run on-save --file src/lib.rs --trust
dare steering list --json
dare steering show crates/dare-core/src/lib.rs --json
dare steering show .env                           # exit 4
```

## Out of scope (048)

Bench / mutation (**049**), MCP `GET /steering` (**051/052**), native Cursor IDE hooks.
