# CLI output, errors, and exit codes (Ciclo 004)

Contrato de saída do DARE CLI nativo após o microplano 004. Complementa [ADR-002](../adr/ADR-002-contrato-saida-json.md) e a [política de idioma](language-policy.md).

## Flags globais

| Flag | Efeito |
|------|--------|
| `--json` | Envelopes JSON em **stdout** (sucesso e erro) |
| `--no-color` | Desliga ANSI |
| `NO_COLOR` (env, qualquer valor) | Desliga ANSI |
| `RUST_LOG` | Filtro tracing (default `warn`); logs em **stderr** |

`--help` / `--version` **nunca** usam envelope JSON.

## Exit codes (core v1)

| Exit | `ErrorKind` | Semântica |
|------|-------------|-----------|
| 0 | — | sucesso |
| 1 | `Internal` | erro interno |
| 2 | `Usage` | args/flags inválidos |
| 3 | `NotFound` | recurso ausente |
| 4 | `InvalidInput`, `Config` | validação / config |
| 5 | `Io` | falha de I/O |
| 6 | `GuardFail` | guard FAIL / preflight agent (microplano 034) |
| ≥7 | — | **reservado** |

## Streams

| Modo | Sucesso | Erro |
|------|---------|------|
| human | stdout | stderr (`error: …`) |
| json | stdout | **stdout** + exit ≠ 0 |

## Envelope JSON (canónico)

Keys de objetos em ordem lexicográfica (ADR-002). Sem bytes ANSI. Sem `schema_version` neste ciclo.

Sucesso:

```json
{"correlation_id":"…","data":{},"ok":true}
```

Erro:

```json
{"correlation_id":"…","error":{"kind":"Usage","message":"…"},"ok":false}
```

`correlation_id` é volátil (allowlist ADR-002).

## Redaction

Mensagens de erro passam por `dare_core::redact` (Bearer, password, api_key, token, Authorization, AKIA…).

## Security (RS-01…RS-08)

| ID | Controlo | Status |
|----|----------|--------|
| RS-01 | Mensagens de erro normalizadas / sem controlo cru indevido | ✅ |
| RS-02 | Redaction de secrets em erros | ✅ |
| RS-03 | Sem stack traces em human default | ✅ |
| RS-04 | `cargo audit` + `cargo deny` | ✅ (gate CI) |
| RS-05 | Sem secrets em código; patterns versionados | ✅ |
| RS-06 | JSON de erro sem campos de secret | ✅ |
| RS-07 | `correlation_id` UUID v4 | ✅ |
| RS-08 | Path safety avançada adiada ao 005 | ✅ |

## Release notes — Ciclo 004

- `ErrorKind` + exit codes 1–5; `InvalidArgument` → `InvalidInput`
- `OutputRenderer` human/json; flags `--json` / `--no-color`
- Tracing com `RUST_LOG` (default warn) + redaction
- Ver DEC-005

## Referências

- Design/Blueprint: `DARE/DESIGN-004-…`, `DARE/BLUEPRINT-004-…`
- DEC-005 em [`DECISION-LOG.md`](../DECISION-LOG.md)
