# Self-update stable channel smoke — v4.0.0

> Microplano **056** · mp056-004 · Date: 2026-07-31  
> Host: Windows (operator worktree) · Prefix: ephemeral `DARE_SELF_HOME` temp dir

## Preconditions

- Public Release https://github.com/darelabs-tech/dare-cli/releases/tag/v4.0.0 (`isPrerelease=false`)
- `DEFAULT_CHANNEL` remains **`beta`** in `crates/dare-self` (unchanged)
- Default release repo: `darelabs-tech/dare-cli`

## Command

```bash
# Temp self-home (no mutation of operator ~/.dare/self)
export DARE_SELF_HOME="$(mktemp -d)"   # PowerShell: New-TemporaryFile / temp dir

dare self update --channel stable --dry-run --json --no-color
```

## Result

| Check | Expected | Observed |
|-------|----------|----------|
| Exit | **0** | **0** |
| `data.targetTag` | `v4.0.0` | `v4.0.0` |
| `data.channel` | `stable` | `stable` |
| `data.assetName` | host triple asset | `dare-v4.0.0-x86_64-pc-windows-msvc.zip` |
| Exit 4 / `stable channel has no non-prerelease` | FAIL if seen | **not seen** |
| Exit 6 (cosign / signing skipped) | FAIL if seen on dry-run | **N/A** (dry-run plans only; no verify) |

### Capture (`--json`)

```json
{"ok":true,"data":{"schemaVersion":1,"ok":true,"mode":"update","channel":"stable","currentVersion":"4.0.0","targetTag":"v4.0.0","targetTriple":"x86_64-pc-windows-msvc","assetName":"dare-v4.0.0-x86_64-pc-windows-msvc.zip","actions":["download","verify-sha256","verify-sig","backup","replace"]}}
```

Dry-run resolves the GitHub Releases API (`/releases/latest`) and does **not** download or run cosign. Full `dare self update --channel stable --yes` remains **fail-closed** on the published `SHA256SUMS.sig` soft-fail prefix (`signing skipped`) → exit **6** until a real cosign signature is attached (known issue in RELEASE-NOTES-stable).

## Packaging cross-check

| Manifest | Filled from SUMS |
|----------|------------------|
| `packaging/homebrew/dare.rb` | aarch64-apple-darwin (`831878c5…`) — Intel **GAP** |
| `packaging/winget/DareLabs.DareCli.yaml` | windows-msvc zip (`3E93F0ED…`) |

See [`../../../packaging/README.md`](../../../packaging/README.md).
