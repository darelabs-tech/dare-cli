# Tasks: GraphRAG semântico opcional (042)

> **Fonte:** `DARE/BLUEPRINT-042-graphrag-semantico-opcional.md` (APPROVED)  
> **Design:** `DARE/DESIGN-042-graphrag-semantico-opcional.md`  
> **DAG:** `DARE/dare-dag-042.yaml`  
> **Specs:** `DARE/EXECUTION-042/`  
> **DEC:** DEC-045  
> **Progresso:** 6/6 (100%)

## Visão Geral

- Total de Tasks: 6
- Ranks paralelos: 5 (rank 0: 2 tasks)
- Tempo estimado: ~10–14 h
- Escopo: feature `semantic` + MiniLM local + RRF 3 canais + fallback 041; **sem** Neo4j

## Tabela de Status

| ID        | Título                                                              | Status  | Depends On           | Complexity |
|-----------|---------------------------------------------------------------------|---------|----------------------|------------|
| mp042-001 | cosine + SearchOptions.no_semantic + hybrid_query_with_warnings     | ✅ DONE | —                    | MED        |
| mp042-002 | semantic.rs fastembed + cache + ensure_model + doctor types         | ✅ DONE | —                    | HIGH       |
| mp042-003 | Wire vector_rank + 3-list RRF + fallback warnings                   | ✅ DONE | mp042-001, mp042-002 | HIGH       |
| mp042-004 | CLI `--no-semantic` + doctor + enable + smokes                      | ✅ DONE | mp042-003            | MED        |
| mp042-005 | Docs graphrag-semantic + DEC-045 + matriz 042                       | ✅ DONE | mp042-004            | LOW        |
| mp042-006 | Ralph dual-feature + audit close                                    | ✅ DONE | mp042-005            | MED        |

## Progresso

```
████████████████████ 100%
```

## Tarefas por Rank

### Rank 0 (paralelo)
- mp042-001 · mp042-002

### Rank 1
- mp042-003

### Rank 2–4
- mp042-004 → mp042-005 → mp042-006

## Ready agora

✅ Microplano 042 concluído (6/6).

## Próximas Etapas

1. `/apply-worktree` para merge-back
2. Review humano antes de merge na branch principal
