# Tasks: Governança, baseline e ADRs prioritárias (Microplano 001)

## Visão Geral

- **Total de Tasks:** 13
- **Ranks paralelos:** 5 (0–4)
- **Tempo estimado:** ~10–14 h
- **Fonte:** `DARE/BLUEPRINT.md` v1.0 (DRAFT → execução)
- **Progresso:** 13/13 (100%)

```
████████████████████ 100%
```

## Tabela de Status

| ID       | Título                                              | Status     | Depends On                          | Complexity |
|----------|-----------------------------------------------------|------------|-------------------------------------|------------|
| task-001 | Scaffold árvore docs/ (placeholders obrigatórios)   | ✅ DONE    | —                                   | LOW        |
| task-002 | Scaffold scripts/governance + verify-structure      | ✅ DONE    | —                                   | MED        |
| task-003 | Containerização governance (Docker + Compose)       | ✅ DONE    | task-001, task-002                  | MED        |
| task-004 | Baseline 3.18.1 + verify-baseline (hash real)       | ✅ DONE    | task-002                            | HIGH       |
| task-005 | Pacote compatibility + DECISION-LOG (DEC-001)       | ✅ DONE    | task-001                            | MED        |
| task-006 | ADR-001 Compatibilidade de bugs legados             | ✅ DONE    | task-001                            | MED        |
| task-007 | ADR-002 Contrato de saída JSON                      | ✅ DONE    | task-001                            | MED        |
| task-008 | ADR-004 REST compatível e MCP real                  | ✅ DONE    | task-001                            | MED        |
| task-009 | ADR-006 Compatibilidade e migração Graph DB         | ✅ DONE    | task-001                            | MED        |
| task-010 | ADR-007 Formato canônico de capabilities            | ✅ DONE    | task-001                            | MED        |
| task-011 | verify-adr-frontmatter + índices README             | ✅ DONE    | task-002, 006, 007, 008, 009, 010   | MED        |
| task-012 | Auditoria segurança npm + NO_SECRETS                | ✅ DONE    | task-004, task-005, task-011        | HIGH       |
| task-013 | CI GHA + fixtures-inventory + fechamento ciclo 0    | ✅ DONE    | task-003, task-012                  | MED        |

## Tarefas por Fase (Blueprint)

### Fase 1 — Containerização e setup
- task-001, task-002 (rank 0)
- task-003 (rank 1)

### Fase 2 — Baseline TypeScript
- task-004

### Fase 3 — Compatibility pack + decision log
- task-005

### Fase 4 — ADRs prioritárias
- task-006 … task-010 (paralelas no rank 1)
- task-011 (validação frontmatter + índices)

### Fase 5–7 — Fixtures, segurança, CI
- task-012 (auditoria)
- task-013 (GHA + inventory + fechamento)

## Fechamento

Microplano 001 concluído. Gates Cargo deferidos (DEC-001). Próximo: microplano 002 após aprovação humana.
