# Compatibility: AST Engine (`dare-ast`)

> **DEC-032** · Microplano 035 · Source: `crates/dare-ast`  
> Library-only — no `dare-cli` / `main.rs` changes in this cycle.

## Purpose

Native tree-sitter extraction of HTTP endpoints and type-like entities, with transparent regex fallback and deterministic merge (AST preferred over regex).

Consumers: future `dare reverse` / `dna` / `patterns` (036–038). GraphRAG does **not** use this crate for code-index (regex there remains separate).

## Public API

```rust
use dare_ast::{analyze_source, detect_language, grammar_available, DataModel, Language};

let model: DataModel = analyze_source("src/routes.ts", source)?;
// model.endpoints / model.entities / model.warnings
```

| Type | Notes |
|------|-------|
| `Language` | typescript, tsx, javascript, python, php, go, ruby, rust |
| `HttpEndpoint` | `method` (uppercase), `path`, `line` (1-based), `source` Ast\|Regex |
| `Entity` | `name`, `kind` (class\|struct\|interface\|model\|enum), `line`, `source` |
| `MAX_SOURCE_BYTES` | 2 MiB |

## Feature flags

Default = all languages. Disable with `--no-default-features` (regex-only). Per-lang: `lang-typescript`, `lang-tsx`, `lang-javascript`, `lang-python`, `lang-php`, `lang-go`, `lang-ruby`, `lang-rust`.

## Determinism

- Endpoint dedupe key: `METHOD\0path` (prefer Ast)
- Entity dedupe key: `kind\0name` (prefer Ast)
- Sort: endpoints by `(method, path, line)`; entities by `(kind, name, line)`

## Compatibility vs TypeScript 3.18.1

| Diff | Class | Note |
|------|-------|------|
| Native tree-sitter vs web-tree-sitter WASM | **B** | Intentional; eliminates WASM load |
| Heuristic extractors ≠ full TS parity | **B** | Corpus MUST; refine in reverse cycle |
| No CLI `dare ast` | **C** | Out of scope; library crate only |
| Cap 2 MiB / NUL reject | **B** | Explicit safety (005 alignment) |

## Security

- No shell / process spawn
- Reject NUL and oversize sources
- Errors in English (`CoreError::invalid_input`)

## Verification

```bash
cargo test -p dare-ast --all-features
cargo test -p dare-ast --no-default-features
cargo fmt --check && cargo clippy --workspace --all-features -- -D warnings && cargo test --workspace
```
