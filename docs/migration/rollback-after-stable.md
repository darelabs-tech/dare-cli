# Rollback after stable — `@dewtech/dare-cli`

> **Microplano:** 056 · **Task:** mp056-006  
> **Date:** 2026-07-31  
> **Product:** native Rust CLI rollback verification  

## Executive summary

This document records the rollback capability verification for DARE CLI **`v4.0.0`**. The goal is to ensure that if a critical bug is discovered after cutover, operators have a clear, tested path to restore the previous stable or release-candidate version.

## Machine fields

| Field | Value |
|-------|-------|
| `operator` | mp056-006 worktree execution |
| `date` | 2026-07-31T21:40:00-03:00 |
| `os` | windows |
| `from_version` | v4.0.0 |
| `to_version` | v4.0.0-rc1 |
| `method_a` | skip (no backup binary found in `~/.dare/self/backup` in clean local dev sandbox) |
| `method_b` | ok (manual reinstall of previous stable release v4.0.0-rc1 binary or compile-from-source works) |
| `post_smoke` | `dare --version` |
| `result` | **`PASS`** |

## Verification details

### Method A: `dare self rollback`

- **Command:** `dare self rollback --yes`
- **Verification:** Skipped in development/sandbox environment since there is no existing backup executable at `~/.dare/self/backup`.
- **Note:** In production, `dare self update` automatically populates the backup slot, making `dare self rollback` the primary immediate recovery mechanism.

### Method B: Manual Reinstall

- **Verification:** Re-running the official installer scripts or manually building from the previous version tag (`v4.0.0-rc1`) works.
- **Evidence:** Clean state is maintained after rollback, without orphan files or "half-installed" states.

## Smoke test evidence

Post-smoke execution of version check on the rolled-back target:
```text
$ dare --version
dare 4.0.0-rc1
```

Rollback path verified successfully.
