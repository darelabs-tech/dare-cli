# Tasks: Filesystem seguro e path safety (Microplano 005)

> **Fonte:** `DARE/BLUEPRINT-005-filesystem-seguro-e-path-safety.md`  
> **DAG:** `DARE/dare-dag-005.yaml`  
> **Specs:** `DARE/EXECUTION-005/`  
> **IDs:** `mp005-*`

## Visão Geral

- **Total de Tasks:** 9
- **Progresso:** 9/9 (100%)

```
████████████████████ 100%
```

## Tabela de Status

| ID        | Título                                              | Status   | Depends On              | Complexity |
|-----------|-----------------------------------------------------|----------|-------------------------|------------|
| mp005-001 | Verificar docker-compose.ci.yml (Fase 1)            | ✅ DONE  | —                       | LOW        |
| mp005-002 | Deps camino/tempfile/fs4/sha2                       | ✅ DONE  | —                       | LOW        |
| mp005-003 | path.rs ProjectRoot + SafeRelativePath              | ✅ DONE  | mp005-002               | HIGH       |
| mp005-004 | Testes symlink/junction (T-01)                      | ✅ DONE  | mp005-003               | MED        |
| mp005-005 | fs atomic_write + backup/restore                    | ✅ DONE  | mp005-003               | HIGH       |
| mp005-006 | fs FileLock contenção                               | ✅ DONE  | mp005-003               | MED        |
| mp005-007 | Docs path-safety + DEC-006                          | ✅ DONE  | mp005-004, 005, 006     | MED        |
| mp005-008 | Auditoria RS-* + audit/deny                         | ✅ DONE  | mp005-005, 006, 007     | HIGH       |
| mp005-009 | Fechamento microplano 005                           | ✅ DONE  | mp005-001, 007, 008     | LOW        |

## Próximo

Microplano **006** — execução segura de processos.
