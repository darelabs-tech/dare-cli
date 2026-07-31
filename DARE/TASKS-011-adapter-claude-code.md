# Tasks: Adapter Claude Code (011)

> **Fonte:** `DARE/BLUEPRINT-011-adapter-claude-code.md`  
> **Design:** `DARE/DESIGN-011-adapter-claude-code.md`  
> **DAG:** `DARE/dare-dag-011.yaml`  
> **Specs:** `DARE/EXECUTION-011/`  
> **Progresso:** 7/7 (100%)  
> **Nota:** contratos DEC-012 congelados; smoke CLI; docs; Ralph OK — próximo: `012-adapter-cursor`

## Visão Geral

- Total de Tasks: 7
- Ranks paralelos: 5 (rank 0: 2; rank 1: 2)
- Status: **DONE** — microplano 011 fechado; próximo: `012-adapter-cursor`

## Tabela de Status

| ID        | Título                                              | Status  | Depends On           | Complexity |
|-----------|-----------------------------------------------------|---------|----------------------|------------|
| mp011-001 | Verificar docker-compose.ci.yml                     | ✅ DONE | —                    | LOW        |
| mp011-002 | Congelar detect + generate_claude_md + preserve     | ✅ DONE | —                    | MED        |
| mp011-003 | install_commands + validate_install (49 + preserve) | ✅ DONE | mp011-002            | MED        |
| mp011-004 | settings.json + PostToolUse + help `--force`        | ✅ DONE | mp011-002            | MED        |
| mp011-005 | CLI smoke + docs DEC-012 + golden SHOULD            | ✅ DONE | mp011-003, mp011-004 | MED        |
| mp011-006 | Auditoria Ralph (test/clippy/audit/deny)            | ✅ DONE | mp011-001, mp011-005 | MED        |
| mp011-007 | Fechamento microplano 011                           | ✅ DONE | mp011-006            | LOW        |

## Próximas Etapas

1. Microplano **012** — adapter Cursor (`012-adapter-cursor`)
