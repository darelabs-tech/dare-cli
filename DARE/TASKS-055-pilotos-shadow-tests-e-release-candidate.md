# Tasks: Pilotos, shadow tests e release candidate (055)

> **Fonte:** `DARE/BLUEPRINT-055-pilotos-shadow-tests-e-release-candidate.md` (APPROVED via `/dare-tasks`)  
> **Design:** `DARE/DESIGN-055-pilotos-shadow-tests-e-release-candidate.md`  
> **DAG:** `DARE/dare-dag-055.yaml`  
> **Specs:** `DARE/EXECUTION-055/`  
> **DEC:** DEC-056  
> **RC tag:** `v4.0.0-rc1`  
> **Progresso:** 7/7 (100%)

## Visão Geral

- Total de Tasks: 7
- Ranks: 0 (001 ∥ 004) → 1 (002) → 2 (003) → 3 (005) → 4 (006) → 5 (007)
- Tempo estimado: ~10–14 h (+ janela shadow / release humano se GH bloquear)
- Escopo: pilotos · shadow isolado · freeze TS/contrato · RC **v4.0.0-rc1** · rollback · DEC-056 · **sem** capability nova

## Tabela de Status

| ID        | Título                                         | Status  | Depends On      | Complexity |
|-----------|------------------------------------------------|---------|-----------------|------------|
| mp055-001 | Pilot inventory + synthetic fixtures           | ✅ DONE | —               | MED        |
| mp055-002 | Shadow playbook + pilot-shadow scripts         | ✅ DONE | mp055-001       | HIGH       |
| mp055-003 | Run ≥3 shadow cycles + incidents log           | ✅ DONE | mp055-002       | HIGH       |
| mp055-004 | TypeScript freeze + contract freeze docs       | ✅ DONE | —               | LOW        |
| mp055-005 | Publish RC v4.0.0-rc1 + notes + smoke          | ✅ DONE | mp055-003, 004  | HIGH       |
| mp055-006 | Rollback drill PASS                            | ✅ DONE | mp055-005       | MED        |
| mp055-007 | DEC-056 + matriz + Ralph close                 | ✅ DONE | mp055-006       | MED        |

## Progresso

```
████████████████████ 100%
```

## Tarefas por Rank

### Rank 0 (paralelo)
- mp055-001 — inventário + fixtures
- mp055-004 — freeze TS + contrato

### Rank 1
- mp055-002 — playbook + scripts (← 001)

### Rank 2
- mp055-003 — 3 ciclos + incidents (← 002)

### Rank 3
- mp055-005 — RC v4.0.0-rc1 (← 003, 004)

### Rank 4
- mp055-006 — rollback drill (← 005)

### Rank 5
- mp055-007 — DEC-056 + Ralph (← 006)

## Caminho crítico

`001 → 002 → 003 → 005 → 006 → 007` (004 ∥ rank0 → junta em 005)

## Ready agora

🟢 Microplano **055** fechado (7/7 DONE). Próximo: **056** (cutover / stable / encerramento do legado).

## Próximas Etapas

1. Merge-back via `/apply-worktree` quando aprovado
2. Avançar microplano **056** na matriz 000A (cutover ≠ DEC-056)
