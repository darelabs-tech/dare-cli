# DESIGN — Rust Reliability & Security Hardening

Cycle: `rust-reliability-hardening-001`
Status: APPROVED FOR EXECUTION; merge requires separate approval.

## Goal
Harden `darelabs-tech/dare-cli` Rust CLI for deterministic DAG execution, durable state, safe worktrees, path confinement, process execution, archive handling, release authenticity and cross-platform reliability. The Node implementation is a behavioral reference, not evidence of parity.

## Acceptance requirements
1. Pin the Rust and Node baseline commits, inventory crates and workflows, and record reproducible build/test/lint/audit evidence.
2. Detect invalid DAG specs, missing dependencies, cycles and unsafe paths before executing work.
3. Reject illegal state transitions and prevent false DONE without task-specific Ralph verification.
4. Exercise interrupted executions, retries, concurrency and recovery with deterministic tests.
5. Ensure cleanup cannot remove arbitrary directories or escape the designated worktree jail.
6. Test process invocation, secret redaction, archive traversal and release integrity against negative fixtures.
7. Compare Rust/Node contracts with reproducible fixtures; report differences rather than assuming parity.
8. Run a 40-task DAG failure-injection scenario and record results.
9. Build a CI/release support matrix for Linux, Windows and macOS, distinguishing executed from unverified targets.
10. Document rollback, release readiness, unresolved risks and a reliability scorecard.

## Gates
- No task is DONE without recorded applicable build, tests, lint and audit results.
- Do not weaken existing tests or security controls to obtain green CI.
- Preserve the existing PR #2 and branch. No merge without explicit authorization.
- Full original task details are in the approved artifact bundle; this summary does not supersede task specs.
