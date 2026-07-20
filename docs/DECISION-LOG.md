# Decision Log

Registro append-only de decisões de governança dos microplanos DARE CLI.

| decision_id | date | summary | adr_refs | owner | status |
|-------------|------|---------|----------|-------|--------|
| DEC-001 | 2026-07-20 | Gates cargo transferidos ao microplano 002 | n/a | Tech Lead DARE CLI | active |
| DEC-002 | 2026-07-20 | Workspace Rust: toolchain 1.85.0, edition 2021, Apache-2.0, versão 0.1.0-alpha.0 | n/a | Tech Lead DARE CLI | active |
| DEC-003 | 2026-07-20 | RF-14 placeholder: épico GitHub Issues do microplano 002 pendente; rastreio via TASKS-002 | n/a | Tech Lead DARE CLI | active |

## Notas

- **DEC-001:** gates `cargo fmt`, `cargo clippy`, `cargo test` e workspace Rust deferidos para microplano 002; ciclo 001 valida governança, docs, baseline e scripts Node de verificação.
- **DEC-002:** workspace Cargo com cinco crates (`dare-cli`, `dare-core`, `dare-contracts`, `dare-config`, `dare-assets`); `rust-toolchain.toml` pin `1.85.0` + components `rustfmt`/`clippy`; MSRV = canal (`rust-version = "1.85.0"`); licença **Apache-2.0**; versão workspace **`0.1.0-alpha.0`** (≠ npm `3.18.1` — rewrite intencional); help CLI em inglês ([`language-policy.md`](compatibility/language-policy.md)); sem `[build] target` global (RS-06); detalhes em [`rust-msrv.md`](compatibility/rust-msrv.md).
- **RF-13 (épico placeholder):** Issue principal + subtarefas rastreáveis para RF-01–RF-11 (SHOULD) — pendente abertura no GitHub Issues. Até lá, o checklist em [`docs/compatibility/README.md`](compatibility/README.md) § Ciclo 0 governance e este log servem como rastreador. Subtarefas sugeridas: uma issue por RF MUST (RF-01…RF-11) vinculada ao microplano 001.
