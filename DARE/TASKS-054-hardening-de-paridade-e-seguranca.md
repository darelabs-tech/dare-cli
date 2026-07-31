# Tasks: Hardening de paridade e segurança (054)

> **Fonte:** `DARE/BLUEPRINT-054-hardening-de-paridade-e-seguranca.md` (APPROVED via `/dare-tasks`)  
> **Design:** `DARE/DESIGN-054-hardening-de-paridade-e-seguranca.md`  
> **DAG:** `DARE/dare-dag-054.yaml`  
> **Specs:** `DARE/EXECUTION-054/`  
> **DEC:** DEC-055  
> **Progresso:** 7/7 (100%)

## Visão Geral

- Total de Tasks: 7
- Ranks: 0 (001 ∥ 006) → 1 (002 ∥ 004 ∥ 005) → 2 (003) → 3 (007)
- Tempo estimado: ~10–14 h
- Escopo: crate `dare-parity` · golden/security/cross-platform · normalizer · perf 15% · DEC-055 · **sem** capability nova

## Tabela de Status

| ID        | Título                                              | Status  | Depends On           | Complexity |
|-----------|-----------------------------------------------------|---------|----------------------|------------|
| mp054-001 | Scaffold dare-parity + CaseSpec + tests/ layout     | ✅ DONE | —                    | MED        |
| mp054-002 | Normalizer N-01..N-08 + anti over-normalize         | ✅ DONE | mp054-001            | HIGH       |
| mp054-003 | Golden runner + fixtures + diff-log                 | ✅ DONE | mp054-002            | HIGH       |
| mp054-004 | Security suite injection/leak/archive/sig/bidi      | ✅ DONE | mp054-001            | HIGH       |
| mp054-005 | proptest paths/parsers + cross-platform               | ✅ DONE | mp054-001            | MED        |
| mp054-006 | Perf scripts + baseline stub + diff-log skeleton    | ✅ DONE | —                    | MED        |
| mp054-007 | Docs DEC-055 + CI paths + Ralph                     | ✅ DONE | 003, 004, 005, 006   | MED        |

## Progresso

```
████████████████████ 100%
```

## Tarefas por Rank

### Rank 0 (paralelo)
- mp054-001 — scaffold crate + CaseSpec + perf gate math
- mp054-006 — scripts measure-perf + baseline/diff-log stubs (não toca crate)

### Rank 1 (paralelo após 001)
- mp054-002 — normalizer (← 001)
- mp054-004 — security suite (← 001)
- mp054-005 — proptest + cross-platform (← 001)

### Rank 2
- mp054-003 — golden runner (← 002)

### Rank 3
- mp054-007 — docs + DEC-055 + Ralph (← 003, 004, 005, 006)

## Caminho crítico

`001 → 002 → 003 → 007` (004 ∥ 005 ∥ 006 alimentam 007)

## Ready agora

🟢 Microplano **054** fechado (7/7 DONE). Próximo: **055** (pilotos / shadow / RC).

## Próximas Etapas

1. Merge-back via `/apply-worktree` quando aprovado
2. Avançar microplano **055** na matriz 000A
