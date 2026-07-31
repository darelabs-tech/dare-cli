# Packaging (Homebrew + WinGet)

Static package-manager manifests for the native DARE CLI binary (microplanos 053 / 056).

| Path | Manager | Notes |
|------|---------|-------|
| `homebrew/dare.rb` | Homebrew | Formula filled for **v4.0.0** (`aarch64-apple-darwin`) |
| `winget/DareLabs.DareCli.yaml` | WinGet | Singleton manifest filled for **v4.0.0** (Windows x64 zip) |

**Not in v1:** Scoop (BLUEPRINT-053 T-10).

## Release v4.0.0 fill (mp056-004)

| Field | Value |
|-------|-------|
| Tag | `v4.0.0` (`prerelease: false`) |
| Homebrew url | `…/dare-v4.0.0-aarch64-apple-darwin.tar.gz` |
| Homebrew sha256 | `831878c53819e9de7ef31358940207eb75e41e8feaef7dc8b04f2a0083d3578c` |
| WinGet PackageVersion | `4.0.0` |
| WinGet InstallerUrl | `…/dare-v4.0.0-x86_64-pc-windows-msvc.zip` |
| WinGet InstallerSha256 | `3E93F0ED558D32A08561A48F660BF0FF5D250EFEB5846D9D9EAD396E1606FB91` |
| SUMS source | https://github.com/darelabs-tech/dare-cli/releases/download/v4.0.0/SHA256SUMS |

### GAP — Intel macOS

Release **v4.0.0** publishes **aarch64-apple-darwin** only. **`x86_64-apple-darwin` is missing** (documented in `docs/migration/stable-smoke/README.md`). Homebrew formula targets Apple Silicon; Intel Mac users should use a Linux/Windows host or wait for a later asset.

### Tap / winget-pkgs publish

Submitting the Homebrew tap PR and `winget-pkgs` PR is **external** to this repo commit. Manifests here are the source of truth for those submissions.

## Validation checklist

- [x] Both files exist, non-empty, UTF-8 LF
- [x] Homebrew formula defines `class Dare < Formula` and `bin.install "dare"`
- [x] Homebrew `url` / `sha256` filled from Release SUMS (placeholders removed)
- [x] WinGet `PackageIdentifier` is `DareLabs.DareCli`
- [x] WinGet `InstallerUrl` / `InstallerSha256` / version filled (placeholders removed)
- [x] No Scoop directory under `packaging/`

Asset naming (ADR-008): `dare-${TAG}-${TARGET}.tar.gz` (Unix) / `.zip` (Windows).
