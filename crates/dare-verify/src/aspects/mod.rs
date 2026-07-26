//! Advanced verification aspects (fail-to-pass, anti-tamper, …).

pub mod anti_tamper;
pub mod fail_to_pass;

pub use anti_tamper::check_anti_tamper;
pub use fail_to_pass::check_fail_to_pass;
