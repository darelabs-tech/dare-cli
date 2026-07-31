# Tasks: Validate — validação do DAG (020)

> **Fonte:** `DARE/BLUEPRINT-020-validate.md`  
> **Design:** `DARE/DESIGN-020-validate.md`  
> **DAG:** `DARE/dare-dag-020.yaml`  
> **Specs:** `DARE/EXECUTION-020/`  
> **Progresso:** 7/7 (100%)  
> **Nota:** IDs `mp020-*`; **DONE** — crate `dare-dag` + `dare validate`; próximo 021

## Visão Geral

- Total de Tasks: 7
- Ranks paralelos: 5 (rank 0: 2 tasks)
- Tempo estimado: ~6–10 h

## Tabela de Status

| ID        | Título                                                    | Status     | Depends On                      | Complexity |
|-----------|-----------------------------------------------------------|------------|---------------------------------|------------|
| mp020-001 | Verificar docker-compose.ci.yml                           | ✅ DONE    | —                               | LOW        |
| mp020-002 | Scaffold crate dare-dag + ValidationReport schema 1       | ✅ DONE    | —                               | MED        |
| mp020-003 | Regras validate + ciclo + fixtures                        | ✅ DONE    | mp020-002                       | HIGH       |
| mp020-004 | CLI dare validate + smokes                                | ✅ DONE    | mp020-003                       | MED        |
| mp020-005 | Docs cli-validate.md + DEC-021                            | ✅ DONE    | mp020-002                       | LOW        |
| mp020-006 | Auditoria Ralph (test/clippy/audit/deny)                  | ✅ DONE    | mp020-001, mp020-004, mp020-005 | MED        |
| mp020-007 | Fechamento microplano 020                                 | ✅ DONE    | mp020-006                       | LOW        |

## Próximas Etapas

1. ~~Revisar e aprovar este TASKS + DAG~~
2. ~~Executar `/dare-dag-run-parallel`~~
3. **Próximo microplano:** `021-update-planejamento-e-manifest`
