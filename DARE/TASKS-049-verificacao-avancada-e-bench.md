# Tasks: Verificação avançada e bench (049)

> **Fonte:** `DARE/BLUEPRINT-049-verificacao-avancada-e-bench.md` (APPROVED via `/dare-tasks`)  
> **Design:** `DARE/DESIGN-049-verificacao-avancada-e-bench.md`  
> **DAG:** `DARE/dare-dag-049.yaml`  
> **Specs:** `DARE/EXECUTION-049/`  
> **DEC:** DEC-050  
> **Progresso:** 8/8 (100%)

## Visão Geral

- Total de Tasks: 8
- Ranks: 0 (001 ∥ 002) → 1 (003 ∥ 004 ∥ 007) → 2 (005) → 3 (006) → 4 (008)
- Tempo estimado: ~16–22 h
- Escopo: pós-Ralph advanced verify · mutation/formal · best-of/decay · `dare bench` · DEC-050

## Tabela de Status

| ID        | Título                                              | Status  | Depends On                | Complexity |
|-----------|-----------------------------------------------------|---------|---------------------------|------------|
| mp049-001 | reports + fail-to-pass + anti-tamper                | ✅ DONE | —                         | HIGH       |
| mp049-002 | bench FixRate + suite loader + fixtures             | ✅ DONE | —                         | HIGH       |
| mp049-003 | mutation adapters + threshold 0.70                  | ✅ DONE | mp049-001                 | HIGH       |
| mp049-004 | formal backends + repair loop                       | ✅ DONE | mp049-001                 | HIGH       |
| mp049-005 | run_advanced_verify + execute flags                 | ✅ DONE | mp049-003, mp049-004      | HIGH       |
| mp049-006 | best-of-N + Pareto + decay policy                   | ✅ DONE | mp049-005                 | HIGH       |
| mp049-007 | CLI dare bench + smokes                             | ✅ DONE | mp049-002                 | MED        |
| mp049-008 | Docs DEC-050 + capabilities + Ralph                 | ✅ DONE | mp049-005, 006, 007       | MED        |

## Progresso

```
████████████████████ 100%
```

## Tarefas por Rank

### Rank 0 (paralelo)
- mp049-001 — reports + ftp + anti-tamper ✅
- mp049-002 — bench FixRate + fixtures ✅

### Rank 1 (paralelo)
- mp049-003 — mutation (← 001) ✅
- mp049-004 — formal + repair (← 001) ✅
- mp049-007 — CLI bench (← 002) ✅

### Rank 2
- mp049-005 — advanced verify + execute flags (← 003, 004) ✅

### Rank 3
- mp049-006 — best-of + decay (← 005) ✅

### Rank 4
- mp049-008 — docs + DEC-050 + Ralph (← 005, 006, 007) ✅

## Caminho crítico

`001 → 003 → 005 → 006 → 008` (002 → 007 e 004 em paralelo)

## Fechamento

Microplano **049** concluído: DEC-050, docs `cli-verify-bench.md`, capability `dare-bench`→`bench`, matriz 049 Concluído, Ralph verde.
