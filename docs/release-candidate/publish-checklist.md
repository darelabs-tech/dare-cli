# Publish checklist — RC `v4.0.0-rc1`

> **Status:** BLOCKED (Actions runners)  
> **Operator:** Wanderson Leandro de Oliveira (`wleandrooliveira`)  
> **Signed:** `operator` + `blocked:actions_billing`  
> **Date:** 2026-07-31  
> **Microplano:** 055 · mp055-005

## Verdict

GitHub API credentials **work** (`gh auth` OK; repo `ADMIN`). The annotated tag **`v4.0.0-rc1`** was pushed to `origin`.

Full ADR-008 multi-target assets could **not** be produced: GitHub Actions jobs failed immediately with:

> The job was not started because recent account payments have failed or your spending limit needs to be increased.

Therefore there is **no** invented Release download URL. Local release smoke evidence is under [`../pilot/results/rc-smoke/`](../pilot/results/rc-smoke/).

| Item | State |
|------|-------|
| Tag `v4.0.0-rc1` on `origin` | Present (`45d1f8084e73e748446bbcb62770d004ae28cc5c`) |
| Branch `mp055-005-rc-v4.0.0-rc1` | Pushed |
| `release.yml` RC tag patterns (`v*-rc*` / `v*-rc.*`) | Committed |
| GitHub Release + 5-target assets + `SHA256SUMS` + `.sig` | **Blocked** — Actions billing / spending limit |
| Local smoke (`--version` / `info` / `--help` on Windows) | PASS — see rc-smoke |

## Unblock (org billing)

1. Fix GitHub org **Billing & plans** / spending limit for `darelabs-tech`.
2. Re-run the release workflow on the existing tag:

```bash
gh workflow run release.yml --ref v4.0.0-rc1
# or:
gh run rerun 30636494439 --failed
```

Workflow URL (attempt that failed on billing):  
https://github.com/darelabs-tech/dare-cli/actions/runs/30636494439

3. Confirm prerelease exists with assets:

```bash
gh release view v4.0.0-rc1
```

## Exact commands (if recreating from scratch)

After billing is healthy, from a commit that includes the RC notes + `release.yml` RC triggers:

```bash
# Tag (skip if already on origin)
git tag -a v4.0.0-rc1 -m "Release candidate v4.0.0-rc1 (microplano 055)"
git push origin v4.0.0-rc1

# release.yml on push tags v*-rc* builds 5 targets, writes SHA256SUMS + SHA256SUMS.sig (+ SBOM),
# then softprops/action-gh-release creates prerelease=true with dist/**

# Manual fallback (only after local/CI artifacts exist for ALL five targets):
gh release create v4.0.0-rc1 \
  --prerelease \
  --title "v4.0.0-rc1" \
  --notes-file docs/release-candidate/RELEASE-NOTES.md \
  dist/dare-v4.0.0-rc1-x86_64-unknown-linux-gnu.tar.gz \
  dist/dare-v4.0.0-rc1-aarch64-unknown-linux-gnu.tar.gz \
  dist/dare-v4.0.0-rc1-x86_64-apple-darwin.tar.gz \
  dist/dare-v4.0.0-rc1-aarch64-apple-darwin.tar.gz \
  dist/dare-v4.0.0-rc1-x86_64-pc-windows-msvc.zip \
  dist/SHA256SUMS \
  dist/SHA256SUMS.sig \
  dist/SBOM.spdx.json \
  installers/install.sh \
  installers/install.ps1
```

Do **not** publish a partial one-OS Release as if it were ADR-008 complete.

## Local smoke already done

```powershell
$env:CARGO_TARGET_DIR = "target-mp055-005"
$env:DARE_SMOKE_VERSION = "v4.0.0-rc1"
.\scripts\smoke-release-install.ps1
# then: dare --version / dare info / dare --help  (Windows PASS; Linux/macOS GAP documented)
```

## Channel reminder

Default `dare self` channel stays **`beta`**. Install this RC with:

```bash
dare self update --version 4.0.0-rc1 --yes
```

(only after a real signed Release exists; fail-closed cosign applies).
