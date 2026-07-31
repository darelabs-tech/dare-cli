# Tasks: GraphRAG — storage e compatibilidade (040)

> **Fonte:** `DARE/BLUEPRINT-040-graphrag-storage-e-compatibilidade.md`  
> **Design:** `DARE/DESIGN-040-graphrag-storage-e-compatibilidade.md`  
> **DAG:** `DARE/dare-dag-040.yaml`  
> **Specs:** `DARE/EXECUTION-040/`  
> **Progresso:** 8/8 (100%)  
> **Nota:** IDs `mp040-*`; **DONE** — crate `dare-graph`; próximo **041**

## Visão Geral

- Total de Tasks: 8
- Ranks: 8
- Tempo estimado: ~10–14 h

## Tabela de Status

| ID        | Título                                         | Status  | Depends On   | Complexity |
|-----------|------------------------------------------------|---------|--------------|------------|
| mp040-001 | Workspace member dare-graph + rusqlite         | ✅ DONE | —            | LOW        |
| mp040-002 | types + ids canônicos + vector f32 LE          | ✅ DONE | mp040-001    | MED        |
| mp040-003 | KnowledgeGraph trait + migrations versionadas  | ✅ DONE | mp040-002    | MED        |
| mp040-004 | SqliteGraph + testes legado                    | ✅ DONE | mp040-003    | HIGH       |
| mp040-005 | JsonGraph + contract tests JSON↔SQLite         | ✅ DONE | mp040-004    | HIGH       |
| mp040-006 | Config/factory + path safety                   | ✅ DONE | mp040-005    | MED        |
| mp040-007 | Docs graphrag-storage + DEC-036 + matriz       | ✅ DONE | mp040-006    | MED        |
| mp040-008 | Ralph Loop + fechamento artefatos              | ✅ DONE | mp040-007    | MED        |

## Progresso

```
████████████████████ 100%
```

## Próximas Etapas

1. Microplano **041** — ingest, keyword, BFS e RRF
