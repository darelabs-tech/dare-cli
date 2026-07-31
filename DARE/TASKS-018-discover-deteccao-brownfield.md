# Tasks: Discover — detecção brownfield (018)

> **Fonte:** `DARE/BLUEPRINT-018-discover-deteccao-brownfield.md`  
> **Design:** `DARE/DESIGN-018-discover-deteccao-brownfield.md`  
> **DAG:** `DARE/dare-dag-018.yaml`  
> **Specs:** `DARE/EXECUTION-018/`  
> **Progresso:** 7/7 (100%)  
> **Nota:** IDs `mp018-*`; **DONE** — crate `dare-project` + `dare discover --check`; install = 019

## Visão Geral

- Total de Tasks: 7
- Ranks paralelos: 5 (rank 0: 2 tasks)
- Tempo estimado: ~6–10 h

## Tabela de Status

| ID        | Título                                                              | Status     | Depends On                      | Complexity |
|-----------|---------------------------------------------------------------------|------------|---------------------------------|------------|
| mp018-001 | Verificar docker-compose.ci.yml                                     | ✅ DONE    | —                               | LOW        |
| mp018-002 | Crate dare-project + report + root + stacks + conflicts             | ✅ DONE    | —                               | HIGH       |
| mp018-003 | Git + monorepo + harnesses + detect() + read-only                   | ✅ DONE    | mp018-002                       | HIGH       |
| mp018-004 | CLI discover --check + fixtures + smoke                             | ✅ DONE    | mp018-003                       | MED        |
| mp018-005 | Docs cli-discover-check.md + DEC-019                                | ✅ DONE    | mp018-002                       | LOW        |
| mp018-006 | Auditoria Ralph (test/clippy/audit/deny)                            | ✅ DONE    | mp018-001, mp018-004, mp018-005 | MED        |
| mp018-007 | Fechamento microplano 018                                           | ✅ DONE    | mp018-006                       | LOW        |

## Tarefas por Fase

### Phase 1: Containerização
- mp018-001

### Phase 2: Crate + stacks
- mp018-002

### Phase 3: detect orquestrado
- mp018-003 (deps: 002)

### Phase 4: CLI + fixtures + smoke
- mp018-004 (deps: 003)

### Phase 5: Docs
- mp018-005 (deps: 002) — paralelo com 003

### Phase 6: Auditoria
- mp018-006 (deps: 001, 004, 005)

### Phase 7: Fechamento
- mp018-007 (deps: 006)

## Próximas Etapas

1. ~~Revisar e aprovar este TASKS + DAG~~
2. ~~Executar `/dare-dag-run-parallel`~~
3. **Próximo microplano:** `019-discover-instalacao-do-dare`
