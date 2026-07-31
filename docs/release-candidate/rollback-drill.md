# Rollback drill — RC `v4.0.0-rc1`

> **Microplano:** 055 · mp055-006  
> **Install scope:** ephemeral temp prefix only (global `dare` untouched)

## MUST fields

| Field | Value |
|-------|-------|
| `operator` | `mp055-006-agent` (Cursor worktree `mp055-006-cdf84219`) |
| `date` | `2026-07-31T14:24:00Z` |
| `os` | `windows` |
| `from_version` | `v4.0.0-rc1` |
| `to_version` | prior known binary — local smoke artifact `dist/smoke/prefix/bin/dare.exe` (SHA256 `6B23DB0D5E08C6990CCB2A8A7425FC1D446D79679ED01370B002679FF1EC4181`) |
| `method_a` | `dare self rollback` → **ok** |
| `method_b` | reinstall previous → **ok** |
| `post_smoke` | `--version` → `dare 0.1.0-alpha.0` (exit 0) after both methods |
| `result` | **PASS** |

## Context

GitHub Release assets for `v4.0.0-rc1` remain blocked (`blocked:actions_billing`; see `publish-checklist.md`). Drill used local cargo-built / smoke binaries against a **temp install prefix** under `%TEMP%\dare-rollback-drill-mp055-006` with isolated `DARE_SELF_HOME` (no writes to the operator global install).

| Role | Source | SHA256 |
|------|--------|--------|
| Current (`from_version` stand-in) | `target/debug/dare.exe` at HEAD lineage of tag `v4.0.0-rc1` (has `dare self`) | `081B764619E74A839AA53A207A432123A5B117FDDC1D43F155F6E5A7E9D090DB` |
| Previous (`to_version`) | `dist/smoke/prefix/bin/dare.exe` (prior RC smoke artifact) | `6B23DB0D5E08C6990CCB2A8A7425FC1D446D79679ED01370B002679FF1EC4181` |

Embedded clap version string remains `0.1.0-alpha.0` (known lag vs product tag `v4.0.0-rc1`; same note as RC smoke docs). Identity for this drill is the **file hash** swap + tag label.

## method_a — `dare self rollback`

1. Installed current binary into `$DrillRoot/prefix/bin/dare.exe`.
2. Staged prior binary at `$DARE_SELF_HOME/backup/dare.exe` (simulates post–self-update backup history in the temp home).
3. Ran:

```powershell
$env:DARE_SELF_HOME = "$env:TEMP\dare-rollback-drill-mp055-006\self"
& "$env:TEMP\dare-rollback-drill-mp055-006\prefix\bin\dare.exe" self rollback -y
```

**Output:**

```
self rollback: ok
backup: C:\Users\wande\AppData\Local\Temp\dare-rollback-drill-mp055-006\self\backup\dare.exe
restored: C:\Users\wande\AppData\Local\Temp\dare-rollback-drill-mp055-006\prefix\bin\dare.exe
mode: rollback
```

**Integrity:** installed SHA256 after rollback = previous hash (`6B23DB0D…`). Exit 0. No half-installed state.

### post_smoke (method_a)

```
dare 0.1.0-alpha.0
```

Exit code: `0`.

## method_b — reinstall previous

1. Reset prefix binary to current (RC stand-in hash `081B7646…`).
2. Reinstalled previous by copying the prior smoke artifact over the prefix binary.
3. Confirmed SHA256 = previous (`6B23DB0D…`).

**Verdict:** **ok**.

### post_smoke (method_b)

```
dare 0.1.0-alpha.0
```

Exit code: `0`.

## Result

Both rollback paths completed cleanly on Windows against an isolated temp prefix. Prefix binary matched the prior artifact hash after each method; `--version` smoked successfully. **`result: PASS`**.
