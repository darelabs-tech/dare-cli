# Tasks: Adapter Antigravity (014)

> **Fonte:** `DARE/BLUEPRINT-014-adapter-antigravity.md`  
> **Design:** `DARE/DESIGN-014-adapter-antigravity.md`  
> **DAG:** `DARE/dare-dag-014.yaml`  
> **Specs:** `DARE/EXECUTION-014/`  
> **Progresso:** 7/7 (100%)  
> **Nota:** DEC-015 congelado; SoT 49; share Codex; próximo: `015-pipeline-de-release-nativo-alpha`

## Visão Geral

- Total de Tasks: 7
- Status: **DONE** — microplano 014 fechado; próximo: `015-pipeline-de-release-nativo-alpha`

## Tabela de Status

| ID        | Título                                                      | Status  | Depends On           | Complexity |
|-----------|-------------------------------------------------------------|---------|----------------------|------------|
| mp014-001 | Verificar docker-compose.ci.yml                             | ✅ DONE | —                    | LOW        |
| mp014-002 | Congelar detect + rules + workflows + preserve              | ✅ DONE | —                    | MED        |
| mp014-003 | install + validate + frontmatter + coexistência Codex       | ✅ DONE | mp014-002            | MED        |
| mp014-004 | CLI help `--force` + install pipeline                       | ✅ DONE | mp014-002            | LOW        |
| mp014-005 | CLI smoke + docs DEC-015 + exception                        | ✅ DONE | mp014-003, mp014-004 | MED        |
| mp014-006 | Auditoria Ralph (test/clippy/audit/deny)                    | ✅ DONE | mp014-001, mp014-005 | MED        |
| mp014-007 | Fechamento microplano 014                                   | ✅ DONE | mp014-006            | LOW        |

## Próximas Etapas

1. Microplano **015** — pipeline de release nativo alpha (`015-pipeline-de-release-nativo-alpha`)
