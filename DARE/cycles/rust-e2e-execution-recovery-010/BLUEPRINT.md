# BLUEPRINT — End-to-end recovery regression

## Existing components
- `dare validate`, `dare execute --next`, `--status`, `--complete`, `--reset`, persistent `.dare/state.json`
- `dare info`, `dare support` and redacted JSON support export
- `crates/dare-cli/tests/cli_smoke.rs` covers commands individually; this cycle adds a cross-command, cross-process scenario.

## Test architecture
- New `crates/dare-cli/tests/e2e_recovery.rs` test uses `assert_cmd` executable for each stage.
- Temp project with committed `valid.v21.yaml` fixture and a basic execution spec.
- Simulated Ralph timeout leaves RUNNING; next process checks durable state; explicit reset preserves recoverability; Ralph mock success writes verification and DONE.
- Check JSON output contracts and that generated support export excludes source config and task IDs.

## Gates
`cargo fmt --all -- --check`, `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, dependency audit where available.
