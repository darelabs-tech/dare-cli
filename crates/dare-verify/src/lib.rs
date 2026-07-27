//! Ralph Loop verification: stack adapters + gated build → test → lint.

pub mod aspects;
pub mod bench;
pub mod ralph;
pub mod repair;
pub mod report;
pub mod stacks;
pub mod verification;

pub use aspects::{
    check, check_anti_tamper, check_fail_to_pass, run_formal, run_mutation, run_mutation_with,
    FormalBackend, MSG_FORMAL_MISSING, MSG_MUTATION_MISSING, MUTATION_THRESHOLD,
};
pub use bench::{
    compare_baseline, compute_drop_pp, compute_fixture_fix_rate, compute_solve_rate,
    compute_suite_fix_rate, load_baseline, load_suite, round_4dp, run_bench, BaselineComparison,
    BaselineFile, BenchOptions, BenchReport, FixtureResult, LoadedCase, LoadedSuite, SuiteCase,
    SuiteFile, BENCH_REPORT_SCHEMA, DEFAULT_SUITE_REL, MSG_BASELINE_INVALID, MSG_SUITE_INVALID,
};
pub use ralph::{run_ralph, GateAspect, GateStep, RalphReport, RALPH_TIMEOUT_SECS};
pub use repair::{can_repair, run_repair, RepairOutcome, REPAIR_MAX};
pub use report::{
    AdvancedAspect, AspectResult, AspectStatus, BestOfCandidate, BestOfSummary, LoopVerdict,
    LOOP_VERDICT_SCHEMA,
};
pub use stacks::{gate_commands, resolve_stack};
pub use verification::{
    load_verification, task_id_is_path_safe, verification_from_ralph, write_verification,
    VerificationReport, VERIFICATION_DIR_REL,
};
