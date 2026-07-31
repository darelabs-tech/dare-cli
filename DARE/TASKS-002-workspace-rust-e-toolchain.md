# Tasks: Workspace Rust e toolchain (Microplano 002)

> **Fonte:** `DARE/BLUEPRINT-002-workspace-rust-e-toolchain.md`  
> **DAG:** `DARE/dare-dag-002.yaml`  
> **Specs:** `DARE/EXECUTION-002/`  
> **IDs:** `mp002-*`  
> **Não substitui:** `DARE/TASKS.md` / `dare-dag.yaml` (001 — 13/13 DONE)

## Visão Geral

- **Total de Tasks:** 11
- **Progresso:** 11/11 (100%)

```
████████████████████ 100%
```

## Tabela de Status

| ID        | Título                                           | Status   | Depends On                     | Complexity |
|-----------|--------------------------------------------------|----------|--------------------------------|------------|
| mp002-001 | Docker Rust smoke (Dockerfile + Compose)         | ✅ DONE  | —                              | LOW        |
| mp002-002 | Toolchain, workspace root, LICENSE, docs MSRV    | ✅ DONE  | —                              | MED        |
| mp002-003 | Crate dare-core (APIs mínimas + testes)          | ✅ DONE  | mp002-002                      | MED        |
| mp002-004 | Crate dare-contracts                             | ✅ DONE  | mp002-003                      | LOW        |
| mp002-005 | Crate dare-assets                                | ✅ DONE  | mp002-003                      | LOW        |
| mp002-006 | Crate dare-config                                | ✅ DONE  | mp002-003, mp002-004           | LOW        |
| mp002-007 | Crate dare-cli --help/--version + smoke tests    | ✅ DONE  | mp002-003,004,005,006          | HIGH       |
| mp002-008 | rustfmt + clippy -D warnings (workspace)         | ✅ DONE  | mp002-007                      | MED        |
| mp002-009 | dare.config.json → rust-axum (Ralph Cargo)       | ✅ DONE  | mp002-007                      | LOW        |
| mp002-010 | cargo audit + checklist RS-*                     | ✅ DONE  | mp002-008, mp002-009           | HIGH       |
| mp002-011 | CI rust-workspace-002 + fechamento microplano    | ✅ DONE  | mp002-001, mp002-010           | MED        |

## Próximo

Microplano **003** — CI cross-platform e qualidade.
