//! Brownfield project detection and install.

pub mod detect;
pub mod git;
pub mod harnesses;
pub mod install;
pub mod monorepo;
pub mod report;
pub mod reverse;
pub mod root;
pub mod stacks;

pub use detect::{detect, format_human, report_to_json};
pub use git::find_git_root;
pub use harnesses::{detect_harnesses, empty_harnesses};
pub use install::{
    apply_install, format_install_human, install, install_report_to_json, plan_install, select_ide,
    InstallOptions, InstallPlan, InstallReport, StepResult, INSTALL_READ_CAP,
    INSTALL_SCHEMA_VERSION,
};
pub use monorepo::detect_monorepo;
pub use report::{
    DetectionReport, HarnessHit, StackConflict, StackHit, DETECTION_SCHEMA_VERSION,
    MANIFEST_READ_CAP, MONOREPO_MAX_DEPTH, MONOREPO_MAX_ENTRIES,
};
pub use reverse::{
    analyze_ast, analyze_modules, format_reverse_human, read_ideia, reverse,
    reverse_report_to_json, write_ideia, AstSummary, ModuleFact, ReverseFacts, ReverseOptions,
    ReverseReport, REVERSE_SCHEMA_VERSION,
};
pub use root::find_project_root;
pub use stacks::detect_stacks;
