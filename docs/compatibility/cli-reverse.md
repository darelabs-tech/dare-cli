# CLI reverse (`dare reverse`)

> **DEC-038** · Microplano 036 · Source: `crates/dare-project/src/reverse.rs` + `crates/dare-cli/src/commands/reverse.rs`

## Purpose

Brownfield **Fase 0** reverse engineering: inventário determinístico de módulos, `DARE/IDEIA.md`, specs em `DARE/REVERSE/`, facts JSON, AST opcional (`dare-ast`), Excalidraw opcional, report de confiança, enrichment soft-fail (`--ai`).

A skill `/dare-reverse` preenche marcadores AGENT; o CLI não inventa domínio.

## Commands

```bash
dare reverse [-d <path>]
dare reverse --check [-d <path>]
dare reverse --deep --ast --report
dare reverse --modules alpha,beta --no-excalidraw
dare reverse --ai [--provider mock|codex|…]
```

Global: `--json` / `--no-color`.

| Flag | Default | Effect |
|------|---------|--------|
| `-d` / `--dir` | cwd | Start path (walk-up to project root) |
| `--check` | off | Analyze only — **zero writes** |
| `--deep` | off | Fase-3 stubs (`erd.md`, `c4/…`, …) |
| `--modules` | all | Comma-separated module ids |
| `--ast` | off | Scan sources via `dare-ast` (caps) |
| `--no-excalidraw` | off | Skip `modules.excalidraw` (default writes it) |
| `--report` | off | Write `confidence-report.md` |
| `--ai` | off | Soft-fail enrichment of IDEIA sections |
| `--provider` | `codex` | Requires `--ai` |

## Artifacts

| Path | Notes |
|------|-------|
| `DARE/IDEIA.md` | Module map (🟢) + AGENT enrichable sections |
| `DARE/REVERSE/reverse-facts.json` | Schema 1 facts |
| `DARE/REVERSE/module-<id>.md` | Per-module skeleton |
| `DARE/REVERSE/modules.excalidraw` | Unless `--no-excalidraw` |
| `DARE/REVERSE/confidence-report.md` | With `--report` |
| Deep stubs | With `--deep` |

## Exit codes

| Code | Kind | When |
|------|------|------|
| 0 | — | Success (incl. check / AI soft-fail) |
| 2 | Usage | Clap / `--provider` without `--ai` |
| 3 | NotFound | `-d` missing directory |
| 4 | InvalidInput | No project root / bad `--modules` |
| 5 | Io | Filesystem errors |

## JSON report (schemaVersion 1)

Envelope ADR-002; `data` includes `mode` (`check`|`reverse`), `moduleCount`, `written[]`, `warnings[]`, `enriched`, flags.

`--check` human output ends with:

```text
mode: check (zero mutations)
```

## Compatibility vs TS 3.18.1

| Diff | Class | Note |
|------|-------|------|
| Module discovery heuristic (crates/src/top-level) | **B** | Simpler than full TS walker; documented |
| Dependency graph = Cargo path/workspace names | **B** | Full import graph deferred (037/038) |
| Enrichment soft-fail | **A** | Same pattern as blueprint |
| Canonical paths `DARE/*` | **A** | Skill parity |
| `--check` zero-write | **A** | Acceptance MUST |

## Security

- All writes via `ProjectRoot` + `atomic_write`
- Skip heavy dirs; caps on AST files/bytes
- No shell concatenation
- AI stderr redacted via `dare-ai`

## Ralph

```bash
cargo fmt --check
cargo clippy -p dare-project -p dare-cli --all-targets -- -D warnings
cargo test -p dare-project
cargo test -p dare-cli --test cli_smoke reverse
```
