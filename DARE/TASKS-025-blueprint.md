# Tasks: Comando `dare blueprint` (025)

> **Fonte:** `DARE/BLUEPRINT-025-blueprint.md`  
> **Design:** `DARE/DESIGN-025-blueprint.md`  
> **DAG:** `DARE/dare-dag-025.yaml`  
> **Specs:** `DARE/EXECUTION-025/`  
> **Progresso:** 8/8 (100%)  
> **Nota:** IDs `mp025-*`; **DONE** — `dare blueprint`; próximo **026**

## Visão Geral

- Total de Tasks: 8
- Ranks paralelos: 5 (rank 0: 3 tasks)
- Tempo estimado: ~10–14 h

## Tabela de Status

| ID        | Título                                                    | Status     | Depends On                      | Complexity |
|-----------|-----------------------------------------------------------|------------|---------------------------------|------------|
| mp025-001 | Verificar docker-compose.ci.yml                           | ✅ DONE    | —                               | LOW        |
| mp025-002 | generate_bundle + heurística Design→tasks                 | ✅ DONE    | —                               | HIGH       |
| mp025-003 | Staging + validate_dag + promote keep/force               | ✅ DONE    | mp025-002                       | HIGH       |
| mp025-004 | CLI dare blueprint + AI soft + smokes                     | ✅ DONE    | mp025-003                       | MED        |
| mp025-005 | Capability matrix cli_commands blueprint                  | ✅ DONE    | —                               | LOW        |
| mp025-006 | Docs cli-blueprint.md + DEC-026                           | ✅ DONE    | mp025-002                       | LOW        |
| mp025-007 | Auditoria Ralph (test/clippy/audit/deny)                  | ✅ DONE    | mp025-001, mp025-004, mp025-005, mp025-006 | MED |
| mp025-008 | Fechamento microplano 025                                 | ✅ DONE    | mp025-007                       | LOW        |

## Próximas Etapas

1. ~~Revisar e aprovar este TASKS + DAG~~
2. ~~Executar DAG `mp025-*`~~
3. **Próximo microplano:** `026-dag-parser-ranks-e-state-store`
