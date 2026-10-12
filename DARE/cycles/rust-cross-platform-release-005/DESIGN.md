# DESIGN — Rust Cross-Platform Release Automation

Cycle: rust-cross-platform-release-005
Status: proposed; implementation and validation pending.

## Goal
Build and verify reproducible Rust CLI release artifacts across supported Linux, Windows and macOS targets.

## Scope
- Inventory existing GitHub Actions release and CI workflows, Rust toolchain, targets and artifact naming.
- Add or improve release build matrix for Linux x86_64/aarch64, Windows x86_64, macOS x86_64/aarch64 where runners and cross-compilation support exist.
- Execute target-appropriate CLI smoke tests, including version/help, invalid input, and exit codes.
- Generate SHA-256 checksums and verify downloads before distribution.
- Validate archive contents and executable permissions where relevant.
- Document unsupported or untested targets and rollback procedure.

## Acceptance
- Each advertised platform has a successful, pinned workflow run and smoke evidence.
- Every published binary has a verifiable checksum.
- Artifact naming, packaging and version match the release tag.
- Rust format, Clippy, tests and dependency audit gates remain enforced.
- Do not claim reproducibility or signed provenance unless demonstrated.
- Merge requires separate explicit authorization.
