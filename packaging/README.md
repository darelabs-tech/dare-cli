# Packaging (Homebrew + WinGet)

Static package-manager templates for the native DARE CLI binary (microplano 053).

| Path | Manager | Notes |
|------|---------|-------|
| `homebrew/dare.rb` | Homebrew | Formula; fill `url` / `sha256` from ADR-008 Release |
| `winget/DareLabs.DareCli.yaml` | WinGet | Minimal singleton manifest; fill `InstallerUrl` / `InstallerSha256` |

**Not in v1:** Scoop (BLUEPRINT-053 T-10).

## Validation checklist

- [ ] Both files exist, non-empty, UTF-8 LF
- [ ] Homebrew formula defines `class Dare < Formula` and `bin.install "dare"`
- [ ] Homebrew `url` / `sha256` are documented PLACEHOLDERS (`REPLACE_ME_*`)
- [ ] WinGet `PackageIdentifier` is `DareLabs.DareCli`
- [ ] WinGet `InstallerUrl` / `InstallerSha256` are documented PLACEHOLDERS
- [ ] No Scoop directory under `packaging/`

Asset naming (ADR-008): `dare-${TAG}-${TARGET}.tar.gz` (Unix) / `.zip` (Windows).