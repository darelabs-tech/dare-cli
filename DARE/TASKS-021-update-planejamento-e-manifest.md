# Tasks: Update — planejamento e manifest (021)

> **Fonte:** `DARE/BLUEPRINT-021-update-planejamento-e-manifest.md`  
> **Design:** `DARE/DESIGN-021-update-planejamento-e-manifest.md`  
> **DAG:** `DARE/dare-dag-021.yaml`  
> **Specs:** `DARE/EXECUTION-021/`  
> **Progresso:** 8/8 (100%)  
> **Nota:** IDs `mp021-*`; **DONE** — crate `dare-update` + `dare update --dry-run`; apply → 022; docs **DEC-022**

## Visão Geral

- Total de Tasks: 8
- Ranks paralelos: 5 (rank 0: 2 tasks)
- Tempo estimado: ~8–12 h

## Tabela de Status

| ID        | Título                                                         | Status     | Depends On                      | Complexity |
|-----------|----------------------------------------------------------------|------------|---------------------------------|------------|
| mp021-001 | Verificar docker-compose.ci.yml                                | ✅ DONE    | —                               | LOW        |
| mp021-002 | Scaffold dare-update + content_is_managed                      | ✅ DONE    | —                               | MED        |
| mp021-003 | Manifest V2 embed + load/validate                              | ✅ DONE    | mp021-002                       | MED        |
| mp021-004 | Classify + plan_update + fixtures                              | ✅ DONE    | mp021-003                       | HIGH       |
| mp021-005 | CLI dare update --dry-run + smokes                             | ✅ DONE    | mp021-004                       | MED        |
| mp021-006 | Docs cli-update-plan.md + DEC-022 + skill                      | ✅ DONE    | mp021-002                       | LOW        |
| mp021-007 | Auditoria Ralph (test/clippy/audit/deny)                       | ✅ DONE    | mp021-001, mp021-005, mp021-006 | MED        |
| mp021-008 | Fechamento microplano 021                                      | ✅ DONE    | mp021-007                       | LOW        |

## Próximas Etapas

1. ~~Revisar e aprovar este TASKS + DAG~~
2. ~~Executar `/dare-dag-run-parallel`~~
3. **Próximo microplano:** `022-update-aplicacao-backup-e-migrations`
