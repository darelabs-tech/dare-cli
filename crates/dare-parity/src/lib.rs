//! Parity harness: golden cases, security suite, and perf regression gate.
//!
//! Modules `normalize`, `runner`, and `security` arrive in later mp054 tasks.

mod axis;
mod case;
mod perf;

#[cfg(test)]
mod fuzz_parsers;
#[cfg(test)]
mod fuzz_paths;

pub use axis::CompareAxis;
pub use case::{
    load_case, validate_case, CaseSpec, ContentExpect, DiffClass, HttpExpect, SkipSpec,
    CASE_SCHEMA_VERSION, MSG_SKIP_NEEDS_CLASS,
};
pub use perf::{within_regression, PERF_REGRESSION_MAX};
