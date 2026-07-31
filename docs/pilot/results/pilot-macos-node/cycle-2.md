# Shadow cycle 2 - pilot-macos-node

| Field | Value |
|-------|-------|
| pilot_id | `pilot-macos-node` |
| cycle | 2 |
| shadow_root | `$HOME\AppData\Local\Temp\dare-pilot-pilot-macos-node-84e7d1fd87c8446faa61199cd31ca7ef` (redacted) |
| source_integrity | `pass` |
| verdict | `pass` |

## Commands

| argv | exit | notes |
|------|------|-------|
| `dare --version` | 0 | stdout_len=0; stderr_len=0 |
| `dare --help` | 0 | stdout_len=0; stderr_len=0 |
| `dare info` | 0 | stdout_len=0; stderr_len=0 |

## Source fingerprint sample (>=3)

- `package.json`: `b92f18ceb9a0...`
- `packages/a/package.json`: `30b7f3b1b23f...`
- `packages/b/package.json`: `ae5c25e093a3...`
- `pnpm-workspace.yaml`: `cc6a8bc70d46...`

## Notes

- Copy-only shadow; original source verified unchanged (`MSG_WRITE_ORIGINAL` gate).
- Allowlist argv only; no shell string concatenation.