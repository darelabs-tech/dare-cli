//! Brownfield project detection and install.

pub mod detect;
pub mod dna;
pub mod git;
pub mod harnesses;
pub mod install;
pub mod migrate;
pub mod monorepo;
pub mod patterns;
pub mod report;
pub mod reverse;
pub mod root;
pub mod stacks;

pub use detect::{detect, format_human, report_to_json};
pub use dna::{
    format_human as format_dna_human, report_to_json as dna_report_to_json, run_dna, DnaOptions,
    DnaReport, DNA_FACTS_REL, DNA_SCHEMA_VERSION, PROJECT_DNA_REL,
};
pub use git::find_git_root;
pub use harnesses::{detect_harnesses, empty_harnesses};
pub use install::{
    apply_install, format_install_human, install, install_report_to_json, plan_install, select_ide,
    InstallOptions, InstallPlan, InstallReport, StepResult, INSTALL_READ_CAP,
    INSTALL_SCHEMA_VERSION,
};
pub use migrate::{
    build_blocking_gaps, build_phases, compare_migration, sort_blocking_gaps, target_family,
    validate_migrate_target, BlockingGap, MigrateOptions, MigrateReport, MigrationPhase,
    MIGRATE_SCHEMA_VERSION, MIGRATE_TARGET_ALLOWLIST,
};
pub use monorepo::detect_monorepo;
pub use patterns::{
    format_human as format_patterns_human, report_to_json as patterns_report_to_json, run_patterns,
    Cooccurrence, DiscoveredPattern, PatternsOptions, PatternsReport, PATTERNS_FACTS_REL,
    PATTERNS_MD_REL, PATTERNS_SCHEMA_VERSION, PATTERN_KINDS,
};
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
