# npm legacy policy — `@dewtech/dare-cli`

> **Microplano:** 056  
> **Supersedes:** [`../release-candidate/typescript-freeze.md`](../release-candidate/typescript-freeze.md) (RC security-only freeze)

## Machine fields

| Field | Value |
|-------|-------|
| `status` | `legacy` |
| `package` | `@dewtech/dare-cli` |
| `last_supported_npm` | `3.18.1` |
| `security_fixes_until` | `2026-10-29` |
| `features` | `rejected` |
| `recommended` | Rust native CLI `v4.0.0` |

## Policy

The TypeScript / npm package **`@dewtech/dare-cli`** is **legacy**.

| Rule | Detail |
|------|--------|
| **Status** | `legacy` — not the recommended install path |
| **Last supported npm version** | **`3.18.1`** (compatibility baseline) |
| **Security fixes until** | **`2026-10-29`** (ISO-8601; aligned with support window end — see [`../support/legacy-support-window.md`](../support/legacy-support-window.md)) |
| **Features** | **`rejected`** — feature PRs, refactors, non-security bugfixes, and capability bumps on the TS line are rejected |
| **Allowed** | Security fixes only (CVE HIGH/CRITICAL, Class D / RS-07 must_fix) through `security_fixes_until` |
| **Recommended** | Migrate to the native Rust CLI **`v4.0.0`** — [`install-rust.md`](install-rust.md) |

## Rationale

- Product surface for new work is the Rust rewrite tagged **`v4.0.0`**.
- npm **`3.18.1`** remains the frozen baseline for Class A/B parity references.
- Keeps the legacy line predictable during the post-cutover security-only window.

## Review gate

Maintainers **MUST** reject PRs that change the TS legacy tree for non-security reasons while this policy is in force. Security PRs SHOULD still follow supply-chain and audit gates (`npm audit --audit-level=high` where applicable).

## Related

- [`install-rust.md`](install-rust.md) — recommended install
- [`RELEASE-NOTES-stable.md`](RELEASE-NOTES-stable.md) — stable notes + legacy pointer
- [`../support/legacy-support-window.md`](../support/legacy-support-window.md) — window dates and scope
- [`../compatibility/breaking-change-process.md`](../compatibility/breaking-change-process.md) — breaking-change machine
