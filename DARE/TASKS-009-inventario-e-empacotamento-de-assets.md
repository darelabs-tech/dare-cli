# Tasks: Inventário e empacotamento de assets (009)

> **Fonte:** `DARE/BLUEPRINT-009-inventario-e-empacotamento-de-assets.md`  
> **DAG:** `DARE/dare-dag-009.yaml`  
> **Specs:** `DARE/EXECUTION-009/`  
> **Progresso:** 7/7 (100%)

## Tabela de Status

| ID        | Título                                            | Status  | Depends On           | Complexity |
|-----------|---------------------------------------------------|---------|----------------------|------------|
| mp009-001 | Verificar docker-compose.ci.yml                   | ✅ DONE | —                    | LOW        |
| mp009-002 | Harden path + materialize errors                  | ✅ DONE | —                    | MED        |
| mp009-003 | Freshness hashes + templates/README               | ✅ DONE | mp009-002            | MED        |
| mp009-004 | Scripts regen manifest + sync espelho             | ✅ DONE | mp009-002            | MED        |
| mp009-005 | CLI smoke assets verify + docs DEC-010            | ✅ DONE | mp009-003, mp009-004 | MED        |
| mp009-006 | Auditoria Ralph (test/clippy/audit/deny)          | ✅ DONE | mp009-001, mp009-005 | MED        |
| mp009-007 | Fechamento microplano 009                         | ✅ DONE | mp009-006            | LOW        |

## Próximo

Microplano **010** — modelo canónico de capabilities.
