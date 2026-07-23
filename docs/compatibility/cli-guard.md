# CLI guard (`dare guard`)

> **DEC-035** · Microplano 034 · Source: `crates/dare-guard` + `crates/dare-cli/src/commands/guard.rs`

## Purpose

Security gate with three layers: **Unicode** → **injection scan** → **provenance/signing**. Verdict `PASS|WARN|FAIL`. FAIL exits **6**. Used as preflight for `dare execute --agent`.

## Commands

```bash
dare guard [target]
dare guard --staged
dare guard --all
dare guard --sign <target>
dare guard --unicode strip|block --strict --fail-on fail|warn
```

Global: `--json` / `--no-color`.

| Flag | Default | Effect |
|------|---------|--------|
| `target` | `DARE/` + `dare.config.json` | File or directory under project root |
| `--staged` | — | Git staged files only |
| `--all` | — | Walk project text files (skips `.git`/`target`/`node_modules`/`.dare`) |
| `--sign` | — | Write `<file>.minisig` (Ed25519); needs `DARE_GUARD_PRIVATE_KEY` |
| `--unicode` | `block` | `strip` sanitizes (WARN); `block` FAIL on ZW/bidi/VS/tags |
| `--strict` | off | WARN → exit 6 |
| `--fail-on` | `fail` | `warn` also fails |
| `--format` | `text` | `text`\|`json` (envelope still via `--json`) |

## Env

| Var | Effect |
|-----|--------|
| `DARE_GUARD_SCAN_RULES_PATH` | Override rules JSON |
| `DARE_GUARD_PRIVATE_KEY` | Hex 32-byte seed for `--sign` |
| `DARE_GUARD_PUBLIC_KEY` | Hex verifying key |
| `DARE_GUARD_SIGNING_ENABLED` | `1`/`true` to require signatures on control paths |

## Rules

Built-in asset: `assets/rules/scan-rules.json` (also embedded):

- `instr-override` (fail)
- `shell-exec` (fail)
- `exfiltration` (fail)
- `hidden-directive` (warn)

Evidence snippets are redacted via `dare_core::redact`.

## Signing (Classe B)

Ed25519 (`ed25519-dalek`) with header `untrusted comment: dare-guard ed25519` in `.minisig`. Not full minisign wire format.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | PASS (or WARN without strict) |
| 2 | Usage |
| 3 | Target not found |
| 4 | Invalid input / config |
| 5 | Io |
| **6** | Guard FAIL |

## Agent preflight

`dare execute --agent` calls `dare_guard::run_preflight` before the loop. FAIL → exit **6**, agent does not start.

## Local verify

```bash
cargo test -p dare-guard
cargo test -p dare-cli --test cli_smoke -- guard
```
