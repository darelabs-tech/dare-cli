# CLI patterns (`dare patterns`)

> **DEC-041** · Microplano 038 · Source: `crates/dare-project/src/patterns.rs` + `crates/dare-cli/src/commands/patterns.rs`

## Purpose

Mine deterministic recurring patterns (frequency + co-occurrence) and materialize:

- `DARE/PATTERNS.md` — human markdown + `<!-- AGENT -->` sections
- `DARE/patterns-facts.json` — machine-readable facts (schemaVersion 1)

## Command / flags

```bash
dare patterns [-d|--dir <path>] [--check] [--modules <csv>] [--inject] [--ast]
dare patterns --check --json [-d <path>]
```

| Flag | Effect |
|------|--------|
| `-d` / `--dir <path>` | Start directory (default: cwd); walk-up project root |
| `--check` | Mine only — **zero filesystem mutations** |
| `--modules <csv>` | Filter to module ids (`crates/*`, `src`, …) |
| `--inject` | When rewriting `PATTERNS.md`, preserve existing AGENT section bodies |
| `--ast` | Optional AST sample via `dare-ast` (call-idiom; caps 32×512 KiB) |
| `--json` | Envelope via output renderer (004); `data` = PatternsReport schema 1 |

## Pattern kinds (closed)

| Kind | Signal |
|------|--------|
| `inferred-layer` | Known layer dirs / path segments (`handlers`, `services`, …) |
| `naming-idiom` | File naming style + suffix idioms |
| `structural-idiom` | Entry files (`mod.rs`, `index.ts`, …) |
| `call-idiom` | AST HTTP methods / entity kinds (`--ast`) |
| `implicit-decision` | Workspace / stack layout decisions |

Pattern id format: `{kind}:{slug}` (e.g. `naming-idiom:snake-case`).

## Exit codes (004 map)

| Code | Kind | When |
|------|------|------|
| 0 | — | Success |
| 2 | Usage | invalid args / clap |
| 3 | NotFound | `--dir` missing / not a directory |
| 4 | InvalidInput | project root not found / bad `--modules` / path safety |
| 5 | Io | unexpected I/O |

## JSON schema version 1 (frozen)

CamelCase fields:

| Field | Type | Notes |
|-------|------|-------|
| `schemaVersion` | number | Always `1` |
| `mode` | string | `"check"` or `"write"` |
| `projectRoot` | string | Absolute display path |
| `patterns` | `DiscoveredPattern[]` | Sorted by `(kind, id)` |
| `cooccurrences` | `Cooccurrence[]` | Sorted by `(left, right)` |
| `written` | string[] | Relative POSIX paths; empty in check |
| `modulesScanned` | string[] | Module ids analyzed |
| `astEnabled` | bool | Whether `--ast` was set |
| `inject` | bool | Whether `--inject` was set |
| `graphIndexed` | bool | True only when existing graph store was updated |
| `warnings` | string[] | Soft failures (ast/graph) |

### DiscoveredPattern

| Field | Type | Notes |
|-------|------|-------|
| `id` | string | `{kind}:{slug}` |
| `kind` | string | One of the five closed kinds |
| `title` | string | Human explanation (redacted) |
| `frequency` | number | Occurrence count |
| `score` | number | Equals `frequency` (stable) |
| `evidence` | string[] | Origin paths; sorted unique |
| `modules` | string[] | Module ids where seen |

## Security

- Path jail via `ProjectRoot` / `SafeRelativePath` / `atomic_write`
- Evidence and titles passed through `redact`
- No shell concatenation
- Graph indexing is **soft**: missing/unmigrated store → warning, exit 0

## Compatibility vs TypeScript 3.18.1

| Diff | Class | Note |
|------|-------|------|
| No `--ai` on Rust CLI in 038 | B | Semantic enrichment via IDE skill `/dare-patterns` |
| Native AST (`dare-ast`) | B | Same opt-in `--ast`; engine from 035 |
| Soft graph Pattern nodes when store exists | B | Does not create graph DB on every run |
| PatternsReport schema 1 camelCase | A | Aligns with dna/discover reports |

## Acceptance smokes

- `dare patterns` writes `PATTERNS.md` + `patterns-facts.json`
- `dare patterns --check` leaves filesystem unchanged
- `dare patterns --help` lists `--check`, `--modules`, `--inject`, `--ast`
