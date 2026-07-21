# CLI: `dare validate` (Ciclo 020)

Validação **read-only** de `DARE/dare-dag.yaml` (v2.1 e legado). Complementa [DEC-021](../DECISION-LOG.md) e o Blueprint 020.

## Flags

| Flag | Efeito |
|------|--------|
| `--dag <path>` | Path do YAML (default: `DARE/dare-dag.yaml` sob o project root) |
| `--strict` | Warnings elevam falha (`ok=false`, exit 1) |
| `--json` | Envelope JSON (004); top-level `ok` = `data.ok` |
| `--no-color` | Sem ANSI |

Project root: walk-up via markers (`dare.config.json`, `DARE/`, `package.json`, `Cargo.toml`, …) — mesmo critério de `dare-project::find_project_root`.

## Regras

| code | severity | Semântica |
|------|----------|-----------|
| `invalid_id` | error | id não kebab-case ASCII |
| `duplicate_id` | error | id repetido (v2.1) |
| `missing_dependency` | error | `depends_on` desconhecido |
| `self_dependency` | error | depende de si |
| `cycle` | error | ciclo; `path` canónico (menor id lexico primeiro) |
| `empty_title` | error | title vazio |
| `invalid_complexity` | error | fora de `LOW`\|`MED`\|`HIGH` (case-sensitive) |
| `missing_prompt_or_spec` | error | v2.1: prompt e spec ambos vazios |
| `missing_spec_file` | warning | `spec_file` relativo a `{root}/DARE/` ausente |
| `invalid_limits` | warning | algum limit == 0 |

Legado: **não** aplica regras de prompt/spec.

## Exit codes

| Code | Quando |
|------|--------|
| 0 | `report.ok == true` |
| 1 | falha de regras (`report.ok == false`) — report em stdout (não envelope `error`) |
| 2 | Usage (clap) |
| 3 | ficheiro DAG ausente |
| 4 | project root / path jail / YAML parse (`Config`) |
| 5 | Io |

## ValidationReport schema 1

Campos camelCase: `schemaVersion`, `mode` (`validate`), `ok`, `dagPath`, `format` (`v2.1`\|`legacy`), `taskCount`, `errorCount`, `warningCount`, `strict`, `issues[]`.

`ok = errorCount==0 && (!strict || warningCount==0)`.

## Segurança / contratos

- Zero writes (só leitura + `is_file` em specs)
- Path jail (`ProjectRoot` / `SafeRelativePath`)
- Cap de bytes via `load_dag` / `read_limited` (007)
- Mensagens ≤200 chars; sem corpo de `subtask_prompt`

## Diff vs TypeScript 3.18.1

| Item | Classificação |
|------|---------------|
| Exit 1 + JSON report em falha de regras | Classe B intencional (DEC-021); alinhado a envelope com `data.issues` |
| Complexity case-sensitive | Congelado fixtures nativas |
| `spec_file` base `DARE/` | Congelado Blueprint T-12 |

## Local verify

```bash
docker compose -f docker-compose.ci.yml config
cargo test -p dare-dag
cargo test -p dare-cli --test cli_smoke -- validate
```

Compose CI reutilizado (sem imagem nova) — microplano 003/015.
