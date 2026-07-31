# DARE CLI Release Candidate — v4.0.0-rc1

> **Tag:** `v4.0.0-rc1`  
> **Core / product version:** `4.0.0-rc1`  
> **Channel:** GitHub **prerelease** (not stable)  
> **Microplano:** 055  
> **Date:** 2026-07-31

## Not a stable cutover

This release candidate is **not** the stable cutover.

- **Stable cutover** (npm legacy retirement, official stable tag, default-channel policy for end users) is **microplano 056** — future work.
- Operators and pilots may install and validate `v4.0.0-rc1`, but **must not** treat it as production-stable or as the end of the TypeScript `@dewtech/dare-cli` line.
- Major **4** follows the npm baseline **`@dewtech/dare-cli@3.18.1`**.

## Install

Install **one** of the following ways:

### A — Download GitHub Release assets (ADR-008)

1. Open the prerelease: `https://github.com/darelabs-tech/dare-cli/releases/tag/v4.0.0-rc1`
2. Download the archive for your target (`dare-v4.0.0-rc1-<TARGET>.tar.gz` or `.zip`), plus `SHA256SUMS` and `SHA256SUMS.sig`.
3. Verify checksums; use `installers/install.sh` or `installers/install.ps1` with `DARE_VERSION=v4.0.0-rc1` (or `DARE_LOCAL_ARCHIVE`).

### B — Self-update with an explicit version

```bash
dare self update --version 4.0.0-rc1 --yes
# equivalent tag form:
dare self update --version v4.0.0-rc1 --yes
```

**Default `dare self` channel remains `beta` (unchanged).**  
Do **not** expect `dare self update` (no flags) or `--channel stable` to select this RC. Install the RC only via **`--version`** (or by downloading assets).

> **Note (ADR-008):** Git tag names release archives. The embedded clap / `[workspace.package]` version string may still display `0.1.0-alpha.0` until an explicit workspace bump; the **product RC identity** is always the tag / core version above.

## Freezes (in force from this RC)

| Doc | Policy |
|-----|--------|
| [`typescript-freeze.md`](typescript-freeze.md) | `@dewtech/dare-cli` TypeScript line: **security fixes only** until 056 supersedes |
| [`contract-freeze.md`](contract-freeze.md) | Classe A contracts frozen unless ADR Accepted + matrix + DECISION-LOG path |

## Known issues

- **Pilot seeds (INC-001, P2 mitigated):** some 054 fixtures have fewer than three files; shadow fingerprint gate needs ≥3 — use `tests/fixtures/monorepo` (or equivalent) as shadow source per [`../pilot/shadow-playbook.md`](../pilot/shadow-playbook.md).
- **Cross-OS smoke (O-12):** local operator smoke for this RC may cover fewer than three OS hosts; gaps are documented under [`../pilot/results/rc-smoke/`](../pilot/results/rc-smoke/). Full five-target binaries come from `.github/workflows/release.yml`.
- **Cosign / `dare self update`:** self-update is **fail-closed** on missing cosign or a `SHA256SUMS.sig` that starts with `signing skipped` (exit **6**). Prefer installers when signature soft-fail applies; see [`../compatibility/cli-self-update.md`](../compatibility/cli-self-update.md).
- **Asset hash in `dare info`:** worktree / dirty trees may report `assets: FAIL (asset hash mismatch …)` — expected for non-release checkouts; not an RC install blocker.

## Smoke

Post-install smoke (per OS where available):

```bash
dare --version
dare info
dare --help
```

Evidence: [`../pilot/results/rc-smoke/`](../pilot/results/rc-smoke/).

## Rollback pointer

If this RC misbehaves:

1. **`dare self rollback --yes`** — restores `~/.dare/self/backup/dare[.exe]` when a prior self-update backup exists.
2. **Reinstall previous native artifact** — there was **no prior GitHub Release tag** on this repository at RC publish time; fall back to a known-good local/CI alpha build or rebuild from a pre-RC commit.
3. **TypeScript legacy** — continue using npm `@dewtech/dare-cli@3.18.1` until microplano **056** cutover.

Formal drill worksheet: [`rollback-drill.md`](rollback-drill.md) (filled in mp055-006).

## Related

- ADR-008 — native release assets, checksums, signature posture  
- [`../compatibility/release-alpha.md`](../compatibility/release-alpha.md) — release pipeline  
- [`../pilot/incidents.md`](../pilot/incidents.md) — pilot findings  
