# DARE CLI Stable — v4.0.0

> **Language:** en-US  
> **Tag:** `v4.0.0`  
> **Core / product version:** `4.0.0`  
> **Channel:** GitHub **stable** (non-prerelease)  
> **Microplano:** 056  
> **Date:** 2026-07-31

## This is stable v4.0.0 — not RC

This release is **stable** **`v4.0.0`**.

- It is **not** the release candidate `v4.0.0-rc1`.
- Do **not** treat RC notes, RC smoke-only evidence, or prerelease assets as the production cutover.
- Major **4** follows the npm baseline **`@dewtech/dare-cli@3.18.1`**.

## Install

Follow [`install-rust.md`](install-rust.md). Summary:

| Path | Command / action |
|------|------------------|
| GitHub Release | Download `v4.0.0` assets (+ `SHA256SUMS` / `.sig`) — URLs **TBD** (`publish_ready=false`, `blocked:actions_billing`; see [`publish-stable-checklist.md`](publish-stable-checklist.md)) |
| Homebrew | Formula under `packaging/homebrew/dare.rb` |
| WinGet | Manifest `DareLabs.DareCli` under `packaging/winget/` |
| Self-update | `dare self update --channel stable --yes` (after Release exists) |

**Node / npm is not required** for the recommended path.

Local Windows smoke (not a published Release): [`stable-smoke/`](stable-smoke/).

### Channel defaults (unchanged)

| Surface | Default | Stable opt-in |
|---------|---------|---------------|
| `dare self update` | **`beta`** (unchanged) | `--channel stable` |
| GitHub Release `v4.0.0` | `prerelease: false` | — |

There is **no** silent redirect from `stable` → `beta`.

## Known issues

- **Download URLs:** public asset URLs remain **TBD** — no GitHub Release `v4.0.0` yet (`blocked:actions_billing`; RC run https://github.com/darelabs-tech/dare-cli/actions/runs/30636494439). Do not invent URLs.
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
