# DESIGN — Node/Rust CLI Compatibility

Cycle: rust-node-parity-004
Status: proposed; implementation pending.

## Goal
Establish measured, regression-tested compatibility between the legacy Node CLI and the Rust CLI without asserting equivalence where implementations differ.

## Scope
- Inventory command names, flags, exit codes, stdout/stderr, JSON schemas and side effects in both implementations.
- Build deterministic, shared fixtures for project initialization, DAG validation, execution state, task transitions, and failure cases.
- Run differential tests with normalized nondeterministic fields (timestamps, paths and platform-specific output).
- Classify differences as intentional, bugs or unsupported and record migration guidance.
- Preserve documented Node contracts unless an explicit breaking change is approved.

## Acceptance
- Parity matrix references exact Node and Rust baseline revisions.
- Automated tests cover the supported shared command surface, including negative cases.
- Every deviation has an owner, rationale and disposition.
- Rust build, tests, formatting, Clippy and audit pass with pinned CI evidence.
- Merge requires explicit separate authorization.
