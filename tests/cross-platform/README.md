# Cross-platform suite

OS-sensitive path and process cases (Windows separators, drive letters, etc.).

## Layout

| Path | Role |
|------|------|
| `windows-path-cases/case.yaml` | CaseSpec metadata for separator / drive-casing coverage |
| `crates/dare-parity/tests/cross_platform.rs` | Unit + integration asserts (`cfg(windows)` / `cfg(unix)`) |

## Property tests (MUST in PR)

`proptest` modules in `dare-parity` (`fuzz_paths`, `fuzz_parsers`) run under
`cargo test -p dare-parity` with **256** cases — no panic on arbitrary paths / YAML bytes.

## cargo-fuzz (SHOULD, not required in PR)

Full coverage-guided fuzzing via [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz)
**SHOULD** run on **nightly** (or a scheduled CI job). It is **not** required to install
`cargo-fuzz` or pass fuzz targets for a green PR / Ralph gate — `proptest` is the PR MUST.
