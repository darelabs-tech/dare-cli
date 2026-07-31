# Tasks: Adapter Cursor (012)

> **Fonte:** `DARE/BLUEPRINT-012-adapter-cursor.md`  
> **Design:** `DARE/DESIGN-012-adapter-cursor.md`  
> **DAG:** `DARE/dare-dag-012.yaml`  
> **Specs:** `DARE/EXECUTION-012/`  
> **Progresso:** 7/7 (100%)  
> **Nota:** DEC-013 congelado; SoT 49 commands; rules `.mdc` deferred (Class C) — próximo: `013-adapter-codex`

## Visão Geral

- Total de Tasks: 7
- Status: **DONE** — microplano 012 fechado; próximo: `013-adapter-codex`

## Tabela de Status

| ID        | Título                                              | Status  | Depends On           | Complexity |
|-----------|-----------------------------------------------------|---------|----------------------|------------|
| mp012-001 | Verificar docker-compose.ci.yml                     | ✅ DONE | —                    | LOW        |
| mp012-002 | Congelar detect + generate_cursorrules + preserve   | ✅ DONE | —                    | MED        |
| mp012-003 | install_cursor_commands + validate (49 + preserve)  | ✅ DONE | mp012-002            | MED        |
| mp012-004 | CLI help `--force` + install pipeline               | ✅ DONE | mp012-002            | LOW        |
| mp012-005 | CLI smoke + docs DEC-013 + exceptions               | ✅ DONE | mp012-003, mp012-004 | MED        |
| mp012-006 | Auditoria Ralph (test/clippy/audit/deny)            | ✅ DONE | mp012-001, mp012-005 | MED        |
| mp012-007 | Fechamento microplano 012                           | ✅ DONE | mp012-006            | LOW        |

## Próximas Etapas

1. Microplano **013** — adapter Codex (`013-adapter-codex`)
