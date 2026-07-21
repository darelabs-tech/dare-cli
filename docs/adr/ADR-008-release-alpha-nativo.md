---
id: ADR-008
title: "Release alpha nativo (GitHub Releases, sem npm)"
status: Accepted
date: 2026-07-21
deciders: ["dare-labs"]
tags: ["release", "supply-chain", "alpha", "ci"]
---

## Contexto

O rewrite Rust do DARE CLI precisa de um canal de distribuição **instalável sem npm**, em paralelo ao pacote legado `@dewtech/dare-cli@3.18.1`. O microplano 003 já cobre CI multi-target e checksums em artifacts; falta um pipeline de **tags alpha** que publique packages, checksums, SBOM e installers.

## Decisão

1. **Canal:** GitHub Releases **prerelease** apenas, dispara em tags `v*-alpha*` / `v*-alpha.*`. `workflow_dispatch` faz dry-run por default (`dry_run=true`) e não publica Release acidentalmente.
2. **Cinco targets** (paridade Documento Mestre / CI 003): linux x64/arm64, darwin x64/arm64, windows x64. Runners macOS: **`macos-13`** e **`macos-14`** (não `macos-latest`). Linux aarch64 via **`cross`** em `ubuntu-latest` (fallback documentado: runner ARM nativo se `cross` falhar de forma recorrente).
3. **SBOM:** MUST = `SBOM.spdx.json` SPDX-2.3 **mínimo** válido no Release. Tool-generated (syft/cyclonedx) = SHOULD futuro; não bloqueia alpha.
4. **Assinatura:** cosign `sign-blob` de `SHA256SUMS` é **best-effort** na alpha — sempre existe `SHA256SUMS.sig` (assinatura real **ou** texto `signing skipped…`). Stable endurece depois.
5. **Versão clap vs tag:** a tag Git nomeia archives; a versão embutida no binário pode permanecer `0.1.0-alpha.0` até bump explícito.
6. **Installers:** `install.sh` / `install.ps1` exigem `DARE_VERSION` ou `DARE_LOCAL_ARCHIVE` (filenames incluem versão). npm **não** é substituído neste ADR.

## Consequências

- Developers alpha instalam via Release assets + scripts, independentes do registry npm.
- Operações devem preferir dry-run antes da primeira tag.
- Auditores veem checksums + SBOM mínimo; assinatura cosign pode ser skip na alpha.
- Cutover stable / package managers / self-update ficam fora (053, 056).

## Referências

- [`docs/compatibility/release-alpha.md`](../compatibility/release-alpha.md)
- [`.github/workflows/release.yml`](../../.github/workflows/release.yml)
- DEC-016 (decision log)
- Microplano `015-pipeline-de-release-nativo-alpha`
