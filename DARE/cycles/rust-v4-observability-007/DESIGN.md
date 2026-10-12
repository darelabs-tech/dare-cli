# DESIGN — Rust CLI Execution Observability

Cycle: rust-v4-observability-007
Status: proposed; implementation pending.

## Objective
Make DAG execution, retries, recovery and verification diagnosable without leaking secrets or changing stable CLI contracts.

## Scope
- Inventory current structured logging, execution records, tracing and CLI output.
- Introduce stable correlation identifiers for cycle, run, task and attempt, reusing existing identifiers where possible.
- Record lifecycle events for claim, start, retry, cancellation, recovery, verification and completion.
- Define machine-readable event schema with deterministic fields and explicit schema version.
- Redact secrets, credentials, sensitive file contents and environment values from logs and errors.
- Add integration tests for event ordering, cross-run correlation, failure visibility and secret redaction.
- Document opt-in verbosity, retention and troubleshooting procedures.

## Acceptance
- Existing CLI output remains compatible by default.
- Event schema and log-redaction tests pass on representative failure paths.
- Build, tests, fmt, Clippy and applicable audit checks pass with pinned CI evidence.
- No telemetry leaves the host without explicit configuration and consent.
- No merge without separate human authorization.
