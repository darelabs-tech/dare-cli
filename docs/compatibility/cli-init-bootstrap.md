# CLI init and bootstrap (`dare init` / `dare bootstrap`)

> **DEC-058** (supersedes DEC-048) · Cycle `rust-init-npm-parity-011` · Source:
> `crates/dare-cli/src/commands/{init,init_prompts,bootstrap}.rs`,
> `crates/dare-scaffold/src/project/` · Reference: npm `@dewtech/dare-cli@3.18.1`

## Contract

`dare init` in Rust shows **the same menus** and generates **the same scaffold**
as the npm `dare init` 3.18.1 for the same answers or flags. The only
differences are the bug fixes B-01..B-10 below and the Rust-only additions.

Parity is enforced by `crates/dare-cli/src/commands/init_npm_parity_tests.rs`
against goldens recorded from the published npm package
(`crates/dare-cli/tests/fixtures/golden/init-npm/`, 27 cases — see its README).
Each case compares the question tree, resolved config, file tree (incl. empty
dirs), every file hash, code-generated file contents, external commands and
console output.

## Interactive flow

Same questions, order, conditions, labels, values and defaults as npm
`commands/init.ts`:

1. `Project name:` (only without a positional name; default `my-dare-project`)
2. `Project structure:` — Monorepo / Backend only / Frontend only / MVC / MCP Server
3. `MVC framework:` (mvc) — Rails 8 / Laravel
4. `MCP server language:`, `Transport type:`, `MCP capabilities to include:` (mcp-server)
5. `Backend stack:` (monorepo, backend) — 7 stacks
6. `Frontend stack:` (monorepo, frontend) — React / Vue / Leptos SSR / Leptos CSR / None
7. `Cargo workspace layout:` + `Short crate prefix` (monorepo rust-axum + Leptos)
8. `Primary IDE / Agent:` — 6 options
9. `GraphRAG backend:` — SQLite / JSON / Neo4j
10. `Enable DARE MCP Server for context queries?` (default yes)
11. `Toolchain for scaffolding …` — auto / native / docker (default auto)

Interactive mode needs a TTY; otherwise use the non-interactive flags.

## Non-interactive flags (npm-identical)

```bash
dare init [NAME] [--stack <id>] [--fullstack] [--mcp <node-ts|python|rust|go>]
          [--transport <stdio|sse|http>] [--toolchain <auto|native|docker>]
          [--non-interactive]
```

- Triggered by `--non-interactive`, `--stack` or `--mcp`. Defaults: name
  `my-dare-project`, `ide=cursor`, `graphrag=sqlite`, `mcp=false`, toolchain `auto`.
- `--fullstack` (flag) only with `ruby-rails-8` / `php-laravel` → structure `mvc`.
- Error texts match npm (`Error: unknown stack '…'. Valid: …`, etc.).

Rust-only additions: `-d/--dir <parent>`, `--force` (reuse a non-empty target),
`--check` (resolve + plan, zero writes), global `--json` / `--no-color`, the
`rails` stack alias, the Rust 4.0 `--mcp` aliases (`ts`, `node`, `py`, …), and
validation of `--transport` / `--toolchain` values (npm passes unknown values
through).

**Breaking vs Rust 4.0:** `--fullstack react|vue` is rejected with a message
pointing to the interactive Monorepo option; `--toolchain none` is gone
(use `auto|native|docker`).

## Layout

| Structure | Backend | Frontend |
|---|---|---|
| backend / mvc / mcp-server | project root | — |
| frontend | — | project root |
| monorepo | `backend/` | `frontend/` |
| monorepo rust-axum + Leptos | `crates/server` or `crates/<prefix>-server` | `crates/web` or `crates/<prefix>-web` (+ workspace root, `-core`/`-cli` in multi) |

Scaffolds: registry templates from npm `templates/stacks/<id>/` (embedded in
`assets/npm/`, synced with `scripts/sync-npm-templates.py`); Rails runs
`rails new` (native / `ruby:3.3` Docker) + DARE overlay, or the offline
templates; Vite runs `npx degit … template-<react|vue>-ts` + `npm install`
(native / `node:20-alpine`); Leptos writes the crate and runs `cargo fetch`
(native / `ghcr.io/dewtech-technologies/dare-rust-leptos:1`).

Common files: `DARE/README.md`, `DARE/EXECUTION/`, `dare.config.json` (npm
schema and key order; `version` = Rust binary version), `.gitignore` merge,
`templates/` (6 method templates), `dare-graph.yml`, and IDE files **only for
the chosen IDE**. No `Dockerfile` / `docker-compose.yml` at init — containers
are produced in blueprint phase 1 (`/dare-docker`), as in npm.

## Bug fixes vs npm 3.18.1

| # | npm 3.18.1 | Rust |
|---|---|---|
| B-01 | `claude-hybrid` installs only Claude files | Claude **and** Cursor |
| B-02 | MCP `.claude/settings.json` always uses python commands | MCP language's commands |
| B-03 | Mojibake (`âœ…`, `â†’`, `â€”`) in generated text | UTF-8 (`✅`, `→`, `—`) |
| B-04 | `hasCommand` misses `.cmd` shims on Windows → `auto` always uses Docker | PATH + PATHEXT lookup |
| B-05 | Frontend `none` writes `skill-none.mdc` | not written |
| B-06 | MCP rust/go next steps print npm commands | cargo / go commands |
| B-07 | Multi-crate next steps show the name slug | show the crate prefix |
| B-08 | Empty `.cargo/` inside Leptos monorepo members | not created |
| B-09 | (Rust 4.0) failed `init --force` deleted the pre-existing dir | rollback removes only created paths |
| B-10 | `Short crate prefix` default is always `app` with a positional name | initials of the project name |

## `dare bootstrap`

- Configs with `structure` (created by this init) follow npm `bootstrap.ts`:
  refuses when `vendor`, `node_modules`, lockfiles, `Cargo.lock` or `target`
  exist (unless `--force`), honors `--toolchain auto|native|docker`, re-runs the
  backend / frontend / MCP bootstrap, never touches DARE artifacts.
- Configs with `stack` (created by Rust 4.0) keep the microplano 047 skeleton
  re-apply (`SkipExisting`, `--toolchain none|docker`).
- `--check` reports the plan without writing.

## Reports (`--json`, schemaVersion 1)

`init`: `mode`, `check`, `projectRoot`, `projectName`, `structure`, `backend`,
`frontend`, `mcpLanguage`, `mcpTransport`, `ide`, `graphrag`, `mcp`,
`toolchain`, `created` (relative paths). `bootstrap` (npm path): `mode`,
`projectRoot`, `structure`, `backend`, `frontend`, `mcpLanguage`, `toolchain`,
`check`. Under `--json`, live progress goes to stderr.
