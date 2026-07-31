# RC smoke — Windows

- **RC tag / core:** `v4.0.0-rc1` / `4.0.0-rc1`
- **Host OS:** Windows (win32)
- **Date:** 2026-07-31
- **Binary source:** `scripts/smoke-release-install.ps1` (`DARE_SMOKE_VERSION=v4.0.0-rc1`)
- **Artifact name:** `dare-v4.0.0-rc1-x86_64-pc-windows-gnu.zip` (host `rustc` triple; CI Windows target remains `x86_64-pc-windows-msvc`)
- **Install prefix:** `dist/smoke/prefix` (ephemeral; not committed)
- **Paths:** redacted worktree root as `$WORKTREE`

## Results

| Command | Exit | Verdict |
|---------|------|---------|
| `dare --version` | 0 | PASS |
| `dare info` | 0 | PASS |
| `dare --help` | 0 | PASS |

## Captures

### `dare --version`

```
dare 0.1.0-alpha.0
```

> Embedded clap / workspace string may lag the RC tag (ADR-008). Product identity for this RC is **`4.0.0-rc1`** / tag **`v4.0.0-rc1`**.

### `dare info`

```
DARE info (schema 1)
  version:    0.1.0-alpha.0
  platform:   windows-x86_64 (windows)
  project:    $WORKTREE
  assets:     FAIL (asset hash mismatch: templates/DESIGN-template.md)
  config:     yes
  DARE/:      yes
  .dare/state: no
  graph:      (absent)
  backend/ide: cursor
  tasks:      (see local DARE/TASKS.md)
  mode:       read-only (zero mutations)
```

`assets: FAIL` is expected on a dirty/dev worktree and is recorded as a known issue in RELEASE-NOTES (not an install smoke failure).

### `dare --help`

```
DARE Framework CLI (native Rust rewrite)

Usage: dare.exe [OPTIONS] [COMMAND]

Commands:
  welcome       Show banner (TTY) and DARE quick-start guide
  info          Read-only installation / project diagnostics
  ...
  self          Manage the dare CLI binary itself (self-update / rollback / uninstall). Distinct from `dare update`, which refreshes project assets under a ProjectRoot

Options:
      --json      Emit JSON envelopes on stdout (ADR-002)
  -h, --help      Print help
  -V, --version   Print version
```

(Full help listing truncated in this log; exit code 0 confirmed.)

## Packaging meta (local)

- `SHA256SUMS` produced for the local zip
- `SHA256SUMS.sig` = `signing skipped — local smoke` (not a CI cosign blob)
