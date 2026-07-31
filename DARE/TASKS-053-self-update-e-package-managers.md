# Tasks: Self-update e package managers (053)

> **Fonte:** `DARE/BLUEPRINT-053-self-update-e-package-managers.md` (APPROVED via `/dare-tasks`)  
> **Design:** `DARE/DESIGN-053-self-update-e-package-managers.md`  
> **DAG:** `DARE/dare-dag-053.yaml`  
> **Specs:** `DARE/EXECUTION-053/`  
> **DEC:** DEC-054  
> **Progresso:** 7/7 (100%)

## Visão Geral

- Total de Tasks: 7
- Ranks: 0 (001 ∥ 006) → 1 (002) → 2 (003) → 3 (004) → 4 (005) → 5 (007)
- Tempo estimado: ~12–16 h
- Escopo: crate `dare-self` · `dare self update|rollback|uninstall` · SHA-256 + cosign fail-closed · Homebrew + WinGet · DEC-054

## Tabela de Status

| ID        | Título                                           | Status  | Depends On      | Complexity |
|-----------|--------------------------------------------------|---------|-----------------|------------|
| mp053-001 | Crate dare-self: paths + lock + channel          | ✅ DONE | —               | MED        |
| mp053-002 | plan + download + SHA-256 verify                 | ✅ DONE | mp053-001       | HIGH       |
| mp053-003 | SignatureVerifier + apply atomic + backup        | ✅ DONE | mp053-002       | HIGH       |
| mp053-004 | rollback + uninstall                             | ✅ DONE | mp053-003       | MED        |
| mp053-005 | CLI dare self + smokes                           | ✅ DONE | mp053-004       | HIGH       |
| mp053-006 | Packaging Homebrew + WinGet                      | ✅ DONE | —               | MED        |
| mp053-007 | Docs DEC-054 + capability + Ralph                | ✅ DONE | mp053-005, 006  | MED        |

## Progresso

```
████████████████████ 100%
```

## Tarefas por Rank

### Rank 0 (paralelo)
- mp053-001 — crate skeleton + lock + channel ✅
- mp053-006 — packaging Homebrew + WinGet ✅

### Rank 1
- mp053-002 — plan + download + sha256 (← 001) ✅

### Rank 2
- mp053-003 — cosign verifier + apply (← 002) ✅

### Rank 3
- mp053-004 — rollback + uninstall (← 003) ✅

### Rank 4
- mp053-005 — CLI `dare self` (← 004) ✅

### Rank 5
- mp053-007 — docs + DEC-054 + Ralph (← 005, 006) ✅

## Caminho crítico

`001 → 002 → 003 → 004 → 005 → 007` (006 ∥ rank0) — **completo**

## Ready agora

✅ Microplano **053** fechado (DEC-054). Próximo: **054**.

## Próximas Etapas

1. `/apply-worktree` / merge após review humano
2. Microplano **054** (hardening)
