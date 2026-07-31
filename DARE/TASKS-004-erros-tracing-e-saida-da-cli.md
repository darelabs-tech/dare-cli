# Tasks: Erros, tracing e saída da CLI (Microplano 004)

> **Fonte:** `DARE/BLUEPRINT-004-erros-tracing-e-saida-da-cli.md`  
> **DAG:** `DARE/dare-dag-004.yaml`  
> **Specs:** `DARE/EXECUTION-004/`  
> **IDs:** `mp004-*`  
> **Não substitui:** TASKS 001–003

## Visão Geral

- **Total de Tasks:** 8
- **Ranks paralelos:** 5 (Kahn)
- **Progresso:** 8/8 (100%)

```
████████████████████ 100%
```

## Tabela de Status

| ID        | Título                                              | Status   | Depends On                    | Complexity |
|-----------|-----------------------------------------------------|----------|-------------------------------|------------|
| mp004-001 | Verificar docker-compose.ci.yml (Fase 1)            | ✅ DONE  | —                             | LOW        |
| mp004-002 | ErrorKind + CoreError + exit_code                   | ✅ DONE  | —                             | MED        |
| mp004-003 | redact + fixtures                                   | ✅ DONE  | —                             | MED        |
| mp004-004 | ExecutionContext + telemetry + uuid                 | ✅ DONE  | mp004-002                     | MED        |
| mp004-005 | OutputRenderer + JSON canónico + wire main          | ✅ DONE  | mp004-002, 003, 004           | HIGH       |
| mp004-006 | Docs cli-output-and-errors + DEC-005                | ✅ DONE  | mp004-005                     | MED        |
| mp004-007 | Auditoria RS-* + audit/deny                         | ✅ DONE  | mp004-003, 005, 006           | HIGH       |
| mp004-008 | Fechamento microplano 004                           | ✅ DONE  | mp004-001, 006, 007           | LOW        |

## Próximo

Microplano **005** — filesystem seguro e path safety.
