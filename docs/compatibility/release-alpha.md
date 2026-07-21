# Release alpha (native channel)

> **DEC-016** · **ADR-008** · Workflow: [`.github/workflows/release.yml`](../../.github/workflows/release.yml)

## Purpose

Publish installable **native** `dare` binaries (no npm) as GitHub **prereleases** from alpha tags. The TypeScript package `@dewtech/dare-cli@3.18.1` remains a parallel legacy channel until cutover (microplan 056).

## Triggers

| Event | Behavior |
|-------|----------|
| Push tag `v*-alpha*` / `v*-alpha.*` | Build 5 targets → meta → **create prerelease** |
| `workflow_dispatch` | Default `dry_run=true`: build + artifacts **only** (no Release). Publish only if `dry_run=false` **and** ref is a tag |

## Matrix (5 targets)

| Target | Runner | Archive |
|--------|--------|---------|
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `.tar.gz` |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` + `cross` | `.tar.gz` |
| `x86_64-apple-darwin` | `macos-13` | `.tar.gz` |
| `aarch64-apple-darwin` | `macos-14` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-latest` | `.zip` |

Aligned with CI microplan 003 runners for macOS (not `macos-latest`).

## Artifact naming

```text
dare-${VERSION}-${TARGET}.tar.gz   # Unix
dare-${VERSION}-${TARGET}.zip      # Windows
VERSION = git tag name (e.g. v0.1.0-alpha.1) or "dev" on dry-run without tag
```

Stage directory contains only the binary (`dare` / `dare.exe`).

## Meta artifacts

| File | Rule |
|------|------|
| `SHA256SUMS` | SHA-256 of all `.tar.gz`/`.zip`, sorted, `sha256sum` two-space format |
| `SHA256SUMS.sig` | cosign blob signature **or** ASCII line starting with `signing skipped` (alpha soft-fail) |
| `SBOM.spdx.json` | SPDX-2.3 minimal document (`Tool: dare-release-alpha`) |
| `install.sh` / `install.ps1` | Copied from `installers/` into the Release |

## Installers

Paths: `installers/install.sh`, `installers/install.ps1`

| Variable | Required | Default |
|----------|----------|---------|
| `DARE_VERSION` **or** `DARE_LOCAL_ARCHIVE` | one of | — |
| `DARE_REPO` | no | `dewtech/dare-cli` |
| `DARE_INSTALL_BASE` | no | `https://github.com/${REPO}/releases/latest/download` |
| `DARE_PREFIX` | no | Unix `$HOME/.local`; Windows `%LOCALAPPDATA%\dare` |

Binary lands in `${DARE_PREFIX}/bin`. Installer runs `dare --version` (must succeed).

- **sh:** remote download **must** verify `SHA256SUMS` (fail on mismatch / missing sums). Exit `2` if neither VERSION nor LOCAL set.
- **ps1:** checksum mismatch throws; sums download failure → warning and continue (alpha).

## Local smoke

```bash
# Unix / Git Bash
bash scripts/smoke-release-install.sh

# Windows PowerShell
.\scripts\smoke-release-install.ps1
```

Builds host release binary, packages, writes meta, installs into `dist/smoke/prefix`, asserts `--version` matches `^dare `, and greps the five targets + `macos-13`/`macos-14` in `release.yml`.

## Local verify (container)

```bash
docker compose -f docker-compose.ci.yml config
# optional: docker build -f Dockerfile.rust .
```

Inherits microplan 003 images (`Dockerfile.rust`, `docker-compose.ci.yml`). No product release image in 015.

## Version note

Git tag names assets (`v0.1.0-alpha.N`). Clap/`Cargo.toml` may remain `0.1.0-alpha.0` until an explicit bump — see ADR-008.

## Permissions

Workflow: `contents: write`, `id-token: write` (OIDC cosign). Uses `GITHUB_TOKEN` only — no deploy keys in repo.
