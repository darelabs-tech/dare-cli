# Configuração e migrations (`dare-config`)

Microplano **008**. Precedência, validação opt-in e migrations de `dare.config.json`.

## Precedência

**CLI > env `DARE_*` > ficheiro > default**

Allowlist env:

| Variável | Efeito |
|----------|--------|
| `DARE_IDE` | override `ide` |
| `DARE_GUARD_ENABLED` | `true`/`false`/`1`/`0` → `guard.enabled` |
| `DARE_GRAPH_ENABLED` | idem `graph` |
| `DARE_AGENT_ENABLED` | idem `agent` |
| `DARE_HOOKS_ENABLED` | idem `hooks` |
| `DARE_PROJECT_ENABLED` | idem `project` |

## API

- `default_config`, `merge_layers`, `validate`
- `load_effective(root, rel, env, cli)`
- `dry_run_migrate` — **zero writes**
- `apply_migrate` — `backup` + `save_dare_config` se houver steps

## Schema version

Não escrito por default. Só com `MigrateOptions { write_schema_version: true, .. }` → chave `schemaVersion` no `extra` (flatten).

## Segurança

- I/O sob `ProjectRoot` (005)
- Cap 2 MiB via contracts (007)
- Erros `CoreError::Config` (exit 4) com JSON Pointer
- DEC-009

## Ver também

- [`persisted-contracts.md`](persisted-contracts.md)
- [`disk-and-json-policy.md`](disk-and-json-policy.md)
