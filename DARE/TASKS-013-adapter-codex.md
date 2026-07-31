# Tasks: Adapter Codex (013)

> **Fonte:** `DARE/BLUEPRINT-013-adapter-codex.md`  
> **Design:** `DARE/DESIGN-013-adapter-codex.md`  
> **DAG:** `DARE/dare-dag-013.yaml`  
> **Specs:** `DARE/EXECUTION-013/`  
> **Progresso:** 7/7 (100%)  
> **Nota:** DEC-014 congelado; SoT 49; share `.agents`; próximo: `014-adapter-antigravity`

## Visão Geral

- Total de Tasks: 7
- Status: **DONE** — microplano 013 fechado; próximo: `014-adapter-antigravity`

## Tabela de Status

| ID        | Título                                                    | Status  | Depends On           | Complexity |
|-----------|-----------------------------------------------------------|---------|----------------------|------------|
| mp013-001 | Verificar docker-compose.ci.yml                           | ✅ DONE | —                    | LOW        |
| mp013-002 | Congelar detect + AGENTS.md + policies                    | ✅ DONE | —                    | MED        |
| mp013-003 | install_codex_skills + validate + coexistência            | ✅ DONE | mp013-002            | MED        |
| mp013-004 | CLI help `--force` + install pipeline                     | ✅ DONE | mp013-002            | LOW        |
| mp013-005 | CLI smoke + docs DEC-014 + exception                      | ✅ DONE | mp013-003, mp013-004 | MED        |
| mp013-006 | Auditoria Ralph (test/clippy/audit/deny)                  | ✅ DONE | mp013-001, mp013-005 | MED        |
| mp013-007 | Fechamento microplano 013                                 | ✅ DONE | mp013-006            | LOW        |

## Próximas Etapas

1. Microplano **014** — adapter Antigravity (`014-adapter-antigravity`)
