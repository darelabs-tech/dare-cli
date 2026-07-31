//! Parity harness: golden cases, security suite, normalize, and perf regression gate.
//!
//! Module `runner` arrives in a later mp054 task.

mod axis;
mod case;
mod normalize;
mod perf;
pub mod security;

#[cfg(test)]
mod fuzz_parsers;
#[cfg(test)]
mod fuzz_paths;

pub use axis::CompareAxis;
pub use case::{
    load_case, validate_case, CaseSpec, ContentExpect, DiffClass, HttpExpect, SkipSpec,
    CASE_SCHEMA_VERSION, MSG_SKIP_NEEDS_CLASS,
};
pub use normalize::{normalize_text, NormalizeCtx, MSG_OVER_NORMALIZE};
pub use perf::{within_regression, PERF_REGRESSION_MAX};
pub use security::{
    test_archive_traversal_fixtures, test_bidi_path_rejected, test_command_injection_payloads,
    test_env_leak_absent, test_signature_mismatch_fixtures,
};
