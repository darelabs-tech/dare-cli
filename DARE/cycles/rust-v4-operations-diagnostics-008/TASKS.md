# TASKS — Operational Diagnostics

| ID | Task | Status | Evidence |
|---|---|---|---|
| 001 | Inventory existing diagnostics and design integration | IMPLEMENTED, UNVERIFIED | BLUEPRINT.md |
| 002 | Add stable JSON/config/state/DAG diagnostics | IMPLEMENTED, UNVERIFIED | `40dc792` |
| 003 | Add negative redaction and valid-state tests | IMPLEMENTED, UNVERIFIED | `40dc792` |
| 004 | Inspect state locks and stale-lock behavior | PENDING | Requires canonical lock implementation inspection |
| 005 | Design and implement safe opt-in support bundle | PENDING | Requires redaction and preview contract |
| 006 | Run build, tests, fmt, Clippy and audit | PENDING | CI evidence not yet confirmed |
| 007 | Write operational playbook and proof | PENDING | After validation |

No task is DONE until build, test, lint and applicable audit evidence passes.

## Dependency graph

`task-001 -> task-002 -> task-003`; `task-001 -> task-004`; `task-002 + task-004 -> task-005`; `task-003 + task-004 + task-005 -> task-006 -> task-007`.

See `dare-dag.yaml`, `EXECUTION/STATUS.md`, `REGRESSION.md` and `PROOF.md` for the complete cycle. No task is marked DONE without CI verification.
