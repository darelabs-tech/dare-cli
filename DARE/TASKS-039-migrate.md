# Tasks: Migrate (039)

> **Fonte:** `DARE/BLUEPRINT-039-migrate.md` (APPROVED)  
> **Design:** `DARE/DESIGN-039-migrate.md`  
> **DAG:** `DARE/dare-dag-039.yaml`  
> **Specs:** `DARE/EXECUTION-039/`  
> **DEC:** DEC-044  
> **Progresso:** 0/6 (0%)

## Visão Geral

- Total de Tasks: 6
- Ranks paralelos: 4 (rank 0 tem 2 tasks)
- Tempo estimado: ~8–12 h
- Escopo: `dare migrate --to <stack>` — plano + Gherkin skeletons; **sem** rewrite destrutivo

## Tabela de Status

| ID        | Título                                                         | Status     | Depends On           | Complexity |
|-----------|----------------------------------------------------------------|------------|----------------------|------------|
| mp039-001 | Domain: types + allowlist + compare + phases/gaps              | ⏳ PENDING | —                    | MED        |
| mp039-002 | Capability README + matrix row (assets)                        | ⏳ PENDING | —                    | LOW        |
| mp039-003 | render + `run_migrate` check/write + path safety               | ⏳ PENDING | mp039-001            | HIGH       |
| mp039-004 | CLI `dare migrate` + main.rs + AI soft-fail                    | ⏳ PENDING | mp039-003            | MED        |
| mp039-005 | Docs DEC-044 + cli-migrate + matriz finalize                   | ⏳ PENDING | mp039-002, mp039-004 | LOW        |
| mp039-006 | Smokes `migrate_*` + Ralph close                               | ⏳ PENDING | mp039-005            | MED        |

## Progresso

```
░░░░░░░░░░░░░░░░░░░░ 0%
```

## Tarefas por Fase

### Rank 0 (paralelo)
- mp039-001: tipos `MigrateOptions`/`MigrateReport`, allowlist `--to`, family compare, phases, gaps
- mp039-002: `assets/capabilities/dare-migrate/README.md` + row em `capability-matrix.yml`

### Rank 1
- mp039-003: render `MIGRATION.md` / facts / parity skeletons; `run_migrate` check=zero-write

### Rank 2
- mp039-004: `commands/migrate.rs` + `Commands::Migrate` + `--ai` soft-fail

### Rank 3
- mp039-005: `cli-migrate.md` + DEC-044 + matriz 039 Concluído

### Rank 4
- mp039-006: smokes + fmt/clippy/test/audit

## Ready agora

🟢 **mp039-001**, **mp039-002** (`depends_on: []`)

```text
dare execute --parallel
# ou
/dare-execute mp039-001
/dare-execute mp039-002
```

## Próximas Etapas

1. Revisar e aprovar este `TASKS-039-migrate.md` + `dare-dag-039.yaml`
2. Abrir `DARE/dag-graph-039.mmd` para ver o grafo
3. Executar: `dare execute --parallel` (DAG 039) ou `/dare-dag-run-parallel`
