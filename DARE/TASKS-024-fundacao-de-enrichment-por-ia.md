# Tasks: Fundação de enrichment por IA (024)

> **Fonte:** `DARE/BLUEPRINT-024-fundacao-de-enrichment-por-ia.md`  
> **Design:** `DARE/DESIGN-024-fundacao-de-enrichment-por-ia.md`  
> **DAG:** `DARE/dare-dag-024.yaml`  
> **Specs:** `DARE/EXECUTION-024/`  
> **Progresso:** 8/8 (100%)  
> **Nota:** IDs `mp024-*`; **DONE** — `dare-ai` + `dare design --ai`; próximo **025**

## Visão Geral

- Total de Tasks: 8
- Ranks paralelos: 6 (rank 0: 2 tasks; rank 1: 2 tasks)
- Tempo estimado: ~10–14 h

## Tabela de Status

| ID        | Título                                                    | Status     | Depends On                      | Complexity |
|-----------|-----------------------------------------------------------|------------|---------------------------------|------------|
| mp024-001 | Verificar docker-compose.ci.yml                           | ✅ DONE    | —                               | LOW        |
| mp024-002 | Scaffold dare-ai + schema + inject                        | ✅ DONE    | —                               | HIGH       |
| mp024-003 | MockProvider + resolve_provider                           | ✅ DONE    | mp024-002                       | MED        |
| mp024-004 | CodexCliProvider + overrides + timeout/stdin              | ✅ DONE    | mp024-003                       | HIGH       |
| mp024-005 | CLI --ai/--provider + DesignReport v2 + smokes            | ✅ DONE    | mp024-004                       | MED        |
| mp024-006 | Docs cli-design AI + DEC-025                              | ✅ DONE    | mp024-002                       | LOW        |
| mp024-007 | Auditoria Ralph (test/clippy/audit/deny)                  | ✅ DONE    | mp024-001, mp024-005, mp024-006 | MED        |
| mp024-008 | Fechamento microplano 024                                 | ✅ DONE    | mp024-007                       | LOW        |

## Próximas Etapas

1. ~~Revisar e aprovar este TASKS + DAG~~
2. ~~Executar DAG `mp024-*`~~
3. **Próximo microplano:** `025-blueprint`
