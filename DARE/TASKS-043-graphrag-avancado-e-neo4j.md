# Tasks: GraphRAG avançado + Neo4j (043)

> **Fonte:** `DARE/BLUEPRINT-043-graphrag-avancado-e-neo4j.md` (APPROVED)  
> **Design:** `DARE/DESIGN-043-graphrag-avancado-e-neo4j.md`  
> **DAG:** `DARE/dare-dag-043.yaml`  
> **Specs:** `DARE/EXECUTION-043/`  
> **DEC:** DEC-046  
> **Progresso:** 6/6 (100%)

## Visão Geral

- Total de Tasks: 6
- Ranks: 0 (4 paralelas) → 1 (CLI) → 2 (docs/Ralph)
- Tempo estimado: ~12–16 h
- Escopo: locate/owners/impact/trace/drift + exit 7 + Neo4j HTTP opt-in

## Tabela de Status

| ID        | Título                                                    | Status  | Depends On                     | Complexity |
|-----------|-----------------------------------------------------------|---------|--------------------------------|------------|
| mp043-001 | advanced locate + owners + tests                          | ✅ DONE | —                              | HIGH       |
| mp043-002 | advanced impact + trace + tests                           | ✅ DONE | —                              | HIGH       |
| mp043-003 | advanced drift + threshold helper                         | ✅ DONE | —                              | MED        |
| mp043-004 | neo4j feature + HTTP client + config                      | ✅ DONE | —                              | HIGH       |
| mp043-005 | CLI locate/owners/impact/trace/drift + exit 7 smokes      | ✅ DONE | mp043-001, 002, 003            | MED        |
| mp043-006 | Docs DEC-046 + matriz + Ralph close                       | ✅ DONE | mp043-004, mp043-005           | MED        |

## Progresso

```
████████████████████ 100%
```

## Tarefas por Rank

### Rank 0 (paralelo)
- mp043-001 · mp043-002 · mp043-003 · mp043-004

### Rank 1
- mp043-005

### Rank 2
- mp043-006

## Ready agora

✅ Microplano 043 concluído (6/6).

## Próximas Etapas

Avançar para microplanos posteriores na matriz (ex.: 044+ já concluídos em paralelo; próximos pendentes conforme `000A-MATRIZ-DE-STATUS.md`).
