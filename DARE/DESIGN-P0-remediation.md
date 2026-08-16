# DARE Design — P0 Remediation

## Context

The native Rust DARE CLI is the official implementation, but review of the greenfield scaffolding and autonomous-agent paths identified behavioral gaps that violate DARE's core principle: evidence must be stronger than an agent or subsystem claim.

This remediation intentionally starts with safety and execution correctness before redesigning scaffolding.

## Problem statement

Three P0 behaviors can invalidate user work or autonomous execution:

1. `dare init --force` may remove a target directory that existed before the command when a later init step fails.
2. The Codex agent driver defaults to a read-only sandbox even though the agent is launched inside an isolated git worktree and is expected to implement code.
3. Agent changes are made in an isolated worktree, but the worktree is removed before a successful candidate is promoted to the project working tree. Uncommitted changes can therefore be discarded and Ralph can run against the unchanged root.

A fourth P1 behavior is recorded but deliberately deferred from this slice: scaffold validation currently proves structural presence of DARE/AX files, not that generated applications compile or run.

## Goals

- Never delete a pre-existing target directory as rollback behavior.
- Allow a coding agent to write only inside its isolated candidate worktree.
- Preserve a successful candidate until its patch has been verified and promoted.
- Run Ralph/review against promoted code, not an unchanged root.
- Add tests that assert filesystem/git effects rather than only exit/status values.
- Keep cleanup deterministic on success, failure, timeout, and cancellation.

## Non-goals

- Full scaffold v2 redesign.
- Best-of-N redesign beyond preventing loss of candidate work.
- New providers or LLM routing.
- Changing persisted DARE schemas unless strictly required.

## Requirements

### R-P0-001 — Safe init rollback

If the init target did not exist before `dare init`, a failed init may remove the newly-created target.

If the target already existed and `--force` was supplied, a failed init MUST NOT remove the target directory or unrelated pre-existing files.

### R-P0-002 — Writable isolated Codex execution

The default Codex execution mode for `dare execute --agent --driver codex` MUST permit writes inside the worktree used as `cwd` while retaining non-interactive approval behavior.

The repository root outside that worktree MUST NOT be used as the agent write target.

### R-P0-003 — Candidate preservation and promotion

A successful agent attempt MUST retain its worktree until DARE has obtained a patch/diff and promoted that patch to the root working tree.

A failed/timeout/cancelled attempt MUST NOT be promoted.

Worktree cleanup MUST happen after the promotion decision.

### R-P0-004 — Evidence-based completion

After promotion, normal Ralph/complete gates MUST run against the promoted root contents. A successful agent process exit alone is not sufficient evidence of implementation correctness.

### R-P0-005 — Behavioral tests

Tests MUST cover at least:

- existing sentinel file survives `init --force` failure;
- newly-created init target is removed on failure;
- default Codex argv contains a writable workspace sandbox;
- a successful fake agent edit appears in the root after promotion;
- a failed fake agent edit does not appear in the root;
- worktrees are cleaned after terminal outcomes.

## Acceptance criteria

This slice is DONE only when the relevant Rust tests pass and the assertions observe final filesystem/git state, not only JSON/status output.

## Follow-up P1

After P0 closes, replace generic static scaffolding with per-stack bootstrap strategies and semantic validation (`build/test/smoke`) for Rails, NestJS, React/Vue, Rust/Axum, Go, FastAPI, Laravel, and MCP stacks.
