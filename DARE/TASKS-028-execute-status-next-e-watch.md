# Tasks: Execute — status, next e watch (028)

> **Fonte:** `DARE/BLUEPRINT-028-execute-status-next-e-watch.md`  
> **Design:** `DARE/DESIGN-028-execute-status-next-e-watch.md`  
> **DAG:** `DARE/dare-dag-028.yaml`  
> **Specs:** `DARE/EXECUTION-028/`  
> **Progresso:** 7/7 (100%)  
> **Nota:** IDs `mp028-*`; **DONE** — `dare_dag::execution` + CLI `dare execute --status|--next|--watch`; próximo **029**

## Visão Geral

- Total de Tasks: 7
- Ranks paralelos: 6 (rank 0: 2 tasks — `mp028-001`, `mp028-002`)
- Tempo estimado: ~10–14 h

## Tabela de Status

| ID        | Título                                              | Status     | Depends On                    | Complexity |
|-----------|-----------------------------------------------------|------------|-------------------------------|------------|
| mp028-001 | Verificar docker-compose.ci.yml                     | ✅ DONE    | —                             | LOW        |
| mp028-002 | execution core (ready/compose/snapshot)             | ✅ DONE    | —                             | HIGH       |
| mp028-003 | CLI `--status` + `--next` + smokes                  | ✅ DONE    | mp028-002                     | HIGH       |
| mp028-004 | CLI `--watch` + read-only guarantee                 | ✅ DONE    | mp028-003                     | MED        |
| mp028-005 | Capability + cli-execute-status.md + DEC-029        | ✅ DONE    | mp028-004                     | MED        |
| mp028-006 | Auditoria Ralph (fmt/clippy/test/audit)             | ✅ DONE    | mp028-001, mp028-004, mp028-005 | MED      |
| mp028-007 | Fechamento TASKS/matriz/Blueprint                   | ✅ DONE    | mp028-006                     | LOW        |

## Tarefas por Fase

### Phase 1: Container
- mp028-001

### Phase 2: Domain execution
- mp028-002

### Phase 3: CLI status/next
- mp028-003 (deps: 002)

### Phase 4: CLI watch
- mp028-004 (deps: 003)

### Phase 5: Docs + capability
- mp028-005 (deps: 004)

### Phase 6–7: Audit + closeout
- mp028-006 → mp028-007

## Progresso

```
████████████████████ 100%
```

## Próximas Etapas

1. Microplano **029** — `dare execute --complete|--fail|--reset` + Ralph inicial
2. Comando: ver `DARE-RUST-MICRO-PLANOS/.../029-execute-complete-fail-reset-e-ralph-inicial.md`
