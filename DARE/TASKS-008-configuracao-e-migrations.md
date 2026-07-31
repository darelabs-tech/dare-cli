# Tasks: Configuração e migrations (008)

> **Fonte:** `DARE/BLUEPRINT-008-configuracao-e-migrations.md`  
> **DAG:** `DARE/dare-dag-008.yaml`  
> **Specs:** `DARE/EXECUTION-008/`  
> **Progresso:** 7/7 (100%)  
> **Nota:** crate `dare-config` — harden + gaps (re-export strict, fixtures tests, docs) + closeout

## Visão Geral

- Total de Tasks: 7
- Ranks paralelos: 5 (rank 0: 2; rank 1: 2)

## Tabela de Status

| ID        | Título                                           | Status   | Depends On           | Complexity |
|-----------|--------------------------------------------------|----------|----------------------|------------|
| mp008-001 | Verificar docker-compose.ci.yml                  | ✅ DONE  | —                    | LOW        |
| mp008-002 | Re-export `env_overrides_from_vars_strict`       | ✅ DONE  | —                    | LOW        |
| mp008-003 | Fixtures golden round-trip                       | ✅ DONE  | mp008-002            | MED        |
| mp008-004 | Matriz P/B + gates migration (schema/backup)     | ✅ DONE  | mp008-002            | MED        |
| mp008-005 | Docs config-and-migrations + DEC-009             | ✅ DONE  | mp008-003, mp008-004 | LOW        |
| mp008-006 | Auditoria Ralph (test/clippy/audit/deny)         | ✅ DONE  | mp008-001, mp008-005 | MED        |
| mp008-007 | Fechamento microplano 008                        | ✅ DONE  | mp008-006            | LOW        |

## Próximo

Microplano **009** — inventário e empacotamento de assets.
