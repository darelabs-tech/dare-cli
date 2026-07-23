//! Guard pipeline: unicode, injection scan, provenance, signing (microplano 034).

mod evidence;
mod pipeline;
mod preflight;
mod provenance;
mod report;
mod rules;
mod scan;
mod signing;
mod unicode;

pub use pipeline::{
    format_human, guard_fail_error, process_exit_for_report, report_to_json, run_guard, scan_paths,
    GuardOptions, UnicodeMode,
};
pub use preflight::{run_preflight, PreflightOptions};
pub use provenance::{classify_provenance, Provenance};
pub use report::{Finding, FindingSeverity, GuardReport, GuardVerdict};
pub use rules::{load_rules, load_rules_from_str, ScanRule, ScanRulesFile, DEFAULT_RULES_JSON};
pub use signing::{public_key_hex_from_private, sign_file, verify_file, SIG_EXT};
pub use unicode::{analyze_unicode, strip_unicode, UnicodeHit, UnicodeKind};

pub const READ_CAP: usize = 1_048_576;
pub const DEFAULT_RULES_REL: &str = "assets/rules/scan-rules.json";
pub const MSG_GUARD_FAIL: &str = "guard failed";
pub const MSG_PREFLIGHT_FAIL: &str = "guard preflight failed";
