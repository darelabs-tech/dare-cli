# REVIEW — Operational Diagnostics

## Scope review
The approved instruction is to generate the entire DARE cycle in the existing PR #9 branch. Existing implementation in `crates/dare-cli/src/commands/info.rs` is retained. The cycle records implementation evidence separately from successful verification.

## Architecture decisions
- Reuse `dare info`, `dare validate`, specialized AI/graph doctor commands.
- Avoid duplicate lock parsing until canonical lock ownership is established.
- Support bundle must be explicit opt-in, local-only, previewable and redact sensitive data.
- Existing global `--json` output remains the transport for machine-readable reports.

## Risks
- Raw paths or parser errors can disclose sensitive information; never include raw invalid file content.
- Parsing `.dare/state.json` as JSON is a syntax check, not semantic validation.
- The DAG warning checks the conventional path, not alternative user-selected DAGs.

## Gate
User authorized cycle generation and implementation in this branch. No merge authorization. No task may be marked DONE without build, test, lint and applicable audit evidence.
