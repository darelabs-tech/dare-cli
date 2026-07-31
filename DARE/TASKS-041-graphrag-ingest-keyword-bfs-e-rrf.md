# Tasks: GraphRAG — ingest, keyword, BFS e RRF (041)

> **Fonte:** `DARE/BLUEPRINT-041-graphrag-ingest-keyword-bfs-e-rrf.md`  
> **Design:** `DARE/DESIGN-041-graphrag-ingest-keyword-bfs-e-rrf.md`  
> **DAG:** `DARE/dare-dag-041.yaml`  
> **Specs:** `DARE/EXECUTION-041/`  
> **Progresso:** 5/5 (100%)  
> **Nota:** IDs `mp041-*`; **DONE** — ingest/search + CLI graph; próximo **042**

## Visão Geral

- Total de Tasks: 5
- Ranks: 5
- Tempo estimado: ~10–14 h

## Tabela de Status

| ID        | Título                                              | Status  | Depends On   | Complexity |
|-----------|-----------------------------------------------------|---------|--------------|------------|
| mp041-001 | ingest.rs contentHash + símbolos regex              | ✅ DONE | —            | HIGH       |
| mp041-002 | search.rs keyword/BFS/RRF + golden rankings         | ✅ DONE | mp041-001    | HIGH       |
| mp041-003 | CLI dare graph ingest\|query\|stats\|viz + smokes   | ✅ DONE | mp041-002    | HIGH       |
| mp041-004 | Docs graphrag-ingest + DEC-042 + matriz             | ✅ DONE | mp041-003    | MED        |
| mp041-005 | Ralph Loop + fechamento artefatos                   | ✅ DONE | mp041-004    | MED        |

## Progresso

```
████████████████████ 100%
```

## Próximas Etapas

1. Microplano **042** — GraphRAG semântico opcional
