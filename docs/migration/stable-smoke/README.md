# Stable smoke matrix — v4.0.0

> Microplano **056** · mp056-003 · Product tag **`v4.0.0`** / core **`4.0.0`**  
> **GitHub Release published:** `false` (`blocked:actions_billing`; retry dry_run [`30645155764`](https://github.com/darelabs-tech/dare-cli/actions/runs/30645155764) still billing-blocked)

## Commands (MUST)

| Command | Purpose |
|---------|---------|
| `dare --version` | Binary identity |
| `dare info` | Read-only diagnostics |
| `dare --help` | Critical help surface |

## OS coverage

| OS | Runner | Status | Evidence |
|----|--------|--------|----------|
| Windows | Local operator host (win32, this worktree) | **PASS** | [`windows.md`](windows.md) |
| Linux | Not available on operator host | **GAP** | Owner: Tech Lead DARE CLI — unblock Actions billing then CI/matrix smoke |
| macOS | Not available on operator host | **GAP** | Owner: Tech Lead DARE CLI — unblock Actions billing then CI/matrix smoke |

### Gaps

Only **Windows** was available for interactive local smoke in mp056-003. Linux and macOS smoke against published multi-target assets is expected from:

1. GitHub Actions matrix in `.github/workflows/release.yml` (five targets), and/or
2. Follow-up installs of the stable archives on those OS after a real Release exists.

Do **not** invent Linux/macOS pass logs without a real host or CI artifact.

## Local packaging smoke (Windows only)

Operator ran `scripts/smoke-release-install.ps1` with `DARE_SMOKE_VERSION=v4.0.0` and `CARGO_TARGET_DIR=target-mp056-003`:

- Built host release binary (`dare 4.0.0`)
- Packaged `dare-v4.0.0-x86_64-pc-windows-gnu.zip` (host `rustc` triple; CI Windows target remains `x86_64-pc-windows-msvc`)
- Wrote `SHA256SUMS` + `SHA256SUMS.sig` (local: `signing skipped — local smoke`) + minimal SBOM
- Installed into `dist/smoke/prefix` and asserted `dare --version` matches `^dare `

This is **not** an ADR-008 multi-target Release. `dist/` and `target*` are **not** committed.

## Release gate

```bash
gh release view v4.0.0 --json isPrerelease,tagName,assets
# Expected until billing unblock: release not found
```

See [`../publish-stable-checklist.md`](../publish-stable-checklist.md).
