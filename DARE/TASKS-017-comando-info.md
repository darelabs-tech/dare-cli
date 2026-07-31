# Tasks: Comando info (017)

> **Fonte:** `DARE/BLUEPRINT-017-comando-info.md`  
> **Design:** `DARE/DESIGN-017-comando-info.md`  
> **DAG:** `DARE/dare-dag-017.yaml`  
> **Specs:** `DARE/EXECUTION-017/`  
> **Progresso:** 6/6 (100%)  
> **Nota:** DEC-018 / `cli-info.md` fechados; próximo: `018-discover-deteccao-brownfield`

## Visão Geral

- Total de Tasks: 6
- Status: **DONE** — microplano 017 fechado; próximo: `018-discover-deteccao-brownfield`

## Tabela de Status

| ID        | Título                                                      | Status  | Depends On                      | Complexity |
|-----------|-------------------------------------------------------------|---------|---------------------------------|------------|
| mp017-001 | Verificar docker-compose.ci.yml                             | ✅ DONE | —                               | LOW        |
| mp017-002 | Congelar collect_info + schema 1 + TASKS sort + read-only   | ✅ DONE | —                               | MED        |
| mp017-003 | CLI wiring + smoke info / --json                            | ✅ DONE | mp017-002                       | LOW        |
| mp017-004 | Docs cli-info.md + DEC-018                                  | ✅ DONE | mp017-002                       | LOW        |
| mp017-005 | Auditoria Ralph (test/clippy/audit/deny)                    | ✅ DONE | mp017-001, mp017-003, mp017-004 | MED        |
| mp017-006 | Fechamento microplano 017                                   | ✅ DONE | mp017-005                       | LOW        |

## Próximas Etapas

1. Microplano **018** — discover detecção brownfield (`018-discover-deteccao-brownfield`)
