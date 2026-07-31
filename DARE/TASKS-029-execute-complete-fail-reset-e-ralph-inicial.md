# Tasks: Execute — complete, fail, reset e Ralph inicial (029)

> **Fonte:** `DARE/BLUEPRINT-029-execute-complete-fail-reset-e-ralph-inicial.md`  
> **Design:** `DARE/DESIGN-029-execute-complete-fail-reset-e-ralph-inicial.md`  
> **DAG:** `DARE/dare-dag-029.yaml`  
> **Specs:** `DARE/EXECUTION-029/`  
> **Progresso:** 8/8 (100%)  
> **Nota:** IDs `mp029-*`; **DONE** — `dare-verify` + CLI `--complete|--fail|--reset`; próximo **030**

## Visão Geral

- Total de Tasks: 8
- Ranks paralelos: 7 (rank 0: 2 tasks — `mp029-001`, `mp029-002`)
- Tempo estimado: ~12–16 h

## Tabela de Status

| ID        | Título                                              | Status     | Depends On                         | Complexity |
|-----------|-----------------------------------------------------|------------|------------------------------------|------------|
| mp029-001 | Verificar docker-compose.ci.yml                     | ✅ DONE    | —                                  | LOW        |
| mp029-002 | Crate dare-verify (stacks + run_ralph)              | ✅ DONE    | —                                  | HIGH       |
| mp029-003 | Verification writer (.dare/verification)            | ✅ DONE    | mp029-002                          | MED        |
| mp029-004 | CLI `--complete` + smokes Ralph mock                | ✅ DONE    | mp029-003                          | HIGH       |
| mp029-005 | CLI `--fail` + `--reset` + smokes                   | ✅ DONE    | mp029-004                          | MED        |
| mp029-006 | Capability + cli-execute-mutations.md + DEC-030     | ✅ DONE    | mp029-005                          | MED        |
| mp029-007 | Auditoria Ralph (fmt/clippy/test/audit)             | ✅ DONE    | mp029-001, mp029-005, mp029-006    | MED        |
| mp029-008 | Fechamento TASKS/matriz/Blueprint                   | ✅ DONE    | mp029-007                          | LOW        |

## Tarefas por Fase

### Phase 1: Container
- mp029-001

### Phase 2: dare-verify core
- mp029-002

### Phase 3: Verification FS
- mp029-003 (deps: 002)

### Phase 4: CLI complete
- mp029-004 (deps: 003)

### Phase 5: CLI fail/reset
- mp029-005 (deps: 004)

### Phase 6: Docs + capability
- mp029-006 (deps: 005)

### Phase 7–8: Audit + closeout
- mp029-007 → mp029-008

## Progresso

```
████████████████████ 100%
```

## Próximas Etapas

1. Microplano **030** — `dare execute --agent` mock / worktrees / budget
2. Comando: ver `DARE-RUST-MICRO-PLANOS/.../030-execute-agent-mock-worktrees-e-budget.md`
