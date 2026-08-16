# DARE Tasks — P0 Remediation

## task-p0-001 — Safe init rollback

**Requirement:** R-P0-001

- Track whether init target pre-existed.
- Remove target on failure only when this invocation created it.
- Add sentinel regression test for `--force` on existing target.
- Preserve existing test proving newly-created failed target is removed.

**DONE when:** both rollback ownership tests pass.

## task-p0-002 — Writable Codex candidate

**Requirement:** R-P0-002

- Change default Codex sandbox from read-only to workspace-write.
- Add argv-level regression test.
- Keep non-interactive approval mode.

**DONE when:** default command is writable only in the supplied candidate cwd and tests lock the argv contract.

## task-p0-003 — Candidate patch lifecycle

**Requirement:** R-P0-003

- Add explicit worktree patch capture.
- Support committed plus staged/unstaged candidate changes.
- Add binary-safe patch promotion to root.
- Separate promotion from discard/cleanup.
- Fail closed on patch conflict.

**DONE when:** real-git integration test proves candidate state reaches root and failed candidates do not.

## task-p0-004 — Reorder autonomous execution

**Requirements:** R-P0-003, R-P0-004

- Do not remove worktree immediately after `driver.run`.
- On Success: capture/promote, then cleanup, then allow Done/Ralph.
- On Failure/Timeout/Cancelled: discard without promotion.
- Ensure promotion failure cannot report Done.

**DONE when:** execute-agent integration tests prove root content before completion gates.

## task-p0-005 — P0 verification

**Requirement:** R-P0-005

Run:

```bash
cargo test -p dare-agent
cargo test -p dare-cli -- init
cargo test -p dare-cli -- execute_agent
cargo clippy -p dare-agent -p dare-cli --all-targets -- -D warnings
```

Record any intentionally deferred failures separately; no P0 behavior may be waived.

## P1 backlog

- Scaffold v2 strategy trait.
- Rails official `rails new` bootstrap + DARE overlay.
- Nest CLI bootstrap + overlay.
- Vite React/Vue bootstrap + overlay.
- Semantic scaffold verification (`build/test/smoke`).
- Real best-of-N candidate generation and Pareto selection.
- Behavioral parity tests rather than file-presence parity.
