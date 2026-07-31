# Tasks: Scaffolding — contratos, stacks e artefatos AX (046)

> **Fonte:** `DARE/BLUEPRINT-046-scaffolding-contratos-stacks-e-artefatos-ax.md` (APPROVED)  
> **Design:** `DARE/DESIGN-046-scaffolding-contratos-stacks-e-artefatos-ax.md`  
> **DAG:** `DARE/dare-dag-046.yaml`  
> **Specs:** `DARE/EXECUTION-046/`  
> **DEC:** DEC-047  
> **Progresso:** 6/6 (100%)

## Visão Geral

- Total de Tasks: 6
- Ranks: 0 (001) → 1 (002 ∥ 003) → 2 (004) → 3 (005) → 4 (006)
- Tempo estimado: ~10–14 h
- Escopo: crate `dare-scaffold` + 11 stacks + 7 AX + plan/apply/rollback — **sem** CLI init/bootstrap

## Tabela de Status

| ID        | Título                                      | Status     | Depends On            | Complexity |
|-----------|---------------------------------------------|------------|-----------------------|------------|
| mp046-001 | crate + types + registry (11 stacks)        | ✅ DONE    | —                     | HIGH       |
| mp046-002 | templates assets/stacks + render            | ✅ DONE    | mp046-001             | HIGH       |
| mp046-003 | AX generators (7 artefatos)                 | ✅ DONE    | mp046-001             | MED        |
| mp046-004 | plan / apply / rollback                     | ✅ DONE    | mp046-002, mp046-003  | HIGH       |
| mp046-005 | validate + fixtures greenfield              | ✅ DONE    | mp046-004             | MED        |
| mp046-006 | Docs DEC-047 + matriz + Ralph close         | ✅ DONE    | mp046-005             | MED        |

## Progresso

```
████████████████████ 100%
```

## Tarefas por Rank

### Rank 0
- mp046-001 — crate + registry ✅

### Rank 1 (paralelo após 001)
- mp046-002 — templates + render ✅
- mp046-003 — AX generators ✅

### Rank 2
- mp046-004 — plan/apply/rollback ✅

### Rank 3
- mp046-005 — validate + fixtures ✅

### Rank 4
- mp046-006 — docs + Ralph ✅

## Caminho crítico

`001 → (002∥003) → 004 → 005 → 006` — **concluído**

## Próximas Etapas

1. Microplano **047** — `dare init` / `dare bootstrap` CLI
2. Revisão humana antes de merge na branch principal
