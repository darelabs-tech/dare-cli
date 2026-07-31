# Tasks: DAG — visualização `dare dag viz` (027)

> **Fonte:** `DARE/BLUEPRINT-027-dag-visualizacao.md`  
> **Design:** `DARE/DESIGN-027-dag-visualizacao.md`  
> **DAG:** `DARE/dare-dag-027.yaml`  
> **Specs:** `DARE/EXECUTION-027/`  
> **Progresso:** 7/7 (100%)  
> **Nota:** IDs `mp027-*`; **DONE** — `dare_dag::viz` + CLI `dare dag viz`; próximo **028**

## Visão Geral

- Total de Tasks: 7
- Ranks paralelos: 6 (rank 0: 2 tasks — `mp027-001`, `mp027-002`)
- Tempo estimado: ~10–14 h

## Tabela de Status

| ID        | Título                                              | Status     | Depends On              | Complexity |
|-----------|-----------------------------------------------------|------------|-------------------------|------------|
| mp027-001 | Verificar docker-compose.ci.yml                     | ✅ DONE    | —                       | LOW        |
| mp027-002 | viz Mermaid core + goldens                          | ✅ DONE    | —                       | HIGH       |
| mp027-003 | viz DOT + Excalidraw + cores + OUTPUT_CAP           | ✅ DONE    | mp027-002               | HIGH       |
| mp027-004 | CLI `dare dag viz` + smokes                         | ✅ DONE    | mp027-003               | HIGH       |
| mp027-005 | Capability matrix + cli-dag-viz.md + DEC-028        | ✅ DONE    | mp027-004               | MED        |
| mp027-006 | Auditoria Ralph (fmt/clippy/test/audit)             | ✅ DONE    | mp027-001, mp027-004, mp027-005 | MED |
| mp027-007 | Fechamento TASKS/matriz/Blueprint                   | ✅ DONE    | mp027-006               | LOW        |

## Tarefas por Fase

### Phase 1: Container
- mp027-001

### Phase 2: Viz Mermaid
- mp027-002

### Phase 3: DOT + Excalidraw
- mp027-003 (deps: 002)

### Phase 4: CLI
- mp027-004 (deps: 003)

### Phase 5: Docs + capability
- mp027-005 (deps: 004)

### Phase 6–7: Audit + closeout
- mp027-006 → mp027-007

## Progresso

```
████████████████████ 100%
```

## Próximas Etapas

1. ~~Revisar e aprovar este TASKS + DAG + specs~~
2. ~~Executar DAG `mp027-*`~~
3. **Próximo microplano:** `028-execute-status-next-e-watch`
