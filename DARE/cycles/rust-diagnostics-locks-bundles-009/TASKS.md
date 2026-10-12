# TASKS — Lock Diagnostics & Support Reports

| Task | Description | Status | Evidence |
|---|---|---|---|
| task-001 | Inspect canonical lock implementation | IMPLEMENTED / UNVERIFIED | `crates/dare-core/src/fs/lock.rs` uses fs4 try_lock and `<path>.darelock` |
| task-002 | Implement read-only lock diagnostics | IMPLEMENTED / UNVERIFIED | `2a6cb390` and `0161013e` |
| task-003 | Local support preview / allowlist | IMPLEMENTED / UNVERIFIED | `70fa0153`, `cdbdcff3` |
| task-004 | Local redacted support export | IMPLEMENTED / UNVERIFIED | `70fa0153`, `e5b06123` |
| task-005 | Negative and regression tests | IMPLEMENTED / UNVERIFIED | `2a6cb390`, `e5b06123` |
| task-006 | Build, test, lint and audit | PENDING | No confirmed CI workflow for latest commits |
| task-007 | Review and proof | PENDING | Quality gate and end-to-end checks required |

No tasks marked DONE before passing build, test, fmt, Clippy and applicable audit.
