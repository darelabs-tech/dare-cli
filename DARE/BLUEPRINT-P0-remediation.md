# DARE Blueprint — P0 Remediation

## Architecture decision

Treat agent execution as a transactional candidate pipeline:

```text
create worktree
  -> run agent
  -> inspect result
  -> if success: capture candidate diff
  -> promote candidate diff to root
  -> run Ralph/complete on root
  -> cleanup worktree/branch
```

No terminal agent status may imply that code has reached the root.

## Component changes

### 1. `crates/dare-cli/src/commands/init.rs`

Track whether the target existed before `fs::create_dir_all`.

Rollback policy:

```text
preexisting=false + pipeline failure -> remove newly-created target
preexisting=true  + pipeline failure -> preserve target
```

Add a regression test with a sentinel file and an invalid stack while `force=true`.

### 2. `crates/dare-agent/src/drivers/codex.rs`

Change the default Codex sandbox from read-only to a workspace-writable mode because `AgentRequest.cwd` is already the isolated worktree.

Add/strengthen an argv test so this cannot silently regress.

### 3. `crates/dare-agent/src/worktree.rs`

Extend the worktree abstraction with explicit lifecycle operations rather than using `remove()` as the only terminal action.

Target API shape:

```rust
capture_patch(spec) -> CandidatePatch
promote_patch(patch) -> PromotionReport
discard(spec)
cleanup(spec)
```

The implementation must handle both:

- committed changes on the candidate branch relative to the root base;
- staged/unstaged changes still present in the candidate worktree.

Patch application should be binary-safe and fail closed on conflict.

### 4. `crates/dare-cli/src/commands/execute_agent.rs`

Move worktree cleanup after result interpretation.

Pseudo-flow:

```rust
let spec = wt_mgr.create(...)?;
let result = driver.run(...)?;

match result.status {
    Success => {
        let patch = wt_mgr.capture_patch(&spec)?;
        wt_mgr.promote_patch(&spec, &patch)?;
        wt_mgr.cleanup(&spec)?;
        // only now allow Done -> Ralph
    }
    Failure | Timeout | Cancelled => {
        wt_mgr.discard(&spec)?;
    }
}
```

If promotion fails, the task must not be reported as Done and the candidate should remain recoverable or be journaled for diagnosis.

### 5. Tests

Prefer real-git integration tests for worktree/promotion semantics. Mock argv tests remain useful but are insufficient for acceptance.

Required test fixtures:

- root repository with initial commit;
- candidate modifies tracked file without commit;
- candidate optionally commits and then modifies another file;
- promotion reproduces the candidate state in root;
- failed candidate is discarded;
- cleanup leaves no registered worktree.

## Invariants

- Root is never modified by a failed candidate.
- Successful candidate code reaches root before Ralph starts.
- Worktree cleanup never precedes promotion on Success.
- Init rollback owns only resources created by that init invocation.
- A provider's ability to write is constrained by the candidate worktree boundary.

## Verification gates

Run at minimum:

```bash
cargo test -p dare-agent
cargo test -p dare-cli -- init
cargo test -p dare-cli -- execute_agent
cargo clippy -p dare-agent -p dare-cli --all-targets -- -D warnings
```

For the worktree promotion integration test, use real `git` and assert final file contents and `git worktree list` state.

## P1 next slice — Scaffold v2

Once P0 is green:

1. introduce per-stack bootstrap strategy rather than a universal `GenericScaffolder`;
2. use official generators where they define framework runtime (`rails new`, Nest CLI, Vite, Composer/Laravel);
3. keep DARE templates as overlays/value-add;
4. replace structural validation with semantic `install/build/test/smoke` contracts;
5. add behavioral parity suites against the TypeScript implementation where useful.
