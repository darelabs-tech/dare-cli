# Tasks: Design determinístico — `dare design` (023)

> **Fonte:** `DARE/BLUEPRINT-023-design-deterministico.md`  
> **Design:** `DARE/DESIGN-023-design-deterministico.md`  
> **DAG:** `DARE/dare-dag-023.yaml`  
> **Specs:** `DARE/EXECUTION-023/`  
> **Progresso:** 8/8 (100%)  
> **Nota:** IDs `mp023-*`; **DONE** — `dare design` determinístico; próximo **024**

## Visão Geral

- Total de Tasks: 8
- Ranks paralelos: 5 (rank 0: 3 tasks)
- Tempo estimado: ~8–12 h

## Tabela de Status

| ID        | Título                                                    | Status     | Depends On                      | Complexity |
|-----------|-----------------------------------------------------------|------------|---------------------------------|------------|
| mp023-001 | Verificar docker-compose.ci.yml                           | ✅ DONE    | —                               | LOW        |
| mp023-002 | Tipos DesignInput/Report + render_canonical + markers     | ✅ DONE    | —                               | MED        |
| mp023-003 | merge_preserve + apply_design + fixtures golden           | ✅ DONE    | mp023-002                       | HIGH       |
| mp023-004 | CLI dare design + interactive + smokes                    | ✅ DONE    | mp023-003                       | MED        |
| mp023-005 | Capability matrix cli_commands design                     | ✅ DONE    | —                               | LOW        |
| mp023-006 | Docs cli-design.md + DEC-024                              | ✅ DONE    | mp023-002                       | LOW        |
| mp023-007 | Auditoria Ralph (test/clippy/audit/deny)                  | ✅ DONE    | mp023-001, mp023-004, mp023-005, mp023-006 | MED |
| mp023-008 | Fechamento microplano 023                                 | ✅ DONE    | mp023-007                       | LOW        |

## Próximas Etapas

1. ~~Revisar e aprovar este TASKS + DAG~~
2. ~~Executar DAG `mp023-*`~~
3. **Próximo microplano:** `024-fundacao-de-enrichment-por-ia`
