# Tasks: Dashboard e REST compatível (051)

> **Fonte:** `DARE/BLUEPRINT-051-dashboard-e-rest-compativel.md` (APPROVED via `/dare-tasks`)  
> **Design:** `DARE/DESIGN-051-dashboard-e-rest-compativel.md`  
> **DAG:** `DARE/dare-dag-051.yaml`  
> **Specs:** `DARE/EXECUTION-051/`  
> **DEC:** DEC-052  
> **Progresso:** 7/7 (100%)

## Visão Geral

- Total de Tasks: 7
- Ranks: 0 (001 ∥ 002) → 1 (003 ∥ 004) → 2 (005) → 3 (006) → 4 (007)
- Tempo estimado: ~12–16 h
- Escopo: crate `dare-server` (Axum) · dashboard read-only · REST legado · CLI `dare dashboard` + `dare server --protocol rest` · DEC-052

## Tabela de Status

| ID        | Título                                              | Status  | Depends On          | Complexity |
|-----------|-----------------------------------------------------|---------|---------------------|------------|
| mp051-001 | Crate dare-server + config/auth/middleware/health   | ✅ DONE | —                   | HIGH       |
| mp051-002 | Dashboard static assets (HTML/CSS/JS)               | ✅ DONE | —                   | LOW        |
| mp051-003 | Dashboard routes + embed + telemetry                | ✅ DONE | mp051-001, 002      | HIGH       |
| mp051-004 | REST routes (tools/context/dag/tasks/graph/…)       | ✅ DONE | mp051-001           | HIGH       |
| mp051-005 | serve + open_browser + graceful shutdown            | ✅ DONE | mp051-001, 003      | MED        |
| mp051-006 | CLI dare dashboard/server + http contract smokes    | ✅ DONE | mp051-003, 004, 005 | HIGH       |
| mp051-007 | Docs DEC-052 + capability + Ralph                   | ✅ DONE | mp051-006           | MED        |

## Progresso

```
████████████████████ 100%
```

## Tarefas por Rank

### Rank 0 (paralelo)
- mp051-001 — crate + config/auth/middleware/health ✅
- mp051-002 — assets dashboard ✅

### Rank 1 (paralelo)
- mp051-003 — dashboard + telemetry (← 001, 002) ✅
- mp051-004 — REST routes (← 001) ✅

### Rank 2
- mp051-005 — serve/browser/shutdown (← 001, 003) ✅

### Rank 3
- mp051-006 — CLI + http contracts (← 003, 004, 005) ✅

### Rank 4
- mp051-007 — docs + DEC-052 + Ralph (← 006) ✅

## Caminho crítico

`001 → 003 → 005 → 006 → 007` (002 ∥ 001; 004 ∥ 003) — **completo**

## Ready agora

🟢 Microplano **051** fechado. Próximo: **052** (MCP real).

## Próximas Etapas

1. Microplano **052** — MCP real como transporte separado (ADR-004)
