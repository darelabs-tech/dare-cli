# Tasks: Comandos `dare ai` (050)

> **Fonte:** `DARE/BLUEPRINT-050-comandos-ai.md` (APPROVED via `/dare-tasks`)  
> **Design:** `DARE/DESIGN-050-comandos-ai.md`  
> **DAG:** `DARE/dare-dag-050.yaml`  
> **Specs:** `DARE/EXECUTION-050/`  
> **DEC:** DEC-051  
> **Progresso:** 7/7 (100%)

## Visão Geral

- Total de Tasks: 7
- Ranks: 0 (001 ∥ 002) → 1 (003 ∥ 004) → 2 (005 ∥ 006) → 3 (007)
- Tempo estimado: ~10–14 h
- Escopo: `dare ai doctor|providers|run|prompt` · registry design+blueprint · write opt-in · SHOULD providers · DEC-051

## Tabela de Status

| ID        | Título                                              | Status  | Depends On           | Complexity |
|-----------|-----------------------------------------------------|---------|----------------------|------------|
| mp050-001 | command_registry design+blueprint                   | ✅ DONE | —                    | MED        |
| mp050-002 | capabilities + doctor statuses                      | ✅ DONE | —                    | HIGH       |
| mp050-003 | prompt preview + redact (no env leak)               | ✅ DONE | mp050-001            | HIGH       |
| mp050-004 | run_enrich + --write opt-in                         | ✅ DONE | mp050-001            | HIGH       |
| mp050-005 | CLI dare ai + ai_cli smokes                         | ✅ DONE | mp050-002, 003, 004  | HIGH       |
| mp050-006 | SHOULD providers claude/cursor/antigravity          | ✅ DONE | mp050-002            | MED        |
| mp050-007 | Docs DEC-051 + capability + Ralph                   | ✅ DONE | mp050-005, 006       | MED        |

## Progresso

```
████████████████████ 100%
```

## Tarefas por Rank

### Rank 0 (paralelo)
- mp050-001 — command_registry ✅
- mp050-002 — capabilities + doctor ✅

### Rank 1 (paralelo)
- mp050-003 — prompt (← 001) ✅
- mp050-004 — run_enrich (← 001) ✅

### Rank 2 (paralelo)
- mp050-005 — CLI + smokes (← 002, 003, 004) ✅
- mp050-006 — SHOULD text providers (← 002) ✅

### Rank 3
- mp050-007 — docs + DEC-051 + Ralph (← 005, 006) ✅

## Caminho crítico

`001 → 003 → 005 → 007` (002 → 006 e 004 em paralelo)

## Ready agora

🟢 **nenhuma** — microplano 050 fechado (7/7)

## Próximas Etapas

1. Merge worktrees via `/apply-worktree`
2. Avançar para microplano **051** (dashboard/REST)
