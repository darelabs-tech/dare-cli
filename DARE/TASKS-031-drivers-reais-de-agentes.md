# Tasks: Drivers reais de agentes (031)

> **Fonte:** `DARE/BLUEPRINT-031-drivers-reais-de-agentes.md`  
> **Design:** `DARE/DESIGN-031-drivers-reais-de-agentes.md`  
> **DAG:** `DARE/dare-dag-031.yaml`  
> **Specs:** `DARE/EXECUTION-031/`  
> **Progresso:** 8/8 (100%)  
> **Nota:** IDs `mp031-*`; estende `dare-agent` com drivers reais; **fora** decay 033 / SDK Anthropic / best-of-N 049 / `dare ai` 050

## Visão Geral

- Total de Tasks: 8
- Ranks paralelos: 6 (rank 0: `mp031-001`∥`mp031-002`; rank 1: `mp031-003`∥`mp031-004`)
- Tempo estimado: ~16–22 h

## Tabela de Status

| ID        | Título                                              | Status  | Depends On                      | Complexity |
|-----------|-----------------------------------------------------|---------|---------------------------------|------------|
| mp031-001 | Verificar docker-compose.ci.yml                     | ✅ DONE | —                               | LOW        |
| mp031-002 | drivers/argv + common finalize (redact/truncate)    | ✅ DONE | —                               | MED        |
| mp031-003 | CodexDriver JSONL + suite comum                     | ✅ DONE | mp031-002                       | HIGH       |
| mp031-004 | Claude/Cursor/Antigravity drivers + suite           | ✅ DONE | mp031-002                       | HIGH       |
| mp031-005 | resolve_driver + smokes CLI                         | ✅ DONE | mp031-003, mp031-004            | HIGH       |
| mp031-006 | Docs cli-execute-agent + DEC-037 + matriz           | ✅ DONE | mp031-005                       | MED        |
| mp031-007 | Auditoria Ralph (fmt/clippy/test/audit)             | ✅ DONE | mp031-001, mp031-005, mp031-006 | MED        |
| mp031-008 | Fechamento TASKS/Blueprint/matriz                   | ✅ DONE | mp031-007                       | LOW        |

## Progresso

```
████████████████████ 100%
```

## Tarefas por Fase (Blueprint §6)

| Fase | Tasks |
|------|-------|
| 1 Container/CI | mp031-001 |
| 2 Argv + common | mp031-002 |
| 3 Codex | mp031-003 |
| 4 Claude/Cursor/Antigravity | mp031-004 |
| 5 resolve + smokes | mp031-005 |
| 6 Docs + DEC | mp031-006 |
| 7 Auditoria | mp031-007 |
| 8 Fechamento | mp031-008 |

## Entrega

- `crates/dare-agent/src/drivers/**` (argv, common, codex, claude, cursor, antigravity)
- `resolve_driver` para `codex|claude|cursor|antigravity`
- Suite comum 9×4 + smokes
- Docs + **DEC-037**
- Grafo: `DARE/dag-graph-031.mmd`
