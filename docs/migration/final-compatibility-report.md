# Final compatibility report — stable cutover `v4.0.0`

> **Microplano:** 056 · **Task:** mp056-005  
> **Date:** 2026-07-31  
> **Product:** native Rust CLI is **official**  
> **Baseline npm:** `@dewtech/dare-cli@3.18.1`  
> **Sources:** [`../compatibility/parity-diff-log.md`](../compatibility/parity-diff-log.md) · [`../compatibility/classification-matrix.md`](../compatibility/classification-matrix.md) · [`../pilot/incidents.md`](../pilot/incidents.md) · ADRs under [`../adr/`](../adr/) · publish/smoke notes

## Executive summary

Stable **`v4.0.0`** is the official DARE CLI. The TypeScript / npm line is **legacy** ([`npm-legacy-policy.md`](npm-legacy-policy.md)). Parity work from microplanos **054** (harness / golden) and **055** (pilots / RC) is aggregated below. Every residual gap has an explicit `compat_class` (**A** \| **B** \| **C** \| **D**).

| Machine field | Value |
|---------------|-------|
| `unclassified_count` | `0` |
| `official_product` | Rust native CLI `v4.0.0` |
| `npm_status` | `legacy` |
| `parity_diff_log_new_class_c` | `none` (no PD-* row added in mp056-005) |

**Rust is official.** New installs MUST follow [`install-rust.md`](install-rust.md).

## Residual gaps (classified)

| id | compat_class | adr_ref | status | summary |
|----|--------------|---------|--------|---------|
| CI-001 | A | — | preserve | Public exit codes (baseline contract) |
| CI-002 | A | — | preserve | Public command / flag names |
| CI-003 | A | — | preserve | Persisted schemas (`dare.config.json`, state, DAG) |
| CI-004 | A | — | preserve | Canonical IDs |
| CI-005 | B | — | fixed / accepted in Rust | Welcome `dare new` text vs legacy |
| CI-006 | B | — | fixed / accepted in Rust | Mojibake / inconsistent formatting |
| CI-007 | C | ADR-001 | accepted | Skill update/remove incomplete vs legacy |
| CI-008 | C | ADR-002 | accepted | JSON / ordering differences |
| CI-009 | C | language-policy / ADR-003 | accepted | Mixed PT/EN surface |
| CI-010 | D | — | must_fix (Rust) | Path escape / symlink abuse |
| CI-011 | D | — | must_fix (Rust) | Unsafe shell concatenation |
| CI-012 | D | — | must_fix (Rust) | Secret leakage in logs/errors |
| CI-013 | D | — | must_fix (Rust) | Unsafe archive extraction (zip-slip) |
| CI-014 | D | — | must_fix (Rust) | Missing/invalid signatures on releases/skills |
| PD-001 | C | ADR-pending / DEC-024–025 | accepted native SoT | Design LLM wording variance TS↔Rust — do not over-normalize |
| INC-001 | B | — | mitigated | Pilot fixture density &lt;3 files; shadow uses `tests/fixtures/monorepo` |
| INC-002 | B | — | mitigated | Pilot smoke used exit-0 stub binary (not product P0/P1) |
| INC-003 | B | — | closed | Synthetic macOS/Linux pilots on Windows host; OS coverage via CI |
| GAP-x86_64-apple-darwin | B | — | open (owner: Tech Lead DARE CLI) | Intel macOS archive missing from Release `v4.0.0` (macos-13 queue stall). Packaging/CI gap — **not** Class C parity intent. Use aarch64 asset or build from source. Tracked in [`publish-stable-checklist.md`](publish-stable-checklist.md) / [`RELEASE-NOTES-stable.md`](RELEASE-NOTES-stable.md). |
| GAP-cosign-soft-fail | B | — | known | `SHA256SUMS.sig` may contain soft-fail text; `dare self update` fail-closed (exit 6). Prefer installers when applicable. |

### Classification notes

- **`GAP-x86_64-apple-darwin`** is **Class B** (fix without ADR): publish/CI asset gap, not an intentional behavioral divergence requiring a new `PD-*` Class C row. [`parity-diff-log.md`](../compatibility/parity-diff-log.md) unchanged in mp056-005.
- All matrix items CI-001..CI-014 already carry classes in [`classification-matrix.md`](../compatibility/classification-matrix.md).
- Pilot incidents carry `compat_class` in [`../pilot/incidents.md`](../pilot/incidents.md); no new Class C gaps from 055.

## `unclassified_count`

```text
unclassified_count == 0
```

Every row in the residual-gaps table above has `compat_class` ∈ {A, B, C, D}. No open gap lacks a class.

## Links — baselines 054 / 055

| Microplano | Artifact | Role |
|------------|----------|------|
| 054 | [`../compatibility/parity-hardening.md`](../compatibility/parity-hardening.md) | Golden / security / xplat harness |
| 054 | [`../compatibility/parity-diff-log.md`](../compatibility/parity-diff-log.md) | Class C `PD-*` index |
| 054 | [`../perf/baseline-054.md`](../perf/baseline-054.md) | Perf baseline + regression gate |
| 055 | [`../pilot/incidents.md`](../pilot/incidents.md) | Pilot / shadow incidents |
| 055 | [`../release-candidate/RELEASE-NOTES.md`](../release-candidate/RELEASE-NOTES.md) | RC notes (`v4.0.0-rc1`) |
| 055 | [`../release-candidate/typescript-freeze.md`](../release-candidate/typescript-freeze.md) | RC TS freeze (superseded by 056 legacy policy) |
| 056 | [`npm-legacy-policy.md`](npm-legacy-policy.md) | npm `legacy` + deprecate / `blocked:credentials` |
| 056 | [`legacy-archive-checklist.md`](legacy-archive-checklist.md) | Archive operationalization |
| 056 | [`RELEASE-NOTES-stable.md`](RELEASE-NOTES-stable.md) | Stable notes |

## Related ADRs

| ADR | Title | Role in cutover |
|-----|-------|-----------------|
| [ADR-001](../adr/ADR-001-compatibilidade-bugs-legados.md) | Compatibilidade de bugs legados | Classes A–D + CI-007 |
| [ADR-002](../adr/ADR-002-contrato-saida-json.md) | Contrato de saída JSON | CI-008 |
| [ADR-004](../adr/ADR-004-rest-compativel-e-mcp-real.md) | REST compatível e MCP real | Transport surface |
| [ADR-006](../adr/ADR-006-compatibilidade-migracao-graph-db.md) | Migração graph DB | Graph parity |
| [ADR-007](../adr/ADR-007-formato-canonico-capabilities.md) | Formato canônico capabilities | Capability contract |
| [ADR-008](../adr/ADR-008-release-alpha-nativo.md) | Release nativo (assets / sums / sig) | Stable publish asset rules |

## Close statement

With **`unclassified_count == 0`**, legacy policy operationalized (deprecate **`blocked:credentials`** + README banner), and archive checklist complete, the compatibility close for microplano **056** Fase E (mp056-005) is satisfied. **Rust is the official CLI.**
