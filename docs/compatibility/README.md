# Pacote de compatibilidade — baseline 3.18.1

Mapa dos artefatos de paridade entre o CLI legado npm `@dewtech/dare-cli@3.18.1` e o rewrite nativo Rust.

## Ciclo 0 governance — release notes

Microplano **001** (governança, baseline e ADRs prioritárias) concluído em 2026-07-20.

### Checklist de fechamento (001)

| Critério | Status |
|----------|--------|
| Árvore `docs/` + ADRs 001, 002, 004, 006, 007 Accepted | ✅ |
| `scripts/governance/` + `verify-all.mjs` exit 0 | ✅ |
| Baseline 3.18.1 + hash SHA-256 verificável | ✅ |
| Pacote compatibility (RF-07–RF-11) | ✅ |
| Security gate RS-01…RS-07 | ✅ |
| Inventário fixtures Ciclo 0 (RF-12) | ✅ |
| CI `.github/workflows/governance-001.yml` + artifact `baseline-manifest.json` | ✅ |
| `cargo fmt` / `cargo clippy` / `cargo test` | ⏸️ deferido — [DEC-001](../DECISION-LOG.md) |

**Próximo microplano:** [`002-workspace-rust-e-toolchain.md`](../../DARE-RUST-MICRO-PLANOS/DARE-RUST-MICRO-PLANOS/002-workspace-rust-e-toolchain.md) — workspace Rust, toolchain e gates Cargo.

### RF-13 (SHOULD)

Épico GitHub Issues com subtarefas espelhando RF-01–RF-11: placeholder em [`../DECISION-LOG.md`](../DECISION-LOG.md) até Issues estar disponível no repositório.

## Índice

| Arquivo | Propósito | RF |
|---------|-----------|-----|
| [`baseline-manifest.json`](baseline-manifest.json) | Fonte canônica máquina-legível (hash SHA-256 do tarball) | RF-01 |
| [`baseline-3.18.1.md`](baseline-3.18.1.md) | Narrativa humana + comando de verificação | RF-01 |
| [`classification-matrix.md`](classification-matrix.md) | Classes A–D — itens CI-001..CI-014 | RF-07 |
| [`language-policy.md`](language-policy.md) | Regras fechadas de idioma (PT governança, EN Rust novo) | RF-08 |
| [`disk-and-json-policy.md`](disk-and-json-policy.md) | Disco, paths, ordenação e writers JSON/YAML | RF-09 |
| [`breaking-change-process.md`](breaking-change-process.md) | Máquina de estados e lista fechada de breaking types | RF-11 |
| [`fixtures-inventory.md`](fixtures-inventory.md) | Inventário de fixtures de regressão | RF-12 |
| [`scaffold-contracts.md`](scaffold-contracts.md) | Crate `dare-scaffold`: 11 stacks, 7 AX, plan/apply (046 / DEC-047) | — |
| [`cli-init-bootstrap.md`](cli-init-bootstrap.md) | CLI `dare init` / `dare bootstrap`: greenfield + idempotent scaffold (047 / DEC-048) | — |
| [`cli-hooks-steering.md`](cli-hooks-steering.md) | CLI `dare hooks` / `dare steering`: trust gate, allowlist, `.env*` deny (048 / DEC-049) | — |
| [`cli-verify-bench.md`](cli-verify-bench.md) | Advanced verify + CLI `dare bench`: Fix·Rate, aspects, execute flags (049 / DEC-050) | — |
| [`cli-ai.md`](cli-ai.md) | CLI `dare ai` doctor/providers/run/prompt: enrich providers, write opt-in (050 / DEC-051) | — |
| [`cli-dashboard-rest.md`](cli-dashboard-rest.md) | CLI `dare dashboard` / `dare server --protocol rest`: Axum shared app, auth, REST legado (051 / DEC-052) | — |
| [`cli-mcp.md`](cli-mcp.md) | CLI `dare mcp serve`: MCP real stdio/streamable-http (052 / DEC-053) | — |
| [`cli-self-update.md`](cli-self-update.md) | CLI `dare self` update/rollback/uninstall + packaging (053 / DEC-054) | — |
| [`parity-hardening.md`](parity-hardening.md) | Harness `dare-parity`: golden/security/xplat, N-01..N-08, gate 15% (054 / DEC-055) | — |
| [`parity-diff-log.md`](parity-diff-log.md) | Diffs classificados TS↔Rust (`PD-*`) para golden Class C (054 / DEC-055) | — |
| [`../release-candidate/typescript-freeze.md`](../release-candidate/typescript-freeze.md) | Freeze TS `@dewtech/dare-cli`: security fixes only from RC `v4.0.0-rc1` (055) | — |
| [`../release-candidate/contract-freeze.md`](../release-candidate/contract-freeze.md) | Freeze contrato Classe A no RC: ADR Accepted + matrix + DECISION-LOG (055) | — |

## Relacionamentos

- Itens **Classe C** na matriz referenciam ADRs em `docs/adr/` via coluna `adr_ref`.
- Waivers e deferrals de escopo ficam em [`../DECISION-LOG.md`](../DECISION-LOG.md) (ex.: DEC-001 → microplano 002).
- Processo de breaking change exige matriz + ADR Accepted + checklist de PR antes do merge.

## Security gate 001

Gate de segurança do Ciclo 0 (microplano 001, Fase 6). Cada requisito RS-* do [`DARE/DESIGN.md`](../../DARE/DESIGN.md) mapeia para artefatos verificáveis no repositório.

| RS | Controle | Artefato / verificação |
|----|----------|------------------------|
| RS-01 | Validação de entradas futuras (paths, JSON/YAML) | [`disk-and-json-policy.md`](disk-and-json-policy.md), [`ADR-002`](../adr/ADR-002-contrato-saida-json.md), [`ADR-007`](../adr/ADR-007-formato-canonico-capabilities.md) |
| RS-02 | Proibir secrets em docs e fixtures | `scripts/governance/verify-no-secrets.mjs` (`NO_SECRETS`), integrado em `verify-all.mjs`; scan reutiliza `scanForSecrets` de `verify-baseline.mjs` |
| RS-03 | Breaking changes só com ADR Accepted + owners | [`breaking-change-process.md`](breaking-change-process.md), [`classification-matrix.md`](classification-matrix.md) |
| RS-04 | Dependências sem CVE HIGH/CRITICAL | `scripts/governance/package.json` + `package-lock.json`; gate Ralph: `npm audit --audit-level=high` |
| RS-05 | Secrets via env — exemplos só com nomes de variáveis | [`.env.governance.example`](../../.env.governance.example) |
| RS-06 | Invariantes path safety, argv separado, redação de secrets | [`ADR-001`](../adr/ADR-001-compatibilidade-bugs-legados.md) § Decisão |
| RS-07 | Classe D = must_fix (sem paridade com bugs de segurança) | [`classification-matrix.md`](classification-matrix.md), [`ADR-001`](../adr/ADR-001-compatibilidade-bugs-legados.md) |

Comando agregado (exit 0 = gate OK):

```bash
node scripts/governance/verify-all.mjs
```

Ordem interna: `verify-structure` → `verify-no-secrets` (manifesto + ADRs) → `verify-adr-frontmatter` → `verify-baseline` (hash tarball 3.18.1).
