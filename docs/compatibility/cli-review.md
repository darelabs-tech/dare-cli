# CLI: `dare review` (Ciclo 032 / §28)

Análise **estática** anti-stub / mock / TODO para uma task. Complementa [DEC-034](../DECISION-LOG.md). Camada semântica via agente IDE + `--from-agent`.

## Uso

```bash
dare review <task-id>
dare review <task-id> --strict --format github
dare review <task-id> --files src/a.rs --fail-on warning
dare review <task-id> --from-agent .dare/review-semantic-<id>.json --comment
```

## Flags

| Flag | Efeito |
|------|--------|
| `<task-id>` | Lê `DARE/EXECUTION/<id>.md` e ficheiros da secção 3 |
| `--files PATH...` | Override da lista de ficheiros (jail no project root) |
| `--strict` | Warnings elevam falha |
| `--errors-only` | Emite só findings `error` |
| `--format human\|json\|github` | Default `human`; `github` = annotations Actions |
| `--from-agent PATH` | Merge JSON `{passed, unmetCriteria[], notes?}` |
| `--comment` | Inclui markdown `## DARE review` |
| `--fail-on error\|warning\|never` | Default `error` |
| `--ai` / `--provider` | Soft stub Classe B (warning `enrichment_stub`; sem LLM) |
| `--json` | Envelope ADR-002 em volta do report |
| `--no-color` | Sem ANSI |

## Regras estáticas

| ruleId | severity | Notas |
|--------|----------|-------|
| `todo_marker` | error | `TODO` `FIXME` `XXX` `HACK` |
| `unimplemented_macro` | error | `unimplemented!` / `todo!` |
| `stub_comment` | error | stub / implement later / not implemented em comentário |
| `placeholder_soft` | warning | coming soon / placeholder |
| `mock_outside_test` | error | jest/sinon/vi mocks fora de test paths |
| `empty_ok_stub` | warning | `{ Ok(()) }` etc. |
| `missing_file` | warning | path da spec ausente no disco |
| `file_too_large` | warning | > 1MiB skipped |
| `enrichment_stub` | warning | só com `--ai` |

Mocks são **permitidos** em paths de teste (`*.test.*`, `*.spec.*`, `__tests__/`, `/tests/`, `/spec/`, `*_test.rs`, …).

## Exit codes

| Code | Quando |
|------|--------|
| 0 | Passou o threshold `--fail-on` (e strict) |
| 1 | Falhou review |
| 2 | Usage |
| 3 | Spec `DARE/EXECUTION/<id>.md` ausente |
| 4 | InvalidInput / id unsafe / format / fail-on / from-agent |
| 5 | Io |

## ReviewReport schema 1

Campos camelCase: `schemaVersion`, `taskId`, `ok`, `errorCount`, `warningCount`, `strict`, `failOn`, `enriched`, `filesScanned`, `findings[]`, `unmetCriteria[]`, `commentMarkdown?`, `notes?`.

`finding`: `path`, `line`, `col`, `severity`, `ruleId`, `message`.

Ordenação determinística: path, line, col, ruleId.

## Diff vs TypeScript 3.18.1

| Item | Classificação |
|------|---------------|
| Enrichment `--ai` soft stub (sem LLM nativo) | **B** (DEC-034) |
| Scan line-oriented (sem AST) | **B** |
| Domínio en-US | **B** (language-policy) |
| Exit map 004 + exit 1 review fail | **A** |

## Local verify

```bash
cargo test -p dare-review
cargo test -p dare-cli --test cli_smoke -- review_
```
