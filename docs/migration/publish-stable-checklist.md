# Publish checklist — stable `v4.0.0`

> **Status:** `ready`  
> **publish_ready:** `true`  
> **Operator:** Wanderson Leandro de Oliveira (`wleandrooliveira`)  
> **Date:** 2026-07-31  
> **STABLE_TAG:** `v4.0.0`  
> **Microplano:** 056 · mp056-003  
> **Release:** https://github.com/darelabs-tech/dare-cli/releases/tag/v4.0.0

## Verdict

GitHub Release **`v4.0.0`** is published with **`isPrerelease: false`**, binary archives, `SHA256SUMS`, `SHA256SUMS.sig`, `SBOM.spdx.json`, and installers.

**mp056-003 publish (2026-07-31) — SUCCESS:**

| Check | Result |
|-------|--------|
| `gh release view v4.0.0 --json isPrerelease,url` | `isPrerelease=false`, URL below |
| Assets | 4/5 ADR-008 targets + meta (see GAP) |
| Build source | Actions run [`30662777245`](https://github.com/darelabs-tech/dare-cli/actions/runs/30662777245) (`workflow_dispatch` on `mp056-003-publish`; linux aarch64 via `ubuntu-24.04-arm`) |
| Publish path | Manual `gh release create` from CI artifacts (dispatch on branch does not hit tag `publish` job) |
| Windows smoke vs Release zip | **PASS** — see [`stable-smoke/`](stable-smoke/) |

```json
{
  "isPrerelease": false,
  "tagName": "v4.0.0",
  "url": "https://github.com/darelabs-tech/dare-cli/releases/tag/v4.0.0",
  "assets": [
    "dare-v4.0.0-aarch64-apple-darwin.tar.gz",
    "dare-v4.0.0-aarch64-unknown-linux-gnu.tar.gz",
    "dare-v4.0.0-x86_64-pc-windows-msvc.zip",
    "dare-v4.0.0-x86_64-unknown-linux-gnu.tar.gz",
    "SHA256SUMS",
    "SHA256SUMS.sig",
    "SBOM.spdx.json",
    "install.sh",
    "install.ps1"
  ]
}
```

| Item | State |
|------|-------|
| Workspace `[workspace.package] version` | `4.0.0` |
| GitHub Release `v4.0.0` | **Published** (`prerelease=false`) |
| Tag `v4.0.0` on origin | Present |
| `x86_64-apple-darwin` archive | **GAP** — macos-13 job stayed queued; owner: Tech Lead DARE CLI (workflow now prefers macos-14 cross-compile) |
| `dare-self` `DEFAULT_CHANNEL` | Remains **`beta`** (unchanged) |
| Capability matrix | Unchanged |
| `SHA256SUMS.sig` | Soft-fail text (`signing skipped — no key / OIDC unavailable`) — known for `dare self update` fail-closed |

## Prior blockers (resolved)

Earlier attempts failed on org Actions billing / stuck queues (`30636494439`, `30645155764`, `30661342770`). Billing unblocked; linux aarch64 fixed by native `ubuntu-24.04-arm` (cross+openssl-sys failed).

## Channel reminder

Default `dare self` channel stays **`beta`**. Pin stable explicitly:

```bash
dare self update --version 4.0.0 --yes
# or, after channel cutover tasks:
# dare self update --channel stable --yes
```
