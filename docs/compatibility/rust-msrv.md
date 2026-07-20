# Política MSRV — workspace Rust DARE CLI

> Microplano **002** · [DEC-002](../DECISION-LOG.md)

## Versão mínima suportada (MSRV)

| Campo | Valor |
|-------|-------|
| **MSRV** | `1.85.0` |
| **Edition** | `2021` |
| **Canal toolchain** | `1.85.0` (pin em `rust-toolchain.toml`) |
| **Versão workspace** | `0.1.0-alpha.0` |

A MSRV é definida em três lugares que devem permanecer alinhados:

1. `rust-toolchain.toml` — canal usado por desenvolvedores e CI
2. `Cargo.toml` workspace — `rust-version = "1.85.0"` em `[workspace.package]`
3. Este documento

## Reprodução local

```bash
rustup show active-toolchain   # deve reportar 1.85.0 (via rust-toolchain.toml)
rustc --version                # rustc 1.85.0 (...)
cargo --version
```

Se o toolchain não estiver instalado:

```bash
rustup toolchain install 1.85.0 --component rustfmt clippy
```

## Upgrade path

1. **Proposta:** abrir issue/PR documentando motivo (features da linguagem, deps, segurança).
2. **Bump coordenado:** atualizar `rust-toolchain.toml`, `rust-version` no workspace e este arquivo na mesma PR.
3. **Validação:** `cargo check --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`.
4. **CI:** workflow `rust-workspace-002.yml` (microplano 002+) deve usar o novo pin.
5. **Decisão:** registrar entrada no [`DECISION-LOG.md`](../DECISION-LOG.md) se a MSRV mudar após o alpha.

## Diferenças vs baseline npm 3.18.1

| Aspecto | npm `@dewtech/dare-cli@3.18.1` | Binário Rust (002+) |
|---------|----------------------------------|---------------------|
| Versão semver | `3.18.1` | `0.1.0-alpha.0` (rewrite novo — intencional) |
| Idioma CLI | conforme legado | inglês (EN) para surface nova — ver [`language-policy.md`](language-policy.md) |
| Runtime | Node.js | binário nativo |

Classificação de paridade: ver [`classification-matrix.md`](classification-matrix.md).

## Security gate 002

| RS | Controle | Evidência |
|----|----------|-----------|
| RS-01 | Args CLI via clap | `crates/dare-cli` |
| RS-02 | Sem secrets em código | crates + `.env.rust.example` só nomes |
| RS-03 | Libs sem I/O de contrato | APIs ping/schema apenas |
| RS-04 | `cargo audit` sem HIGH/CRITICAL | gate deste microplano |
| RS-05 | Secrets via env | `RUST_LOG` opcional |
| RS-06 | Sem `[build] target` global | ausência de `.cargo/config.toml` com target |
| RS-07 | `validate_nonempty_name`; sem shell | `dare-core` |
| RS-08 | LICENSE + CODEOWNERS | raiz do repo |

## Restrições

- **Proibido:** `.cargo/config.toml` com `[build] target` global (RS-06) — quebra cross-platform no Windows/Linux.
- Bumps de MSRV exigem aprovação do time via CODEOWNERS em `crates/` e `rust-toolchain.toml`.
