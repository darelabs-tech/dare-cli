# BLUEPRINT — Operational Diagnostics

## Existing architecture
`dare info` in `crates/dare-cli/src/commands/info.rs` already exposes read-only project discovery, embedded-asset checks, state presence and JSON reporting. `dare validate` already performs DAG validation. `dare ai doctor` and `dare graph doctor` are separate specialized diagnostics.

## Implementation
1. Extend `InfoReport` with a versioned, deterministic diagnostic array. Do not emit file contents or exception strings that may contain secrets.
2. Check whether existing `dare.config.json` and `.dare/state.json` parse as JSON; use stable diagnostic codes with remediation, not raw parser errors.
3. Check whether a DARE DAG file exists; defer structural DAG validation to the existing `dare validate` command rather than duplicate it.
4. Keep `dare info` read-only and use existing global `--json` envelope.
5. Add regression tests for malformed config/state, missing DAG, valid input, and secret non-disclosure.

## Deferred to separate verified work
Lock-owner/staleness interpretation requires identifying the canonical state/lock implementation; support bundle creation requires an explicit preview/redaction contract. Neither is considered complete by this blueprint alone.

## Verification
`cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo audit` (where available), CI run evidence. No DONE without results.
