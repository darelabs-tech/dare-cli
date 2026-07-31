# Tasks: Modelo canónico de capabilities (010)

> **Fonte:** `DARE/BLUEPRINT-010-modelo-canonico-de-capabilities.md`  
> **Design:** `DARE/DESIGN-010-modelo-canonico-de-capabilities.md`  
> **DAG:** `DARE/dare-dag-010.yaml`  
> **Specs:** `DARE/EXECUTION-010/`  
> **Progresso:** 7/7 (100%)  
> **Nota:** tipos + matrix 49 já existem — escopo = harden validate, exceptions Classe C, smoke, docs, closeout

## Visão Geral

- Total de Tasks: 7
- Ranks paralelos: 5 (rank 0: 2; rank 1: 2)
- Status: **DONE** — microplano 010 fechado; próximo: `011-adapter-claude-code`

## Tabela de Status

| ID        | Título                                              | Status  | Depends On           | Complexity |
|-----------|-----------------------------------------------------|---------|----------------------|------------|
| mp010-001 | Verificar docker-compose.ci.yml                     | ✅ DONE | —                    | LOW        |
| mp010-002 | Harden validate (regex/paths) + exceptions YAML     | ✅ DONE | —                    | MED        |
| mp010-003 | Contagem 49 + matrix_loads_and_validates            | ✅ DONE | mp010-002            | LOW        |
| mp010-004 | Render snapshots (claude + skill frontmatter)       | ✅ DONE | mp010-002            | LOW        |
| mp010-005 | CLI smoke capabilities validate + docs DEC-011      | ✅ DONE | mp010-003, mp010-004 | MED        |
| mp010-006 | Auditoria Ralph (test/clippy/audit/deny)            | ✅ DONE | mp010-001, mp010-005 | MED        |
| mp010-007 | Fechamento microplano 010                           | ✅ DONE | mp010-006            | LOW        |

## Próximas Etapas

1. Microplano **011** — adapter Claude Code (`011-adapter-claude-code`)
