# CLI init and bootstrap (`dare init` / `dare bootstrap`)

> **DEC-048** · Microplano 047 · Source: `crates/dare-cli/src/commands/init.rs`, `bootstrap.rs` · Library: [`scaffold-contracts.md`](scaffold-contracts.md) (DEC-047)

## Purpose

Greenfield project creation and idempotent re-application of scaffold artefacts on an existing DARE project.

| Command | Role |
|---------|------|
| `dare init` | Create `{dir}/{name}`: stack scaffold + `dare.config.json` + **four** IDE harnesses (claude, cursor, codex, antigravity) |
| `dare bootstrap` | Re-run scaffold from existing `dare.config.json`; **does not** reinstall harnesses |

## Commands / flags

### `dare init`

```bash
dare init [NAME]
  [--stack <ID>] [--mcp <LANG>] [--fullstack <react|vue>]
  [--transport <stdio|http|sse>] [--toolchain <none|docker>]
  [--non-interactive] [--force] [--check]
  [-d|--dir <PATH>]
  [--json] [--no-color]
```

| Flag | Effect |
|------|--------|
| `NAME` | Project name (directory under parent); required in `--non-interactive` |
| `--stack <ID>` | Backend stack id; alias `rails` → `ruby-rails-8` (mutually exclusive with `--mcp`) |
| `--mcp <LANG>` | MCP language alias → canonical stack id (see table below) |
| `--fullstack <react\|vue>` | Backend only; embeds companion under `frontend/` |
| `--transport` | MCP stacks only (`stdio`, `http`, `sse`) |
| `--toolchain` | `none` (default) or `docker` |
| `--non-interactive` | Skip dialoguer prompts; requires `NAME` + (`--stack` or `--mcp`) |
| `--force` | Overwrite when target directory or scaffold paths exist |
| `--check` | Plan/report only — **zero writes** (no directory creation) |
| `-d` / `--dir` | Parent directory (default: cwd); target = `{dir}/{NAME}` |

**Interactive mode:** when `--non-interactive` is omitted, missing flags are collected via `dialoguer`. Non-TTY stdin without `--non-interactive` → InvalidInput exit **4**: `interactive mode requires a TTY (use --non-interactive)`.

#### `--mcp` language map (case-insensitive)

| Input | Stack id |
|-------|----------|
| `ts`, `node`, `typescript`, `mcp-node-ts` | `mcp-node-ts` |
| `python`, `py`, `mcp-python` | `mcp-python` |
| `rust`, `mcp-rust` | `mcp-rust` |
| `go`, `mcp-go` | `mcp-go` |
| other | InvalidInput `unknown mcp language: {input}` |

### `dare bootstrap`

```bash
dare bootstrap [--force] [--toolchain <none|docker>] [--check] [-d|--dir <PATH>] [--json]
```

| Flag | Effect |
|------|--------|
| `--force` | Replace existing scaffold files (`ConflictPolicy` → Replace) |
| `--toolchain` | Override config toolchain; persisted on disk when not `--check` |
| `--check` | Plan only — zero writes |
| `-d` / `--dir` | Project root (default: cwd) |

## ConflictPolicy and SkipExisting

| Context | Policy | Behavior |
|---------|--------|----------|
| `dare init` (default) | `FailFast` | Existing planned path without `--force` → InvalidInput |
| `dare init --force` | Replace | Backups + overwrite via scaffold journal |
| `dare bootstrap` (default) | **`SkipExisting`** | Present files → `skipped`; missing → `created` |
| `dare bootstrap --force` | Replace | Same as init force |

Second bootstrap on an unchanged tree: `created=[]`, `skipped` non-empty (idempotent).

## Fullstack `frontend/`

Requires `--stack` on a **backend** stack (not MCP). Templates from `assets/stacks/_frontend/{react,vue}/`:

| Path | Content |
|------|---------|
| `frontend/package.json` | name `{project_name}-web`, `private: true` |
| `frontend/src/main.tsx` (react) / `frontend/src/main.ts` (vue) | stub entry |
| `frontend/README.md` | companion line + stack id |

`dare.config.json` includes `"frontend": "react"` or `"vue"` when set.

## Harnesses ×4 (init only)

Always installed on successful init (order in report ASC):

1. **antigravity** — `.antigravityrules`, workflows, commands
2. **claude** — `CLAUDE.md`, `.claude/commands`
3. **codex** — `AGENTS.md`, `.codex/skills`
4. **cursor** — `.cursorrules`, `.cursor/commands`

Bootstrap **never** re-runs harness install (scope minimal).

## Init pipeline (side effects, `!check`)

1. `create_dir_all(target)`
2. `run_scaffold` (`FailFast` or force)
3. Atomic write `dare.config.json` (`schemaVersion` 1)
4. Install four harnesses (`force` from init flags)
5. `validate_stack_output` — must `ok`

On failure after directory creation: scaffold rollback + `remove_dir_all(target)` best-effort.

## Reports (schemaVersion 1, camelCase)

### InitReport

`schemaVersion`, `mode` (`"init"`), `projectRoot`, `projectName`, `stackId`, `frontend`, `toolchain`, `transport`, `created`, `replaced`, `skipped`, `harnessesInstalled`, `rolledBack`, `check`.

Human line: `init: {projectName} stack={stackId} check={check}`.

### BootstrapReport

`schemaVersion`, `mode` (`"bootstrap"`), `projectRoot`, `stackId`, `toolchain`, `created`, `replaced`, `skipped`, `rolledBack`, `check`.

Human line: `bootstrap: stack={stackId} check={check}`.

## Exit codes (DEC-005)

| Code | When |
|------|------|
| 0 | Ok (including `--check`) |
| 1 | Internal / severe rollback failure |
| 2 | Usage (e.g. `--stack` + `--mcp`, incomplete `--non-interactive`) |
| 3 | NotFound (`dare.config.json` missing on bootstrap) |
| 4 | InvalidInput (validation, path exists, TTY, unknown stack/frontend/mcp) |
| 5 | Io |

## Edge cases

| Case | Result |
|------|--------|
| `--non-interactive` without name | Usage 2 |
| `--non-interactive` without `--stack`/`--mcp` | Usage 2 |
| `--stack` + `--mcp` | Usage 2 |
| `--fullstack` without `--stack` | InvalidInput 4 |
| `--fullstack` + MCP | InvalidInput 4 |
| backend + `--transport` | InvalidInput 4 |
| target exists, no `--force` | InvalidInput `target directory already exists: {name}` |
| `init --check` | zero writes; `created=[]` |
| bootstrap ×2 without `--force` | second run `created=[]` |
| bootstrap `--force` | `replaced` non-empty; toolchain override persisted |

## Examples

```bash
dare init demo-app --stack rust-axum --non-interactive --json
dare init mcp-svc --mcp ts --transport stdio --non-interactive
dare init shop --stack node-nestjs --fullstack react --non-interactive
dare init demo-app --stack go-gin --check --non-interactive
cd demo-app && dare bootstrap && dare bootstrap
dare bootstrap --force --toolchain docker
```

## Diff vs TypeScript 3.18.1

| Area | Class | Note |
|------|-------|------|
| inquirer → dialoguer | B | Stable flags; TTY gate |
| Exit codes (DEC-005) | B | Rust table above |
| Bootstrap SkipExisting | B/C | TS may differ on idempotent re-apply |
| Init 4 harnesses always | B | vs detect-only legacy paths |
| Paths `frontend/` | C | Frozen layout |
| `--check` on init/bootstrap | C | Agent/CI extension |

## Ralph (047 close)

```bash
cargo test -p dare-cli -p dare-scaffold
cargo clippy -p dare-cli -p dare-scaffold --all-targets -- -D warnings
cargo audit
```

## Related

- **[DEC-048](../DECISION-LOG.md)** — init/bootstrap CLI contracts
- **[DEC-047](../DECISION-LOG.md)** / [`scaffold-contracts.md`](scaffold-contracts.md) — `dare-scaffold` library
- [`cli-output-and-errors.md`](cli-output-and-errors.md) — global JSON envelope and exit codes
