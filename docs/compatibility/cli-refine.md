# CLI: `dare refine` (Ciclo 033 / §29)

Avaliação determinística de complexidade e splice de sub-DAG (REPLAN). Complementa [DEC-040](../DECISION-LOG.md). Camada semântica via agente IDE `/dare-refine`.

## Uso

```bash
dare refine <task-id>
dare refine <task-id> --split --format json
dare refine <task-id> --apply
dare refine <task-id> --strict
```

## Flags

| Flag | Efeito |
|------|--------|
| `<task-id>` | Task em `DARE/dare-dag.yaml` (v2.1) |
| `--split` | Inclui proposta de split no report |
| `--apply` | Persiste splice em `DARE/dare-dag.yaml` + `.dare/state.json` |
| `--strict` | Exit **2** se level HIGH\|CRITICAL |
| `--format human\|json` | Default `human` |
| `--json` | Envelope ADR-002 |
| `--no-color` | Sem ANSI |

## Scoring

Sinais: `#ficheiros` (spec §3), `prompt_chars`, `#depends_on`, keywords pesadas, baseline DAG `complexity`.

| Score | Level |
|-------|-------|
| 0–5 | LOW |
| 6–11 | MED |
| 12–17 | HIGH |
| ≥18 | CRITICAL |

`recommendsSplit` = HIGH\|CRITICAL. **CRITICAL** existe só no report — YAML children usam LOW\|MED\|HIGH (validate inalterado).

## spliceSubDag

- Remove a task pai do YAML; insere children `{id}-a`… em cadeia.
- Dependents que apontavam ao pai passam a apontar ao **último** child.
- State: pai `SPLIT`; children com `parentId` + `dependsOn` preservados.
- `MaxDepthError` se profundidade via `parentId` > **2**.
- `CycleError` se o grafo pós-splice ciclar.

## Exit codes

| Code | Quando |
|------|--------|
| 0 | Report / apply OK / no-op LOW\|MED |
| 1 | Falha interna / validate pós-splice |
| 2 | `--strict` HIGH\|CRITICAL **ou** usage clap |
| 3 | Task / DAG NotFound |
| 4 | InvalidInput / MaxDepth / Cycle / id unsafe / Legacy DAG |
| 5 | Io |

## RefineReport schema 1

Campos camelCase: `schemaVersion`, `taskId`, `report{score,level,signals,recommendsSplit}`, `proposal?`, `applied`, `noop`.

## Diff vs TypeScript 3.18.1

| Item | Classificação |
|------|---------------|
| Domínio en-US | **B** (language-policy) |
| CRITICAL só no score (não no YAML) | **B** |
| Sem `--ai` / `--from-agent` neste ciclo | **B** (adiado) |
| Exit 2 strict HIGH/CRITICAL | **A** |
| `parentId` no state | **A** |

## Local verify

```bash
cargo test -p dare-dag --lib subdag
cargo test -p dare-cli --test cli_smoke -- refine_
```
