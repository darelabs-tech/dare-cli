# Install the native DARE CLI (Rust) — v4.0.0

> **Language:** en-US  
> **Product version:** `4.0.0` · **Git tag:** `v4.0.0`  
> **Microplano:** 056  
> **Audience:** operators and end users migrating off npm `@dewtech/dare-cli`

## Overview

The official DARE CLI is the **native Rust** binary tagged **`v4.0.0`**.

- Node.js / npm is **not required** to install or run the recommended CLI.
- Package managers (Homebrew, WinGet) and `dare self update --channel stable` install the same native binary.
- The TypeScript npm line `@dewtech/dare-cli` is **legacy** — see [`npm-legacy-policy.md`](npm-legacy-policy.md).

## Download v4.0.0 (GitHub Release)

1. Open the GitHub Release for tag **`v4.0.0`** (must be a **non-prerelease** release — not `v4.0.0-rc1`).
2. Download the archive for your target, plus `SHA256SUMS` and `SHA256SUMS.sig` (ADR-008).
3. Verify checksums; install with `installers/install.sh` or `installers/install.ps1` using `DARE_VERSION=v4.0.0` (or `DARE_LOCAL_ARCHIVE`).

Asset naming:

```text
dare-v4.0.0-<TARGET>.tar.gz   # Unix
dare-v4.0.0-<TARGET>.zip      # Windows
```

> Download URLs may still be **TBD** until the stable Release assets are published (mp056-003). Do not invent URLs.

## Homebrew

Formula template: [`packaging/homebrew/dare.rb`](../../packaging/homebrew/dare.rb).

After the formula is published / filled from the `v4.0.0` Release:

```bash
brew install dare
# or, when using a tap / local formula path documented by maintainers:
# brew install --formula ./packaging/homebrew/dare.rb
```

Fill `url` / `sha256` from the published Release (see [`packaging/README.md`](../../packaging/README.md)).

## WinGet

Manifest template: [`packaging/winget/DareLabs.DareCli.yaml`](../../packaging/winget/DareLabs.DareCli.yaml).

```powershell
winget install DareLabs.DareCli
```

`InstallerUrl` / `InstallerSha256` must match the published `v4.0.0` Windows asset.

## Self-update (stable channel)

If you already have a native `dare` binary:

```bash
dare self update --channel stable --yes
```

Notes:

- Default `dare self` channel remains **`beta`** (unchanged). Opt into stable with `--channel stable`.
- There is **no** silent redirect from `stable` → `beta`.
- Self-update is fail-closed on missing cosign / bad signature (exit **6**). Details: [`../compatibility/cli-self-update.md`](../compatibility/cli-self-update.md).

## Verify

```bash
dare --version
dare info
dare --help
```

Expect product identity **`4.0.0` / `v4.0.0`** (stable), not `v4.0.0-rc1`.

## Rollback pointer

If stable misbehaves after install or self-update:

1. `dare self rollback --yes` — restores `~/.dare/self/backup/dare[.exe]` when a backup exists.
2. Reinstall a known-good prior native artifact (previous Release tag or local archive).
3. Formal post-cutover worksheet (when filled): [`rollback-after-stable.md`](rollback-after-stable.md) (mp056 follow-up).
4. RC-era drill reference: [`../release-candidate/rollback-drill.md`](../release-candidate/rollback-drill.md).

## Node / npm not required

Installing and running the recommended DARE CLI **does not require Node.js or npm**.

- Prefer this document’s Rust install paths.
- The npm package `@dewtech/dare-cli` remains available only as a **legacy** line under a security-only support window — see [`npm-legacy-policy.md`](npm-legacy-policy.md) and [`../support/legacy-support-window.md`](../support/legacy-support-window.md).

## Related

- [`RELEASE-NOTES-stable.md`](RELEASE-NOTES-stable.md) — stable release notes
- [`../support/incident-response.md`](../support/incident-response.md) — severity / SLA after cutover
- [`../compatibility/README.md`](../compatibility/README.md) — compatibility package index
