# CI cross-platform e qualidade (Ciclo CI 003)

Documentação dos workflows canônicos do DARE CLI nativo em Rust após o microplano 003.

## Workflows

| Workflow | Papel |
|----------|-------|
| [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) | Qualidade em PR/push: fmt, clippy `-D warnings`, test, `cargo audit`, `cargo deny` |
| [`.github/workflows/build.yml`](../../.github/workflows/build.yml) | Matrix 5 targets: release + smoke + SHA-256 + upload-artifact (7 dias) |
| [`.github/workflows/governance-001.yml`](../../.github/workflows/governance-001.yml) | Governança docs/scripts Node — **pipeline separado**, sem misturar tokens |

**Fonte de verdade Rust:** `ci.yml` + `build.yml`. O workflow legado `rust-workspace-002.yml` foi **removido** no fechamento do microplano 003 (DEC-004 / mp003-008).

## Targets ↔ runners ↔ artifacts

| Target triple | Runner | Artifact name | Binário |
|---------------|--------|---------------|---------|
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `dare-x86_64-unknown-linux-gnu` | `dare` |
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` | `dare-aarch64-unknown-linux-gnu` | `dare` |
| `x86_64-apple-darwin` | `macos-13` | `dare-x86_64-apple-darwin` | `dare` |
| `aarch64-apple-darwin` | `macos-14` | `dare-aarch64-apple-darwin` | `dare` |
| `x86_64-pc-windows-msvc` | `windows-latest` | `dare-x86_64-pc-windows-msvc` | `dare.exe` |

Cada artifact inclui o binário e `dare.sha256` (hex SHA-256 do ficheiro).

## Como baixar um artifact

1. Abra o run do workflow **build** no GitHub Actions.
2. Em **Artifacts**, escolha `dare-<target-triple>`.
3. Extraia o ZIP; o binário e `dare.sha256` estão na raiz do archive (staging `bin/`).

## Como verificar `dare.sha256`

```bash
# Unix
sha256sum -c dare.sha256
# ou: sha256sum dare && cat dare.sha256

# Windows (PowerShell)
(Get-FileHash .\dare.exe -Algorithm SHA256).Hash.ToLower()
Get-Content .\dare.sha256
```

Os hashes devem coincidir (hex minúsculo, conforme gerado no job).

## Smoke local

```bash
cargo build -p dare-cli
bash scripts/ci/smoke-dare.sh target/debug/dare          # Unix
pwsh scripts/ci/smoke-dare.ps1 -Bin target/debug/dare.exe # Windows
```

Esperado: `--version` → `dare 0.1.0-alpha.0`; `--help` contém `Usage:` ou `--version`.

## Pins de ferramentas (CI)

| Ferramenta | Versão | Nota |
|------------|--------|------|
| Rust | 1.85.0 | `rust-toolchain.toml` + `dtolnay/rust-toolchain@1.85.0` |
| `Swatinem/rust-cache` | v2.7.8 | |
| `cargo-audit` | **0.22.0** | 0.21.2 falha no advisory-db com CVSS 4.0 |
| `cargo-deny` | **0.18.6** | 0.18.2 mesma classe de incompatibilidade CVSS 4.0 |
| `actions/upload-artifact` | v4.6.0 | retenção 7 dias |

Política: [`deny.toml`](../../deny.toml).

## Security (RS-01…RS-08)

| ID | Controlo | Status |
|----|----------|--------|
| RS-01 | Paths/matrix fixos no YAML — sem interpolação insegura de input não confiável | ✅ |
| RS-02 | Sem secrets/PII em artifacts ou logs de smoke | ✅ |
| RS-03 | Permissions mínimas (`contents: read`; `actions: write` só no build para upload) | ✅ |
| RS-04 | `cargo audit` + `cargo deny check` sem HIGH/CRITICAL / violação de política | ✅ |
| RS-05 | Sem secrets hardcoded em YAML — só `GITHUB_TOKEN` default | ✅ |
| RS-06 | Smoke com argv direto (`smoke-dare.sh` / `.ps1`) — sem `eval` | ✅ |
| RS-07 | Checksum `dare.sha256` anexado a cada artifact | ✅ |
| RS-08 | `governance-001.yml` intacto e isolado dos workflows Rust | ✅ |

## Release notes — Ciclo CI 003

- Introduzidos `ci.yml` e `build.yml` como fonte de verdade de qualidade e artifacts multi-OS.
- Linux ARM via runner nativo `ubuntu-24.04-arm` (sem `cross`).
- Checksums SHA-256 nos artifacts; SBOM adiado (COULD).
- Ver [DEC-004](../DECISION-LOG.md).

## Referências

- Design: `DARE/DESIGN-003-ci-cross-platform-e-qualidade.md`
- Blueprint: `DARE/BLUEPRINT-003-ci-cross-platform-e-qualidade.md`
- MSRV: [`rust-msrv.md`](rust-msrv.md)
