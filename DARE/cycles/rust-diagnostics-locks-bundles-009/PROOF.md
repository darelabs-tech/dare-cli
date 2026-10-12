# PROOF — Lock Diagnostics & Support Reports

Implementation commits:
- `70fa0153` support allowlist and export
- `fd795489` CLI support command
- `2a6cb390` read-only fs4 lock probe
- `0161013e` lock probe integrated into support preview
- `cdbdcff3` human-readable preview
- `e5b06123` secret-exclusion and export rejection tests

## Verification status
No passing GitHub CI run was returned for the most recent commit when checked. No local Rust toolchain available for build, formatting or tests. All implementation tasks remain **UNVERIFIED** and are not marked DONE.

## Safety choices
- No age-only stale classification. Available residual locks are described as unheld.
- No lock creation or deletion in read-only probe.
- Export includes generated diagnostics only; excludes project files and raw logs.
- No automatic transmission; export requires explicit destination outside the project.

## Required evidence before ready for review
Actual CI run URL and commit SHA; green Rust fmt, build, test, Clippy and applicable audit results. No merge authorized.
