# Stable smoke matrix — v4.0.0

> Microplano **056** · mp056-003 · Product tag **`v4.0.0`** / core **`4.0.0`**  
> **GitHub Release published:** `true` — https://github.com/darelabs-tech/dare-cli/releases/tag/v4.0.0 (`isPrerelease=false`)

## Commands (MUST)

| Command | Purpose |
|---------|---------|
| `dare --version` | Binary identity |
| `dare info` | Read-only diagnostics |
| `dare --help` | Critical help surface |

## OS coverage

| OS | Runner | Status | Evidence |
|----|--------|--------|----------|
| Windows | Local operator host + published `x86_64-pc-windows-msvc` zip | **PASS** | [`windows.md`](windows.md) |
| Linux | CI build success (`x86_64` + `aarch64` via `ubuntu-24.04-arm`) — no interactive host smoke | **CI build OK** | Actions [`30662777245`](https://github.com/darelabs-tech/dare-cli/actions/runs/30662777245); interactive smoke **GAP** (owner: Tech Lead) |
| macOS | CI build success (`aarch64-apple-darwin`); `x86_64-apple-darwin` not published | **Partial** | aarch64 CI OK; intel macOS asset **GAP** |

### Gaps

- Interactive Linux/macOS install smoke against the published archives was not run on an operator host.
- **`x86_64-apple-darwin`** archive missing from the Release (macos-13 queue stall).

## Release gate

```bash
gh release view v4.0.0 --json isPrerelease,tagName,assets,url
# Expected: isPrerelease=false, assets non-empty, url https://github.com/darelabs-tech/dare-cli/releases/tag/v4.0.0
```

## Self channel (mp056-004)

| Check | Status | Evidence |
|-------|--------|----------|
| `dare self update --channel stable --dry-run` → `v4.0.0` | **PASS** | [`self-channel-stable.md`](self-channel-stable.md) |
| Default `dare self` channel | **`beta`** (unchanged) | `DEFAULT_CHANNEL` in `dare-self` |
| Homebrew / WinGet manifests | **filled** (no `REPLACE_ME`) | [`../../../packaging/README.md`](../../../packaging/README.md) |

See [`../publish-stable-checklist.md`](../publish-stable-checklist.md).
