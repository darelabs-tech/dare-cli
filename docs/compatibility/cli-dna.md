# CLI dna (`dare dna`)

> **DEC-039** · Microplano 037 · Source: `crates/dare-project/src/dna.rs` + `crates/dare-cli/src/commands/dna.rs`

## Purpose

Extract deterministic project convention facts (tooling, naming, architecture, tests, libraries, limited Git commits) and materialize:

- `DARE/PROJECT-DNA.md` — human markdown with evidence tables + `<!-- AGENT -->` sections
- `DARE/dna-facts.json` — machine-readable facts (schemaVersion 1)

## Command / flags

```bash
dare dna [-d|--dir <path>] [--check] [--ast]
dare dna --check --json [-d <path>]
```

| Flag | Effect |
|------|--------|
| `-d` / `--dir <path>` | Start directory (default: cwd); walk-up project root |
| `--check` | Collect only — **zero filesystem mutations** |
| `--ast` | Optional AST sample via `dare-ast` (caps: 32 files × 512 KiB) |
| `--json` | Envelope via output renderer (004); `data` = DnaReport schema 1 |

## Exit codes (004 map)

| Code | Kind | When |
|------|------|------|
| 0 | — | Success |
| 2 | Usage | invalid args / clap |
| 3 | NotFound | `--dir` missing / not a directory |
| 4 | InvalidInput | project root not found / path safety |
| 5 | Io | unexpected I/O |

## JSON schema version 1 (frozen)

CamelCase fields:

| Field | Type | Notes |
|-------|------|-------|
| `schemaVersion` | number | Always `1` |
| `mode` | string | `"check"` or `"write"` |
| `projectRoot` | string | Absolute display path |
| `gitRoot` | string\|null | Absolute; null when no Git |
| `facts` | `DnaFact[]` | Sorted by `(category, key)` |
| `written` | string[] | Relative POSIX paths written; empty in check |
| `astEnabled` | bool | Whether `--ast` was set |
| `graphIndexed` | bool | True only when existing graph store was updated |
| `warnings` | string[] | Soft failures (git/ast/graph) |

### DnaFact

| Field | Type | Notes |
|-------|------|-------|
| `category` | string | `tooling`\|`naming`\|`architecture`\|`tests`\|`libraries`\|`commits` |
| `key` | string | Stable identifier |
| `value` | string | Redacted display value |
| `evidence` | string[] | Origin paths / `git:<hash>`; sorted unique |

## Security

- Path jail via `ProjectRoot` / `SafeRelativePath` / `atomic_write`
- Git via `SafeCommand` argv only (no shell concatenation)
- Evidence and values passed through `redact`
- Graph indexing is **soft**: missing/unmigrated store → warning, exit 0

## Compatibility vs TypeScript 3.18.1

| Diff | Class | Note |
|------|-------|------|
| No `--ai` on Rust CLI in 037 | B | Semantic enrichment via IDE skill `/dare-dna` |
| Native AST (`dare-ast`) | B | Same opt-in `--ast`; engine from 035 |
| Soft graph index when store exists | B | Does not create graph DB on every run |
| DnaReport schema 1 camelCase | A | Aligns with discover-style reports |

## Acceptance smokes

- `dare dna` writes `PROJECT-DNA.md` + `dna-facts.json`
- `dare dna --check` leaves filesystem unchanged
- Project without `.git` succeeds with commit facts omitted
