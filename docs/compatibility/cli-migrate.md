# CLI migrate (`dare migrate`)

> **DEC-044** · Microplano 039 · Source: `crates/dare-project/src/migrate.rs` + `crates/dare-cli/src/commands/migrate.rs`

## Purpose

Brownfield **Fase 2** migration planning: compare origin stacks vs a closed `--to` allowlist, emit a phased plan under `DARE/MIGRATION/**`, and write **Gherkin parity skeletons** per reverse module.

The CLI does **not** rewrite application source. Semantic strategy / paradigm / cutover text and real parity scenarios are filled by the IDE skill `/dare-migrate` (or optional soft-fail `--ai` on `MIGRATION.md` AGENT sections).

## Pre-requisite

`dare reverse` must have produced:

- `DARE/IDEIA.md`
- modules via `DARE/REVERSE/reverse-facts.json` and/or `DARE/REVERSE/module-*.md`

Missing IDEIA or zero modules → exit **4** (`InvalidInput`) with message to run reverse first.

Optional inputs (warnings / gaps if absent): `DARE/PROJECT-DNA.md`, `DARE/PATTERNS.md`.

## Commands

```bash
dare migrate --to <stack> [-d <path>]
dare migrate --to <stack> --check [-d <path>]
dare migrate --to <stack> --ai [--provider mock|codex|…]
dare migrate --to <stack> --json [--no-color]
```

Global: `--json` / `--no-color` (004 output renderer).

| Flag | Default | Effect |
|------|---------|--------|
| `--to <stack>` | **required** | Target stack id; closed allowlist (case-sensitive) |
| `-d` / `--dir` | cwd | Start path (walk-up to project root) |
| `--check` | off | Plan only — **zero writes** under `DARE/MIGRATION/**` |
| `--ai` | off | Soft-fail enrichment of `MIGRATION.md` AGENT sections (after write) |
| `--provider` | `codex` | Requires `--ai`; otherwise Usage exit **2** |
| `--json` | off | Envelope ADR-002; `data` = MigrateReport schema 1 |
| `--no-color` | off | Disable ANSI (also honors `NO_COLOR`) |

### `--to` allowlist (frozen)

`node-nestjs`, `python-fastapi`, `php-laravel`, `go-gin`, `go-stdlib`, `rails`, `rust-axum`, `rust`, `rust-leptos`, `rust-leptos-csr`, `react`, `vue`, `mcp-node-ts`.

Unknown / empty → `InvalidInput` exit **4**.

## Artifacts (`DARE/MIGRATION/**` only)

Non-check runs write **only** under `DARE/MIGRATION/` (path jail). No source rewrite, no `dare.config.json` mutation, no graph schema change.

| Path | Notes |
|------|-------|
| `DARE/MIGRATION/MIGRATION.md` | Summary, phases, blocking gaps + `<!-- AGENT -->` sections |
| `DARE/MIGRATION/migration-facts.json` | Same payload as report (`schemaVersion` **1**, camelCase) |
| `DARE/MIGRATION/parity/<moduleId>.feature` | **Skeleton** Gherkin per module (`@skeleton`; fill via `/dare-migrate`) |

### Gherkin = skeleton only

Each `.feature` is a placeholder Scenario (`Given`/`When`/`Then` generic). Observable flows and edge cases come from legacy behavior via `/dare-migrate` — the CLI must not invent business steps.

## MigrateReport / migration-facts (schemaVersion 1)

CamelCase fields:

| Field | Type | Notes |
|-------|------|-------|
| `schemaVersion` | number | Always `1` |
| `mode` | string | `"check"` or `"write"` |
| `fromStacks` | string[] | Detected origin stack ids |
| `toStack` | string | Allowlist target |
| `toFamily` | string | Family of target (`node`, `rust`, …) |
| `comparison` | string | `same_family` \| `cross_stack` \| `unknown_origin` |
| `phases` | `MigrationPhase[]` | `foundations` → `modules` → `cutover` |
| `blockingGaps` | `BlockingGap[]` | Sorted blocking then warning by `id` |
| `moduleIds` | string[] | Sorted; max 64 |
| `written` | string[] | Relative POSIX paths; empty in check |
| `warnings` | string[] | Soft notes (missing DNA/patterns, AI skip, …) |

`--check` human output ends with:

```text
mode: check (zero mutations)
```

## Exit codes (004 map)

| Code | Kind | When |
|------|------|------|
| 0 | — | Success (incl. check / AI soft-fail) |
| 2 | Usage | Clap / `--provider` without `--ai` |
| 3 | NotFound | `-d` missing directory |
| 4 | InvalidInput | No project root / bad `--to` / missing reverse |
| 5 | Io | Filesystem errors |

## Distinction: three different “migrate” surfaces

| Surface | Microplano | What it does |
|---------|------------|--------------|
| **`dare migrate --to`** (this doc) | **039** | Brownfield **reimplementation plan** + parity skeletons → `DARE/MIGRATION/**` |
| **Config migrate** (`dare-config`) | **008** | Evolve **`dare.config.json`** schema (`plan_migrate` / dry-run / apply); see [`config-and-migrations.md`](config-and-migrations.md) |
| **`KnowledgeGraph::migrate` / graph open path** | **040** (+ CLI graph **041**) | Apply **graph store schema** migrations (SQLite/JSON); never confuses with brownfield stack migrate; see [`graphrag-storage.md`](graphrag-storage.md) |

## Security

- Writes jailed to `DARE/MIGRATION/**` via `SafeRelativePath` + `atomic_write`
- Allowlist rejects arbitrary `--to` strings
- Evidence / gap detail / warnings passed through `redact`
- `--ai` soft-fail: provider/parse/inject errors become warnings, exit **0** (non-corrupt plan remains)
- No shell concatenation

## Compatibility notes

| Diff | Class | Note |
|------|-------|------|
| Writes only plan artifacts (no code rewrite) | **A** | Acceptance MUST |
| Gherkin skeleton; skill fills behavior | **A** | Same contract as reverse AGENT markers |
| Closed `--to` allowlist | **A** | Fail closed |
| Soft-fail `--ai` | **A** | Same pattern as reverse/blueprint |
| Distinct from config/graph migrate | **A** | Naming collision documented |

## Ralph

```bash
cargo fmt --check
cargo clippy -p dare-project -p dare-cli --all-targets -- -D warnings
cargo test -p dare-project migrate
cargo test -p dare-cli --test cli_smoke migrate
```
