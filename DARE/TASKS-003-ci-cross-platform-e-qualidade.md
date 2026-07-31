# Tasks: CI cross-platform e qualidade (Microplano 003)

> **Fonte:** `DARE/BLUEPRINT-003-ci-cross-platform-e-qualidade.md`  
> **DAG:** `DARE/dare-dag-003.yaml`  
> **Specs:** `DARE/EXECUTION-003/`  
> **IDs:** `mp003-*`  
> **Não substitui:** `DARE/TASKS.md` (001) nem `TASKS-002-*` (002)

## Visão Geral

- **Total de Tasks:** 8
- **Ranks paralelos:** 4 (Kahn)
- **Tempo estimado:** ~4–6 h
- **Progresso:** 8/8 (100%)

```
████████████████████ 100%
```

## Tabela de Status

| ID        | Título                                              | Status   | Depends On                         | Complexity |
|-----------|-----------------------------------------------------|----------|------------------------------------|------------|
| mp003-001 | docker-compose.ci.yml (Fase 1)                      | ✅ DONE  | —                                  | LOW        |
| mp003-002 | deny.toml + cargo deny check                        | ✅ DONE  | —                                  | MED        |
| mp003-003 | Scripts smoke Unix/Windows                          | ✅ DONE  | —                                  | LOW        |
| mp003-004 | Workflow ci.yml (fmt/clippy/test/audit/deny)        | ✅ DONE  | mp003-002                          | HIGH       |
| mp003-005 | Workflow build.yml (matrix 5 + smoke + SHA256)      | ✅ DONE  | mp003-003                          | HIGH       |
| mp003-006 | Docs CI + DEC-004                                   | ✅ DONE  | mp003-004, mp003-005               | MED        |
| mp003-007 | Auditoria RS-* + audit/deny final                   | ✅ DONE  | mp003-002, mp003-004, mp003-005    | HIGH       |
| mp003-008 | Remover rust-workspace-002 + fechamento 003         | ✅ DONE  | mp003-001, mp003-006, mp003-007    | MED        |

## Tarefas por Fase (Blueprint)

### Fase 1 — Containerização
- mp003-001 ✅

### Fase 2 — deny + smoke
- mp003-002, mp003-003 ✅

### Fase 3–4 — Workflows
- mp003-004, mp003-005 ✅

### Fase 5 — Docs
- mp003-006 ✅

### Fase 6 — Segurança (N-1)
- mp003-007 ✅

### Fase 7 — Fechamento (N)
- mp003-008 ✅

## Próximo

Microplano **004** — erros, tracing e saída da CLI.
