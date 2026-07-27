//! Ralph Loop verification: stack adapters + gated build → test → lint.

pub mod aspects;
pub mod bench;
pub mod ralph;
pub mod report;
pub mod stacks;
pub mod verification;

pub use aspects::{check_anti_tamper, check_fail_to_pass};
pub use bench::{
    compute_drop_pp, compute_fixture_fix_rate, compute_solve_rate, compute_suite_fix_rate,
    load_baseline, load_suite, round_4dp, BaselineComparison, BaselineFile, BenchReport,
    FixtureResult, LoadedCase, SuiteCase, SuiteFile, DEFAULT_SUITE_REL, MSG_BASELINE_INVALID,
    MSG_SUITE_INVALID,
};
pub use ralph::{run_ralph, GateAspect, GateStep, RalphReport, RALPH_TIMEOUT_SECS};
pub use report::{
    AdvancedAspect, AspectResult, AspectStatus, BestOfCandidate, BestOfSummary, LoopVerdict,
    LOOP_VERDICT_SCHEMA,
};
pub use stacks::{gate_commands, resolve_stack};
pub use verification::{
    load_verification, task_id_is_path_safe, verification_from_ralph, write_verification,
    VerificationReport, VERIFICATION_DIR_REL,
};
