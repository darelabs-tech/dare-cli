# REGRESSION — Test Matrix

| Scenario | Expected | Status |
|---|---|---|
| `dare info` with valid config/state | No invalid-JSON diagnostics | Test authored; not executed |
| Malformed `dare.config.json` | DARE_CONFIG_INVALID_JSON | Test authored; not executed |
| Malformed `.dare/state.json` | DARE_STATE_INVALID_JSON | Test authored; not executed |
| DARE directory without conventional DAG | DARE_DAG_MISSING warning | Test authored; not executed |
| Secret embedded in malformed input | Never printed in human/JSON report | Test authored; not executed |
| Stale vs active execution lock | Correct actionable diagnostic | Not implemented |
| Support bundle contains secrets | Block or redact before writing | Not implemented |
| Read-only diagnostics | Zero mutations | Existing test; new coverage not verified |

## Required commands
`cargo build --workspace`; `cargo test --workspace`; `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo audit` when available.
