# Legacy archive checklist — `@dewtech/dare-cli`

> **Microplano:** 056 · **Task:** mp056-005  
> **Date:** 2026-07-31  
> **Upstream:** Release **`v4.0.0`** live (stable)  
> **Rule:** each item `done` \| `n/a` + reason. No hard delete of npm history.

## Checklist

| Item | Status | Reason |
|------|--------|--------|
| Docs / README npm legacy banner | `done` | Root [`README.md`](../../README.md) carries the **Legacy npm banner** with pointers to [`install-rust.md`](install-rust.md) and [`npm-legacy-policy.md`](npm-legacy-policy.md). Stable notes and install doc also point legacy → Rust. |
| `npm deprecate` **or** `blocked:credentials` | `done` | Attempted `npm deprecate @dewtech/dare-cli@"*"` on 2026-07-31. `npm whoami` → **401 Unauthorized**; no `NPM_TOKEN` / `NODE_AUTH_TOKEN`. Recorded as **`blocked:credentials`** in [`npm-legacy-policy.md`](npm-legacy-policy.md). Re-run deprecate when maintainer credentials exist. |
| CI TS feature freeze | `done` | Product CI (`.github/workflows/ci.yml`) is Rust-only (`crates/**`, Cargo). No TypeScript feature jobs for the npm product line. Policy freeze: [`../release-candidate/typescript-freeze.md`](../release-candidate/typescript-freeze.md) superseded by [`npm-legacy-policy.md`](npm-legacy-policy.md) (`features: rejected`). Governance Node scripts remain (not TS product features). |
| Redirects docs → `install-rust.md` | `done` | Recommended path documented in [`install-rust.md`](install-rust.md); cross-links from [`npm-legacy-policy.md`](npm-legacy-policy.md), [`RELEASE-NOTES-stable.md`](RELEASE-NOTES-stable.md), [`../support/legacy-support-window.md`](../support/legacy-support-window.md), root README, and [`../compatibility/README.md`](../compatibility/README.md). |
| Branch / archive note | `n/a` | Single monorepo Rust rewrite (`dare-cli`); there is no separate TypeScript package repository or orphan branch to archive. Legacy line remains the published npm package under security-only policy — not a git archive of a second tree. |

## Explicit non-goals

- **No** hard delete of `@dewtech/dare-cli` from the npm registry.
- **No** packaging/Homebrew/WinGet edits in this task (mp056-004 ownership).

## Sign-off

| Field | Value |
|-------|-------|
| `operator` | mp056-005 worktree execution |
| `date` | `2026-07-31` |
| `open_items` | none (deprecate remains blocked until credentials; banner + policy document the block) |
