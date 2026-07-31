# Tasks: Update — aplicação, backup e migrations (022)

> **Fonte:** `DARE/BLUEPRINT-022-update-aplicacao-backup-e-migrations.md`  
> **Design:** `DARE/DESIGN-022-update-aplicacao-backup-e-migrations.md`  
> **DAG:** `DARE/dare-dag-022.yaml`  
> **Specs:** `DARE/EXECUTION-022/`  
> **Progresso:** 8/8 (100%)  
> **Pré-requisito:** microplano **021** (`UpdatePlan`, dry-run)  
> **Nota:** IDs `mp022-*`; **DONE** — apply + backup + migrate + `-y`/`--force`; docs **DEC-023**; próximo → 023

## Visão Geral

- Total de Tasks: 8
- Ranks paralelos: 6 (rank 0: 2 tasks)
- Tempo estimado: ~10–14 h

## Tabela de Status

| ID        | Título                                                         | Status     | Depends On                      | Complexity |
|-----------|----------------------------------------------------------------|------------|---------------------------------|------------|
| mp022-001 | Verificar docker-compose.ci.yml                                | ✅ DONE    | —                               | LOW        |
| mp022-002 | Policy + session backup + journal/rollback                     | ✅ DONE    | —                               | MED        |
| mp022-003 | apply_update assets + UpdateApplyReport                        | ✅ DONE    | mp022-002                       | HIGH       |
| mp022-004 | Migrations config + failpoint rollback                         | ✅ DONE    | mp022-003                       | HIGH       |
| mp022-005 | CLI -y/--force + smokes apply                                  | ✅ DONE    | mp022-004                       | MED        |
| mp022-006 | Docs cli-update-apply.md + DEC-023                             | ✅ DONE    | mp022-002                       | LOW        |
| mp022-007 | Auditoria Ralph (test/clippy/audit/deny)                       | ✅ DONE    | mp022-001, mp022-005, mp022-006 | MED        |
| mp022-008 | Fechamento microplano 022                                      | ✅ DONE    | mp022-007                       | LOW        |

## Próximas Etapas

1. ~~Revisar e aprovar este TASKS + DAG~~
2. ~~Executar `/dare-dag-run-parallel`~~
3. **Próximo microplano:** `023-design-deterministico`
