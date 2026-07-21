# Process safety (Ciclo 006)

Primitivas de execução segura de processos em `dare-core` (`process`).

## Política

| Tema | Decisão |
|------|---------|
| Shell | Proibido — só argv (`SafeCommand`) |
| Env | Denylist substring case-insensitive: `SECRET`, `TOKEN`, `KEY`, `PASSWORD` |
| Truncate | 4000 Unicode scalars por stream (`stdout_truncated` / `stderr_truncated`) |
| Timeout | `ProcessOutput.exit_code = 124`, `timed_out = true` |
| Cancel | `exit_code = -1`, `cancelled = true` |
| Kill tree | `kill_tree` 0.2.4 + grace 2s → SIGKILL |
| Runtime | `std::process` (sem tokio no core — Classe B vs Documento Mestre) |
| Exe ausente | `CoreError::NotFound` — `executable not found` |

## API (overview)

- `SafeCommand` builder · `ProcessRunner` · `SystemProcessRunner` · `MockProcessRunner`
- `sanitize_env` / `env_key_is_denied` · `truncate_chars`
- Program: bare → PATH; relativo → jail 005; absoluto → só dentro de `ProjectRoot`

## Security (RS-01…RS-09)

| ID | Status |
|----|--------|
| RS-01 Validação program/argv/cwd | ✅ |
| RS-02 Sem log de env values; redact | ✅ |
| RS-03 Env sanitizado | ✅ |
| RS-04 audit/deny | ✅ (gate) |
| RS-05 fixtures sem secrets | ✅ |
| RS-06 sem shell | ✅ |
| RS-07 timeout + kill-tree | ✅ |
| RS-08 truncate | ✅ |
| RS-09 mock sem spawn | ✅ |

## Release notes — Ciclo 006

- Safe process runner (argv, denylist, 124, cancel, mock)
- Pin: `kill_tree 0.2.4`
- Ver DEC-007

## Referências

- `DARE/DESIGN-006-…`, `DARE/BLUEPRINT-006-…`
- DEC-007 em [`DECISION-LOG.md`](../DECISION-LOG.md)
