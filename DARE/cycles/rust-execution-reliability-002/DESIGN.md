# Rust Execution Reliability — DESIGN

Status: proposed; implementation pending baseline verification.

## Objective
Improve durable DAG task execution, crash recovery, concurrent task claims, idempotent transitions, and failure-injection coverage in the Rust CLI without breaking existing Node contracts.

## Acceptance
- Deterministic tests prove a task cannot be claimed by two concurrent workers.
- Restart after interruption preserves completed tasks and safely recovers interrupted ones.
- Repeated state transitions are either idempotent or rejected with explicit errors.
- Task completion requires task-specific verification evidence.
- A deterministic 40-task fixture exercises failure, retry, cancellation and resume.
- Existing Rust CLI behavior remains compatible; any Node parity deviations are recorded.
- Cargo build, test, fmt, Clippy and dependency audit evidence is attached before DONE.

## Constraints
- Preserve current public CLI contracts unless a change is explicitly reviewed.
- Reuse existing state and DAG implementations; avoid duplicate engines.
- No merge without separate approval.
