# RC smoke matrix — v4.0.0-rc1

> Microplano **055** · mp055-005 · Product tag **`v4.0.0-rc1`** / core **`4.0.0-rc1`**

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
| Linux | Not available on operator host | **GAP** | See below |
| macOS | Not available on operator host | **GAP** | See below |

### Gaps (O-12)

Only **Windows** was available for interactive local smoke in mp055-005. Linux and macOS smoke for the published multi-target assets is expected from:

1. GitHub Actions matrix in `.github/workflows/release.yml` (five targets including linux gnu/aarch64 and darwin x64/arm64), and/or
2. Follow-up pilot/CI runs that install the RC archives on those OS and append logs here.

Do **not** invent Linux/macOS pass logs without a real host or CI artifact.

## Local packaging smoke

Operator also ran `scripts/smoke-release-install.ps1` with `DARE_SMOKE_VERSION=v4.0.0-rc1`:

- Built host release binary
- Packaged `dare-v4.0.0-rc1-<host-triple>.zip`
- Wrote `SHA256SUMS` + `SHA256SUMS.sig` (local: `signing skipped — local smoke`) + minimal SBOM
- Installed into `dist/smoke/prefix` and asserted `dare --version` matches `^dare `

`dist/` and `target*` are **not** committed.
