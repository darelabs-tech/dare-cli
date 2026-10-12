# DESIGN — Rust CLI Operations & Diagnostics

Cycle: rust-v4-operations-diagnostics-008
Status: proposed; implementation and verification pending.

## Objective
Make Rust CLI execution failures actionable for developers and operators with local-first diagnostics, safe support bundles and deterministic recovery guidance.

## Scope
1. Inventory existing doctor, status, logging, execution events and troubleshooting commands to avoid duplicate functionality.
2. Provide a local diagnostics workflow that inspects workspace configuration, DAG validity, state-store health, execution locks, toolchain and release version.
3. Emit structured, actionable diagnostics with stable codes, severity, remediation and machine-readable JSON output; preserve existing output contracts.
4. Produce opt-in support bundles with aggressive secret/path/PII redaction, explicit preview and no automatic upload.
5. Test corrupted state, stale locks, missing tools, malformed DAG and recovery advice, including negative redaction tests.
6. Document operational playbooks for failed tasks, interrupted runs and version incompatibilities.

## Acceptance
- Representative failure fixtures produce deterministic codes and actionable remediation.
- Redaction tests demonstrate that credentials and sensitive environment values do not leak.
- No network transmission of diagnostics without explicit user action.
- Build, tests, formatting, Clippy and applicable audit gates pass with actual CI evidence.
- Merge requires separate approval.
