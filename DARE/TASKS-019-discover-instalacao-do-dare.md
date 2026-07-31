# Tasks: Discover — instalação do DARE (019)

> **Fonte:** `DARE/BLUEPRINT-019-discover-instalacao-do-dare.md`  
> **Design:** `DARE/DESIGN-019-discover-instalacao-do-dare.md`  
> **DAG:** `DARE/dare-dag-019.yaml`  
> **Specs:** `DARE/EXECUTION-019/`  
> **Progresso:** 8/8 (100%)  
> **Nota:** IDs `mp019-*`; **DONE** — `dare discover` install + rollback; próximo 020-validate

## Visão Geral

- Total de Tasks: 8
- Ranks paralelos: 6 (rank 0: 2 tasks)
- Tempo estimado: ~8–14 h

## Tabela de Status

| ID        | Título                                                              | Status     | Depends On                      | Complexity |
|-----------|---------------------------------------------------------------------|------------|---------------------------------|------------|
| mp019-001 | Verificar docker-compose.ci.yml                                     | ✅ DONE    | —                               | LOW        |
| mp019-002 | Tipos InstallPlan/Report + plan_install + select_ide + conflicts    | ✅ DONE    | —                               | HIGH       |
| mp019-003 | apply FS steps + journal/rollback + dry_run                         | ✅ DONE    | mp019-002                       | HIGH       |
| mp019-004 | Harnesses + ensure_capability + validate + install()                | ✅ DONE    | mp019-003                       | HIGH       |
| mp019-005 | CLI discover install + flags + smokes                               | ✅ DONE    | mp019-004                       | MED        |
| mp019-006 | Docs cli-discover-install.md + DEC-020                              | ✅ DONE    | mp019-002                       | LOW        |
| mp019-007 | Auditoria Ralph (test/clippy/audit/deny)                            | ✅ DONE    | mp019-001, mp019-005, mp019-006 | MED        |
| mp019-008 | Fechamento microplano 019                                           | ✅ DONE    | mp019-007                       | LOW        |

## Próximas Etapas

1. ~~Revisar e aprovar este TASKS + DAG~~
2. ~~Executar `/dare-dag-run-parallel`~~
3. **Próximo microplano:** `020-validate`
