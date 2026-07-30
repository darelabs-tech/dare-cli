# CLI self-update (`dare self`)

> **DEC-054** · Microplano 053 · Library: `crates/dare-self` · CLI: `commands/self_cmd.rs` · Packaging: `packaging/`

## Purpose

Manage the **dare CLI binary itself**: download a GitHub Release asset, verify SHA-256 + cosign, atomically replace `current_exe`, rollback from backup, or uninstall the binary.

**Not** project-asset sync. That is `dare update` (microplanos 021–022 / DEC-022–023) — refreshes harness/templates under a ProjectRoot.

| Surface | Role |
|---------|------|
| `dare self update` | Plan / download / verify / replace binary |
| `dare self rollback` | Restore `~/.dare/self/backup/dare[.exe]` |
| `dare self uninstall` | Remove **only** `current_exe()` (no project wipe) |
| `dare update` | Project assets under ProjectRoot (distinct) |

## Commands

```text
dare self update [--channel beta|stable] [--version <tag>] [--dry-run] [--yes] [--force-unlock] [--json]
dare self rollback [--yes] [--json]
dare self uninstall [--yes] [--json]
```

| Flag | Applies to | Effect |
|------|------------|--------|
| `--channel` | update | `beta` (default) \| `stable`; mutually exclusive with `--version` |
| `--version` | update | Explicit release tag (`vX` / `X`); mutually exclusive with `--channel` |
| `--dry-run` | update | Plan only; no download or replace; exit **0** |
| `-y` / `--yes` | all | Skip confirmation; **required** when non-interactive |
| `--force-unlock` | update | Drop stale `~/.dare/self/update.lock` before acquire |
| `--json` | all | JSON report on stdout (global CLI flag) |

## Exit codes

| Code | When |
|------|------|
| **0** | Success (update / rollback / uninstall / dry-run ok) |
| **2** | Usage / unknown subcommand / `--channel`+`--version` together / unknown channel |
| **3** | Path resolve failure (`current_exe`, self home) |
| **4** | Invalid input: lock held, stable empty, no backup, confirmation denied / missing `--yes` |
| **5** | Network / I/O / HTTP non-2xx / timeout |
| **6** | Checksum mismatch, signature fail, cosign missing, or `signing skipped` |

Exit **6** aligns semantically with `dare guard` (integrity / verify).

## Channels

| Channel | Resolution |
|---------|------------|
| **`beta`** (default) | Latest GitHub Release with `prerelease: true` (alpha/beta product track) |
| **`stable`** | Latest non-prerelease; if none → exit **4** (`stable channel has no non-prerelease GitHub Release`) — no silent redirect to beta |
| `--version <tag>` | Exact tag match (`X` or `vX`) |

## Cosign: fail-closed (≠ ADR-008 installers)

| Path | Policy |
|------|--------|
| **`dare self update`** | **Fail-closed**: missing cosign, verify-blob failure, or `.sig` prefix `signing skipped` → exit **6**; binary unchanged |
| **Installers / release alpha (ADR-008 / DEC-016)** | Cosign **soft-fail** allowed for bootstrap scripts |

Dev-only escape: `DARE_SELF_ALLOW_UNSIGNED=1` skips cosign **after** checksum OK (stderr warning). Not for production docs or CI.

## State & packaging

| Path | Role |
|------|------|
| `~/.dare/self/` (override `DARE_SELF_HOME`) | Lock, backup, temp downloads |
| `packaging/homebrew/dare.rb` | Homebrew formula template |
| `packaging/winget/DareLabs.DareCli.yaml` | WinGet singleton manifest |
| `packaging/README.md` | Placeholder fill checklist |

Scoop is **out of scope** for v1 (BLUEPRINT T-10).

## Capability

`dare-self` → `cli_commands: ["self"]` in `assets/capability-matrix.yml`.

## Examples

```bash
dare self update --dry-run
dare self update --channel beta --yes
dare self update --channel stable --yes
dare self update --version v0.1.0-alpha.2 --yes
dare self rollback --yes
dare self uninstall --yes
# project assets (different command):
dare update --dry-run
```

## Out of scope (053)

Hardening / parity (**054**), npm cutover (**056**), Scoop, Docker packaging, background auto-update, rewriting `dare update` asset pipeline.
