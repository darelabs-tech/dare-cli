# Tasks: DAG — parser, ranks e state store (026)

> **Fonte:** `DARE/BLUEPRINT-026-dag-parser-ranks-e-state-store.md`  
> **Design:** `DARE/DESIGN-026-dag-parser-ranks-e-state-store.md`  
> **DAG:** `DARE/dare-dag-026.yaml`  
> **Specs:** `DARE/EXECUTION-026/`  
> **Progresso:** 8/8 (100%)  
> **Nota:** IDs `mp026-*`; **DONE** — library-first em `dare-dag` (sem CLI execute/viz); próximo **027**

## Visão Geral

- Total de Tasks: 8
- Ranks paralelos: 5 (rank 0: 2 tasks — `mp026-001`, `mp026-002`)
- Tempo estimado: ~12–16 h

## Tabela de Status

| ID        | Título                                              | Status     | Depends On                         | Complexity |
|-----------|-----------------------------------------------------|------------|------------------------------------|------------|
| mp026-001 | Verificar docker-compose.ci.yml                     | ✅ DONE    | —                                  | LOW        |
| mp026-002 | graph ranks + find_cycle_path + fixtures            | ✅ DONE    | —                                  | HIGH       |
| mp026-003 | TaskStatus + cascading skip                         | ✅ DONE    | mp026-002                          | MED        |
| mp026-004 | State store ensure/transition + FileLock            | ✅ DONE    | mp026-003                          | HIGH       |
| mp026-005 | Canvas render/write + refresh                       | ✅ DONE    | mp026-004                          | MED        |
| mp026-006 | next_executable + ranks_validated + proptest     | ✅ DONE    | mp026-002, mp026-003               | MED        |
| mp026-007 | Auditoria Ralph (fmt/clippy/test/audit)             | ✅ DONE    | mp026-001, mp026-004, mp026-005, mp026-006 | MED |
| mp026-008 | Docs dag-runtime.md + DEC-027 + fechamento          | ✅ DONE    | mp026-007                          | LOW        |

## Tarefas por Fase

### Phase 1: Container
- mp026-001

### Phase 2: Graph
- mp026-002

### Phase 3: Skip / status
- mp026-003

### Phase 4–5: State + canvas
- mp026-004 → mp026-005

### Phase 6: Helpers + properties (paralelo a state após 003)
- mp026-006

### Phase 7–8: Audit + docs
- mp026-007 → mp026-008

## Progresso

```
████████████████████ 100%
```

## Próximas Etapas

1. ~~Revisar e aprovar este TASKS + DAG + specs~~
2. ~~Executar DAG `mp026-*`~~
3. **Próximo microplano:** `027-dag-visualizacao`
