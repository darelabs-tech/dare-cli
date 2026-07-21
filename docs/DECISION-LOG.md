# Decision Log

Registro append-only de decisões de governança dos microplanos DARE CLI.

| decision_id | date | summary | adr_refs | owner | status |
|-------------|------|---------|----------|-------|--------|
| DEC-001 | 2026-07-20 | Gates cargo transferidos ao microplano 002 | n/a | Tech Lead DARE CLI | active |
| DEC-002 | 2026-07-20 | Workspace Rust: toolchain 1.85.0, edition 2021, Apache-2.0, versão 0.1.0-alpha.0 | n/a | Tech Lead DARE CLI | active |
| DEC-003 | 2026-07-20 | RF-14 placeholder: épico GitHub Issues do microplano 002 pendente; rastreio via TASKS-002 | n/a | Tech Lead DARE CLI | active |
| DEC-004 | 2026-07-20 | CI cross-platform: matrix 5 runners, pins audit/deny, checksums; remoção planejada rust-workspace-002 | n/a | Tech Lead DARE CLI | active |
| DEC-005 | 2026-07-20 | Erros/tracing/saída CLI: ErrorKind exit 1–5, JSON err→stdout, --json/--no-color, redact, uuid | ADR-002 | Tech Lead DARE CLI | active |
| DEC-006 | 2026-07-20 | Path safety: jail ProjectRoot, symlink deny-escape, .dare/backups, fs4 try_lock, camino | n/a | Tech Lead DARE CLI | active |
| DEC-007 | 2026-07-21 | Process safety: std::process (no tokio), env denylist, kill_tree 0.2.4, exit 124/cancel -1 | n/a | Tech Lead DARE CLI | active |
| DEC-008 | 2026-07-21 | Persisted contracts: flatten nested, yaml_serde 0.10.4 as serde_yaml, 2MiB cap, no garde | ADR-002 | Tech Lead DARE CLI | active |
| DEC-009 | 2026-07-21 | Config: CLI>env>file>default; dry-run zero-write; schemaVersion só com flag; JSON Pointer | ADR-002 | Tech Lead DARE CLI | active |
| DEC-010 | 2026-07-21 | Assets: rust-embed + assets/manifest.yml SHA-256; templates canónicos; .claude external | n/a | Tech Lead DARE CLI | active |
| DEC-011 | 2026-07-21 | Capabilities ADR-007: matrix 49 Claude; tipos em dare-assets; harness crate defer 011+ | ADR-007 | Tech Lead DARE CLI | active |
| DEC-012 | 2026-07-21 | Adapter Claude: dare-harness; install/validate/detect; preserve unmanaged | ADR-007 | Tech Lead DARE CLI | active |
| DEC-013 | 2026-07-21 | Adapter Cursor: commands from matrix; .cursorrules managed; preserve | ADR-007 | Tech Lead DARE CLI | active |
| DEC-014 | 2026-07-21 | Adapter Codex: AGENTS.md + skills; .agents share; UPDATE_HARNESS_IDES includes codex | ADR-007 | Tech Lead DARE CLI | active |
| DEC-015 | 2026-07-21 | Adapter Antigravity: rules + commands + shared .agents/skills; frontmatter validate | ADR-007 | Tech Lead DARE CLI | active |
| DEC-016 | 2026-07-21 | Release alpha: 5 targets GH Actions; SHA256SUMS+SBOM; install.sh/ps1; optional cosign | ADR-008 | Tech Lead DARE CLI | active |
| DEC-017 | 2026-07-21 | dare welcome: TTY banner; --no-banner / DARE_NO_BANNER; no dare new; snapshots | — | Tech Lead DARE CLI | active |

## Notas

- **DEC-001:** gates `cargo fmt`, `cargo clippy`, `cargo test` e workspace Rust deferidos para microplano 002; ciclo 001 valida governança, docs, baseline e scripts Node de verificação.
- **DEC-002:** workspace Cargo com cinco crates (`dare-cli`, `dare-core`, `dare-contracts`, `dare-config`, `dare-assets`); `rust-toolchain.toml` pin `1.85.0` + components `rustfmt`/`clippy`; MSRV = canal (`rust-version = "1.85.0"`); licença **Apache-2.0**; versão workspace **`0.1.0-alpha.0`** (≠ npm `3.18.1` — rewrite intencional); help CLI em inglês ([`language-policy.md`](compatibility/language-policy.md)); sem `[build] target` global (RS-06); detalhes em [`rust-msrv.md`](compatibility/rust-msrv.md).
- **RF-13 (épico placeholder):** Issue principal + subtarefas rastreáveis para RF-01–RF-11 (SHOULD) — pendente abertura no GitHub Issues. Até lá, o checklist em [`docs/compatibility/README.md`](compatibility/README.md) § Ciclo 0 governance e este log servem como rastreador. Subtarefas sugeridas: uma issue por RF MUST (RF-01…RF-11) vinculada ao microplano 001.
- **DEC-004:** workflows `ci.yml` + `build.yml`; runners `ubuntu-latest`, `ubuntu-24.04-arm`, `macos-13`, `macos-14`, `windows-latest`; cache `Swatinem/rust-cache@v2.7.8`; `cargo-audit@0.22.0` e `cargo-deny@0.18.6` (pins Blueprint 0.21.2/0.18.2 incompatíveis com advisory-db CVSS 4.0 no MSRV 1.85); checksums SHA-256 MUST técnico; SBOM fora; remoção de `rust-workspace-002.yml` no fechamento mp003-008; detalhes em [`ci-cross-platform.md`](compatibility/ci-cross-platform.md).
- **DEC-005:** microplano 004 — `ErrorKind`/`exit_code` (1–5; ≥6 reservado); JSON erro em stdout + exit≠0 (T-01); human erro em stderr; flags `--json`/`--no-color` + `NO_COLOR`; `redact` sem crate regex; `uuid` 1.16.0; `anstream` 0.6.18; `tracing` 0.1.44 + `tracing-subscriber` env-filter; `InvalidArgument`→`InvalidInput`; envelope sem `schema_version`; docs em [`cli-output-and-errors.md`](compatibility/cli-output-and-errors.md).
- **DEC-006:** microplano 005 — `ProjectRoot`/`SafeRelativePath`; mensagem escape canónica; symlink/junction deny-if-outside; backups `.dare/backups/<utc>-<sha8>/…`; `atomic_write`; `FileLock` via **fs4 1.1.0** (`try_lock`, não 0.12.1 — versão inexistente no crates.io); `camino 1.1.9`, `tempfile 3.20.0`, `sha2 0.10.9`; docs em [`path-safety.md`](compatibility/path-safety.md).
- **DEC-007:** microplano 006 — `SafeCommand` argv-only; denylist `SECRET|TOKEN|KEY|PASSWORD`; truncate 4000 chars; timeout→`ProcessOutput` 124; cancel→`-1`; **`std::process`** (Classe B vs Mestre `tokio::process`); `kill_tree` **0.2.4**; mock runner; docs em [`process-safety.md`](compatibility/process-safety.md).
- **DEC-008:** microplano 007 — flatten nested + `extra` maps; **`yaml_serde` 0.10.4** as `serde_yaml`; cap 2 MiB; JSON canónico; YAML igualdade semântica; `CONTRACTS_SCHEMA_VERSION=0.1.0-contracts`; sem garde (008); docs em [`persisted-contracts.md`](compatibility/persisted-contracts.md).
- **DEC-009:** microplano 008 — precedência **CLI > env `DARE_*` > ficheiro > default**; allowlist env; `enabled:false` skip deep; dry-run sem writes; apply com `backup`+atomic; `schemaVersion` só com `MigrateOptions.write_schema_version`; diagnóstico JSON Pointer; docs em [`config-and-migrations.md`](compatibility/config-and-migrations.md).
- **DEC-010:** microplano 009 — `assets/` + `manifest.yml` SHA-256; embed `rust-embed` 8.7.2; `verify_embedded_assets` / `materialize_to`; templates canónicos; `.claude/commands` não apagados (external); docs em [`assets-inventory.md`](compatibility/assets-inventory.md).
- **DEC-011:** microplano 010 — `capability-matrix.yml` com 49 capabilities Claude (ADR-007); validate + render; tipos em `dare-assets` até crate harness nos adapters; docs em [`capabilities-canonical.md`](compatibility/capabilities-canonical.md).
- **DEC-012:** microplano 011 — crate `dare-harness` adapter Claude (`detect`/`install`/`validate`); preserve unmanaged; CLI `dare harness claude`; docs em [`harness-claude.md`](compatibility/harness-claude.md).
