# Publish checklist — stable `v4.0.0`

> **Status:** `blocked:actions_billing`  
> **publish_ready:** `false`  
> **Operator:** Wanderson Leandro de Oliveira (`wleandrooliveira`)  
> **Date:** 2026-07-31  
> **STABLE_TAG:** `v4.0.0`  
> **Microplano:** 056 · mp056-001

## Verdict

GitHub API credentials work (`gh auth` OK; repo `darelabs-tech/dare-cli`). Workspace version is bumped to **`4.0.0`** and `release.yml` now triggers on stable tags (`v[0-9]+.[0-9]+.[0-9]+`) with:

```yaml
prerelease: ${{ contains(github.ref_name, '-rc') || contains(github.ref_name, '-alpha') }}
```

so **`v4.0.0` is not a prerelease**.

Full ADR-008 multi-target assets for stable **cannot** be produced yet: the RC attempt on tag `v4.0.0-rc1` still shows Actions jobs failing / stuck because of account payments or spending limit (same gate documented in [`../release-candidate/publish-checklist.md`](../release-candidate/publish-checklist.md)).

There is **no** GitHub Release download URL for `v4.0.0` — do not invent one.

| Item | State |
|------|-------|
| Workspace `[workspace.package] version` | `4.0.0` |
| `release.yml` stable tag pattern + conditional `prerelease` | Ready (this commit) |
| Actions billing / spending limit (`darelabs-tech`) | **Blocked** |
| GitHub Release `v4.0.0` + 5-target assets + `SHA256SUMS` + `.sig` | **Not published** |
| `dare-self` `DEFAULT_CHANNEL` | Remains **`beta`** (unchanged) |
| Capability matrix | Stays **51** (unchanged) |

## Unblock (org billing)

1. Fix GitHub org **Billing & plans** / spending limit for `darelabs-tech`.
2. After this branch is on the default branch (or the intended release commit), create and push the annotated tag:

```bash
git tag -a v4.0.0 -m "Stable release v4.0.0 (microplano 056)"
git push origin v4.0.0
```

3. Confirm the workflow run succeeds (5 targets + checksums + sig):

```bash
gh run list --workflow=release.yml --branch v4.0.0 --limit 5
gh run watch
```

4. Confirm the Release is **not** prerelease and lists ADR-008 assets:

```bash
gh release view v4.0.0
```

Evidence of the prior billing failure (RC):  
https://github.com/darelabs-tech/dare-cli/actions/runs/30636494439

## Manual publish path (only after real multi-target artifacts exist)

Do **not** run these until billing is healthy **or** you have locally/CI-built packages for **all five** targets plus `SHA256SUMS` and `SHA256SUMS.sig`.

```bash
# Create the stable Release (prerelease=false)
gh release create v4.0.0 \
  --title "v4.0.0" \
  --notes "DARE CLI stable v4.0.0" \
  dist/dare-v4.0.0-x86_64-unknown-linux-gnu.tar.gz \
  dist/dare-v4.0.0-aarch64-unknown-linux-gnu.tar.gz \
  dist/dare-v4.0.0-x86_64-apple-darwin.tar.gz \
  dist/dare-v4.0.0-aarch64-apple-darwin.tar.gz \
  dist/dare-v4.0.0-x86_64-pc-windows-msvc.zip \
  dist/SHA256SUMS \
  dist/SHA256SUMS.sig \
  dist/SBOM.spdx.json \
  installers/install.sh \
  installers/install.ps1

# If the Release shell already exists without assets:
gh release upload v4.0.0 \
  dist/dare-v4.0.0-x86_64-unknown-linux-gnu.tar.gz \
  dist/dare-v4.0.0-aarch64-unknown-linux-gnu.tar.gz \
  dist/dare-v4.0.0-x86_64-apple-darwin.tar.gz \
  dist/dare-v4.0.0-aarch64-apple-darwin.tar.gz \
  dist/dare-v4.0.0-x86_64-pc-windows-msvc.zip \
  dist/SHA256SUMS \
  dist/SHA256SUMS.sig \
  dist/SBOM.spdx.json \
  installers/install.sh \
  installers/install.ps1
```

Do **not** publish a partial one-OS Release as if it were ADR-008 complete.

After a real Release exists, update this file:

- `Status:` `ready`
- `publish_ready:` `true`
- paste the real `gh release view v4.0.0 --json url,isPrerelease,assets` output (no invented URLs)

## Channel reminder

Default `dare self` channel stays **`beta`**. Pin stable explicitly when ready:

```bash
dare self update --version 4.0.0 --yes
# or, after channel cutover tasks:
# dare self update --channel stable --yes
```
