# DARE CLI Stable — v4.0.0

> **Language:** en-US  
> **Tag:** `v4.0.0`  
> **Core / product version:** `4.0.0`  
> **Channel:** GitHub **stable** (non-prerelease)  
> **Microplano:** 056  
> **Date:** 2026-07-31  
> **Release:** https://github.com/darelabs-tech/dare-cli/releases/tag/v4.0.0

## This is stable v4.0.0 — not RC

This release is **stable** **`v4.0.0`**.

- It is **not** the release candidate `v4.0.0-rc1`.
- Do **not** treat RC notes, RC smoke-only evidence, or prerelease assets as the production cutover.
- Major **4** follows the npm baseline **`@dewtech/dare-cli@3.18.1`**.

## Install

Follow [`install-rust.md`](install-rust.md). Summary:

| Path | Command / action |
|------|------------------|
| GitHub Release | https://github.com/darelabs-tech/dare-cli/releases/tag/v4.0.0 (+ `SHA256SUMS` / `.sig`) |
| Linux x86_64 | https://github.com/darelabs-tech/dare-cli/releases/download/v4.0.0/dare-v4.0.0-x86_64-unknown-linux-gnu.tar.gz |
| Linux aarch64 | https://github.com/darelabs-tech/dare-cli/releases/download/v4.0.0/dare-v4.0.0-aarch64-unknown-linux-gnu.tar.gz |
| macOS aarch64 | https://github.com/darelabs-tech/dare-cli/releases/download/v4.0.0/dare-v4.0.0-aarch64-apple-darwin.tar.gz |
| Windows x86_64 | https://github.com/darelabs-tech/dare-cli/releases/download/v4.0.0/dare-v4.0.0-x86_64-pc-windows-msvc.zip |
| Checksums | https://github.com/darelabs-tech/dare-cli/releases/download/v4.0.0/SHA256SUMS |
| Signature | https://github.com/darelabs-tech/dare-cli/releases/download/v4.0.0/SHA256SUMS.sig |
| Installer (Unix) | https://github.com/darelabs-tech/dare-cli/releases/download/v4.0.0/install.sh |
| Installer (Windows) | https://github.com/darelabs-tech/dare-cli/releases/download/v4.0.0/install.ps1 |
| Homebrew | Formula under `packaging/homebrew/dare.rb` |
| WinGet | Manifest `DareLabs.DareCli` under `packaging/winget/` |
| Self-update | `dare self update --channel stable --yes` (after Release exists) |

**Node / npm is not required** for the recommended path.

Smoke evidence: [`stable-smoke/`](stable-smoke/).

### Channel defaults (unchanged)

| Surface | Default | Stable opt-in |
|---------|---------|---------------|
| `dare self update` | **`beta`** (unchanged) | `--channel stable` |
| GitHub Release `v4.0.0` | `prerelease: false` | — |

There is **no** silent redirect from `stable` → `beta`.

## Known issues

- **`x86_64-apple-darwin` GAP:** Intel macOS archive was not published in this cut — macos-13 runner stayed queued. Owner: Tech Lead DARE CLI. Use aarch64 macOS asset on Apple Silicon, or build from source until the gap is filled.
- **Cosign / `dare self update`:** fail-closed on missing cosign or a `SHA256SUMS.sig` that starts with `signing skipped` (exit **6**). Prefer installers when signature soft-fail applies; see [`../compatibility/cli-self-update.md`](../compatibility/cli-self-update.md).
- **Asset hash in `dare info`:** dirty / worktree checkouts may report `assets: FAIL` — expected for non-release trees; not a stable install blocker once Release assets match.
- **Pilot leftovers (from RC):** fixture density and synthetic multi-OS notes remain documented under [`../pilot/incidents.md`](../pilot/incidents.md) — not blockers for the stable product identity.

## Legacy npm pointer

The TypeScript package **`@dewtech/dare-cli`** is **legacy**:

- Policy: [`npm-legacy-policy.md`](npm-legacy-policy.md) (`status: legacy`, `last_supported_npm: 3.18.1`)
- Support window: [`../support/legacy-support-window.md`](../support/legacy-support-window.md) (`scope: security-only`)

## Rollback

1. `dare self rollback --yes` when a self-update backup exists.
2. Reinstall a prior known-good native artifact.
3. Post-stable worksheet: [`rollback-after-stable.md`](rollback-after-stable.md) (when filled).
4. RC drill reference: [`../release-candidate/rollback-drill.md`](../release-candidate/rollback-drill.md).

## Related

- [`install-rust.md`](install-rust.md)  
- [`../support/incident-response.md`](../support/incident-response.md)  
- [`../compatibility/README.md`](../compatibility/README.md)  
- DEC-057 (stable cutover decision — DECISION-LOG when recorded)
