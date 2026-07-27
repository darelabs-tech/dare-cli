//! Advanced verification aspects (fail-to-pass, anti-tamper, …).

pub mod anti_tamper;
pub mod fail_to_pass;
pub mod formal;

pub use anti_tamper::check_anti_tamper;
pub use fail_to_pass::check_fail_to_pass;
pub use formal::{check, run_formal, FormalBackend, MSG_FORMAL_MISSING};
