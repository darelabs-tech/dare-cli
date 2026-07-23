//! Static anti-stub / mock / TODO review (microplano 032).

mod agent;
mod format;
mod report;
mod rules;
mod run;
mod scan;
mod spec;
mod types;

pub use agent::{load_agent_semantic, AgentSemantic};
pub use format::{format_github, format_human, report_to_json};
pub use report::{compute_ok, should_fail_exit, ReviewReport};
pub use rules::apply_line;
pub use run::{run_review, ReviewOptions};
pub use scan::{is_scannable_path, is_test_path, scan_text};
pub use spec::{execution_spec_rel, parse_spec_files, task_id_is_path_safe};
pub use types::{FailOn, Finding, OutputFormat, Severity};

pub const REPORT_SCHEMA: u32 = 1;
pub const MAX_FILE_BYTES: u64 = 1_048_576;
pub const EXECUTION_DIR_REL: &str = "DARE/EXECUTION";
pub const MSG_PASS: &str = "Review passed.";
pub const MSG_FAIL: &str = "Review failed.";
