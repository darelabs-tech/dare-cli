# REGRESSION — Lock & Support Scenarios

| Scenario | Intended behavior | Evidence status |
|---|---|---|
| Absent lock file | absent, zero files created | Test authored, unverified |
| Residual unlocked lock file | unheld; do not label stale | Test authored, unverified |
| Lock held by another file descriptor | held; no removal | Test authored, unverified |
| Non-regular lock path | unknown; safe failure | Test authored, unverified |
| Invalid config with embedded secret | diagnostic code only; secret excluded | Test authored, unverified |
| Export outside project | write only `dare-support.json` with create_new | Test authored, unverified |
| Export within project | reject | Test authored, unverified |
| Existing destination | reject overwrite | Test authored, unverified |
| Missing output parent or wrong filename | reject | Test authored, unverified |
| CLI human preview | include/exclude list and state-lock state | Implementation authored, unverified |

Required gates: `cargo build --workspace`, `cargo test --workspace`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, applicable advisory scan.
