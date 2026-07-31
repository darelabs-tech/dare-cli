# Tasks: Init e bootstrap greenfield (047)

> **Fonte:** `DARE/BLUEPRINT-047-init-e-bootstrap.md` (APPROVED)  
> **Design:** `DARE/DESIGN-047-init-e-bootstrap.md`  
> **DAG:** `DARE/dare-dag-047.yaml`  
> **Specs:** `DARE/EXECUTION-047/`  
> **DEC:** DEC-048  
> **Progresso:** 7/7 (100%)

## Visão Geral

- Total de Tasks: 7
- Ranks: 0 (001 ∥ 002) → 1 (003 ∥ 005) → 2 (004) → 3 (006) → 4 (007)
- Tempo estimado: ~12–16 h
- Escopo: CLI `dare init` / `dare bootstrap` + `ConflictPolicy` + frontend unlock + golden + DEC-048

## Tabela de Status

| ID        | Título                                      | Status     | Depends On            | Complexity |
|-----------|---------------------------------------------|------------|-----------------------|------------|
| mp047-001 | clap + resolve flags (init/bootstrap)       | ✅ DONE    | —                     | HIGH       |
| mp047-002 | ConflictPolicy + frontend unlock            | ✅ DONE    | —                     | HIGH       |
| mp047-003 | run_init pipeline + rollback                | ✅ DONE    | mp047-001, mp047-002  | HIGH       |
| mp047-004 | interactive dialoguer                       | ✅ DONE    | mp047-003             | MED        |
| mp047-005 | run_bootstrap + idempotência                | ✅ DONE    | mp047-001, mp047-002  | HIGH       |
| mp047-006 | golden trees + CLI integration              | ✅ DONE    | mp047-003, mp047-004, mp047-005 | HIGH |
| mp047-007 | Docs DEC-048 + capabilities + Ralph         | ✅ DONE    | mp047-006             | MED        |

## Progresso

```
████████████████████ 100%
```

## Tarefas por Rank

### Rank 0 (paralelo)
- mp047-001 — clap + resolve flags ✅
- mp047-002 — ConflictPolicy + frontend unlock ✅

### Rank 1 (paralelo após 001+002)
- mp047-003 — run_init + rollback ✅
- mp047-005 — run_bootstrap + idempotência ✅

### Rank 2
- mp047-004 — interactive dialoguer ✅

### Rank 3
- mp047-006 — golden + CLI tests ✅

### Rank 4
- mp047-007 — docs + DEC-048 + Ralph ✅

## Caminho crítico

`001∥002 → 003 → 004 → 006 → 007` (005 em paralelo com 003)

## Próximas Etapas

1. Microplano **048** (hooks e steering)
