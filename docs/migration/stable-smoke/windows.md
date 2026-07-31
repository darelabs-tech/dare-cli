# Stable smoke — Windows

- **Stable tag / core:** `v4.0.0` / `4.0.0`
- **Host OS:** Windows (win32)
- **Date:** 2026-07-31
- **Binary source:** GitHub Release asset `dare-v4.0.0-x86_64-pc-windows-msvc.zip`
- **Download URL:** https://github.com/darelabs-tech/dare-cli/releases/download/v4.0.0/dare-v4.0.0-x86_64-pc-windows-msvc.zip
- **CI run (artifact origin):** https://github.com/darelabs-tech/dare-cli/actions/runs/30662777245
- **Install prefix:** ephemeral `dist-smoke/` (not committed)
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
dare 4.0.0
```

### `dare info`

```
DARE info (schema 1)
  version:    4.0.0
  platform:   windows-x86_64 (windows)
  project:    $WORKTREE
  assets:     FAIL (asset hash mismatch: templates/DESIGN-template.md)
  config:     yes
  DARE/:      no
  .dare/state: no
  graph:      (absent)
  backend/ide: cursor
  tasks:      done=0 pending=0 (no TASKS.md)
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

## Packaging meta

- Public Release: https://github.com/darelabs-tech/dare-cli/releases/tag/v4.0.0 (`isPrerelease=false`)
- `SHA256SUMS` / `SHA256SUMS.sig` published on the Release
- Note: archive inner directory may still carry the CI branch staging name (`dare-mp056-003-publish-…`); binary identity is `dare 4.0.0`
