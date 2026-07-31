# Tasks: Execute agent — mock, worktrees e budget (030)

> **Fonte:** `DARE/BLUEPRINT-030-execute-agent-mock-worktrees-e-budget.md`  
> **Design:** `DARE/DESIGN-030-execute-agent-mock-worktrees-e-budget.md`  
> **DAG:** `DARE/dare-dag-030.yaml`  
> **Specs:** `DARE/EXECUTION-030/`  
> **Progresso:** 8/8 (100%)  
> **Nota:** IDs `mp030-*`; crate `dare-agent` + CLI `--agent --driver mock`; **fora** drivers 031 / decay / guard 034

## Visão Geral

- Total de Tasks: 8
- Ranks paralelos: 7 (rank 0: 2 tasks — `mp030-001`, `mp030-002`)
- Tempo estimado: ~14–18 h

## Tabela de Status

| ID        | Título                                              | Status  | Depends On                      | Complexity |
|-----------|-----------------------------------------------------|---------|---------------------------------|------------|
| mp030-001 | Verificar docker-compose.ci.yml                     | ✅ DONE | —                               | LOW        |
| mp030-002 | Crate dare-agent (driver/mock/budget/policy/sig)    | ✅ DONE | —                               | HIGH       |
| mp030-003 | WorktreeManager (git worktree jail)                 | ✅ DONE | mp030-002                       | HIGH       |
| mp030-004 | CLI `--agent` loop + Ralph on Done                  | ✅ DONE | mp030-003                       | HIGH       |
| mp030-005 | CLI `--cleanup-worktrees` + recovery                | ✅ DONE | mp030-004                       | MED        |
| mp030-006 | Capability + cli-execute-agent.md + DEC-031         | ✅ DONE | mp030-005                       | MED        |
| mp030-007 | Auditoria Ralph (fmt/clippy/test/audit)             | ✅ DONE | mp030-001, mp030-005, mp030-006 | MED        |
| mp030-008 | Fechamento TASKS/matriz/Blueprint                   | ✅ DONE | mp030-007                       | LOW        |

## Progresso

```
████████████████████ 100%
```

## Entrega

- Crate `dare-agent` (driver/mock/budget/policy/signature/worktree)
- CLI `dare execute --agent` + `--cleanup-worktrees`
- Docs: `docs/compatibility/cli-execute-agent.md` + **DEC-031**
