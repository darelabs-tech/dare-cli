# DESIGN — Rust Supply Chain & Security Hardening

Cycle: rust-supply-chain-security-003
Status: proposed, pending implementation evidence

## Objective
Strengthen release authenticity, dependency provenance and security regression coverage for the Rust CLI without changing its public contracts.

## Scope
1. Inventory existing Rust CI, release workflows, signing and dependency-audit mechanisms before modifications.
2. Enforce artifact checksums and verify signatures where signing is configured. Never claim signing is enabled unless a keyless or managed signing workflow exists.
3. Validate release asset names, archive extraction confinement and path traversal with negative tests.
4. Add regression coverage for unsafe worktree paths, subprocess argument boundaries and secret redaction.
5. Run cargo audit or equivalent advisory scanning and document any exceptions with ownership and expiry.
6. Publish a reproducible release verification procedure and platform matrix based on executed checks.

## Acceptance gates
- Existing Rust tests, fmt, Clippy and release workflows pass.
- All new security regression tests pass, including negative cases.
- Every artifact has a reproducible digest; signatures are verified if present.
- No high/critical unmitigated advisory in release dependencies.
- Evidence references actual workflow run, commit SHA and produced assets.
- Merge requires separate human approval.

## Non-goals
- No credentials or signing secrets committed.
- No fabricated SBOM, attestations or platform test results.
- No changes to Node/Rust CLI semantics unless a verified vulnerability requires it.
