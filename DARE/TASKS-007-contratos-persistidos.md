# Tasks: Contratos persistidos (Microplano 007)

> **Fonte:** `DARE/BLUEPRINT-007-contratos-persistidos.md`  
> **DAG:** `DARE/dare-dag-007.yaml`  
> **Specs:** `DARE/EXECUTION-007/`  
> **IDs:** `mp007-*`

## Visão Geral

- **Total de Tasks:** 11
- **Progresso:** 11/11 (100%)

```
████████████████████ 100%
```

## Tabela de Status

| ID        | Título                                              | Status   | Depends On                    | Complexity |
|-----------|-----------------------------------------------------|----------|-------------------------------|------------|
| mp007-001 | Verificar docker-compose.ci.yml (Fase 1)            | ✅ DONE  | —                             | LOW        |
| mp007-002 | Dep yaml_serde 0.10.4 as serde_yaml                 | ✅ DONE  | —                             | LOW        |
| mp007-003 | io.rs + CONTRACTS_SCHEMA_VERSION + cap 2MiB         | ✅ DONE  | mp007-002                     | HIGH       |
| mp007-004 | DareConfig load/save + flatten                      | ✅ DONE  | mp007-003                     | HIGH       |
| mp007-005 | DagV21 + LegacyDag + parse_dag                      | ✅ DONE  | mp007-003                     | HIGH       |
| mp007-006 | RuntimeState + Verification + Telemetry             | ✅ DONE  | mp007-003                     | HIGH       |
| mp007-007 | Graph + Skills + UpdateManifest                     | ✅ DONE  | mp007-003                     | HIGH       |
| mp007-008 | Fixtures + tests/roundtrip.rs                       | ✅ DONE  | mp007-004…007                 | MED        |
| mp007-009 | Docs persisted-contracts + DEC-008                  | ✅ DONE  | mp007-008                     | MED        |
| mp007-010 | Auditoria RS-* + audit/deny                         | ✅ DONE  | mp007-008, 009                | HIGH       |
| mp007-011 | Fechamento microplano 007                           | ✅ DONE  | mp007-001, 009, 010           | LOW        |

## Próximo

Microplano **008** — configuração e migrations.
