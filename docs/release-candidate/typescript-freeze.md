# TypeScript freeze (RC)

> **Microplano 055** · Fase D · RC tag: **`v4.0.0-rc1`** (major **4** after npm baseline **3.18.1**)

## Policy

From the RC tag date forward, the TypeScript package `@dewtech/dare-cli` is under freeze:

> From RC tag date forward, `@dewtech/dare-cli` TypeScript line accepts **security fixes only**. Feature PRs to TS legacy are rejected until after microplano 056 policy supersedes this freeze.

| Rule | Detail |
|------|--------|
| **Effective from** | Date of Git tag / GitHub Release **`v4.0.0-rc1`** (prerelease) |
| **Package** | `@dewtech/dare-cli` (TypeScript / npm legacy line) |
| **Allowed** | Security fixes only (CVE HIGH/CRITICAL, Class D / RS-07 must_fix) |
| **Rejected** | Feature work, refactors, non-security bugfixes, capability bumps on the TS line |
| **Supersedes when** | Microplano **056** cutover / stable policy replaces this freeze |

## Rationale

- Product surface for new work is the Rust rewrite; TS remains a compatibility / install path until 056.
- RC **`v4.0.0-rc1`** starts the major-4 prerelease track after baseline npm **3.18.1**.
- Keeps the legacy line predictable for pilots during shadow testing.

## Review gate

Maintainers **MUST** reject PRs that change the TS legacy tree for non-security reasons while this document is in force. Security PRs SHOULD still follow supply-chain and audit gates (`npm audit --audit-level=high` where applicable).

## Related

- [`contract-freeze.md`](contract-freeze.md) — Classe A contract freeze during RC
- [`../compatibility/breaking-change-process.md`](../compatibility/breaking-change-process.md) — breaking-change machine
- [`../compatibility/README.md`](../compatibility/README.md) — compatibility package index
