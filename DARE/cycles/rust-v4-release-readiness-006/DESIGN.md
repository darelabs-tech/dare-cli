# DESIGN — Rust CLI v4 Release Readiness

Cycle: rust-v4-release-readiness-006
Status: proposed; no implementation or validation evidence yet.

## Baseline
Workspace Cargo.toml on main declares version 4.0.0 and Rust MSRV 1.88.0. Prior PRs introduced reliability, supply-chain, parity and cross-platform release cycle documentation; their presence alone does not prove runtime acceptance.

## Objective
Establish an executable end-to-end release acceptance gate for the DARE CLI v4, covering first-run onboarding, real DAG execution, failure recovery and packaged binary verification.

## Scope
1. Inventory actual CI workflows, release artifacts and CLI command surface before changing implementation.
2. Create a reproducible clean-project smoke journey: install/version/help, project initialization, DAG validation, task execution, failure/retry, resume and status inspection.
3. Execute the same journey using packaged release binaries for supported platforms, not only cargo test binaries.
4. Verify checksum, archive contents, executable version and platform compatibility of published artifacts.
5. Publish evidence with exact commit, workflow run, artifact digest, target triple, test results and known limitations.
6. Document rollback and migration from the preceding CLI release.

## Acceptance
- A documented, automated end-to-end scenario passes in CI.
- Every supported release target has pinned artifact and smoke evidence; unsupported targets are labeled.
- Build, test, fmt, Clippy and dependency audit are green.
- No unsupported claims of production readiness, parity or signature provenance.
- Human approval required for merge.
