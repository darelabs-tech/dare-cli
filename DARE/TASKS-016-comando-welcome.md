# Tasks: Comando welcome (016)

> **Fonte:** `DARE/BLUEPRINT-016-comando-welcome.md`  
> **Design:** `DARE/DESIGN-016-comando-welcome.md`  
> **DAG:** `DARE/dare-dag-016.yaml`  
> **Specs:** `DARE/EXECUTION-016/`  
> **Progresso:** 6/6 (100%)  
> **Nota:** DEC-017 / `cli-welcome.md` fechados; próximo: `017-comando-info`

## Visão Geral

- Total de Tasks: 6
- Status: **DONE** — microplano 016 fechado; próximo: `017-comando-info`

## Tabela de Status

| ID        | Título                                                      | Status  | Depends On                          | Complexity |
|-----------|-------------------------------------------------------------|---------|-------------------------------------|------------|
| mp016-001 | Verificar docker-compose.ci.yml                             | ✅ DONE | —                                   | LOW        |
| mp016-002 | Congelar render_welcome + política banner + CI-005          | ✅ DONE | —                                   | MED        |
| mp016-003 | CLI wiring + smoke welcome                                  | ✅ DONE | mp016-002                           | LOW        |
| mp016-004 | Docs cli-welcome.md + DEC-017                               | ✅ DONE | mp016-002                           | LOW        |
| mp016-005 | Auditoria Ralph (test/clippy/audit/deny)                    | ✅ DONE | mp016-001, mp016-003, mp016-004     | MED        |
| mp016-006 | Fechamento microplano 016                                   | ✅ DONE | mp016-005                           | LOW        |

## Próximas Etapas

1. Microplano **017** — comando info (`017-comando-info`)
