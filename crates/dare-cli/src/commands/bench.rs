//! `dare bench` — deterministic Fix·Rate harness (microplano 049).

use std::path::PathBuf;
use std::process::ExitCode;

use dare_core::{CoreError, CoreResult, ProjectRoot, SystemProcessRunner};
use dare_verify::{run_bench, BenchOptions, BenchReport, DEFAULT_SUITE_REL};
use serde_json::Value;

use crate::output::OutputRenderer;

/// CLI args for `dare bench`.
pub struct BenchCliOpts {
    pub suite: Option<PathBuf>,
    pub baseline: Option<PathBuf>,
    pub fail_on_regression: Option<u32>,
    pub filter: Option<String>,
    pub dir: Option<PathBuf>,
}

/// Run `dare bench`; regression → exit 1 with report; suite/usage → exit 2.
pub fn run_bench_cmd(opts: BenchCliOpts, renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_bench_inner(opts) {
        Ok((human, data, exit)) => {
            let ok = exit == 0;
            if let Err(e) = renderer.write_report(&human, data, ok) {
                return exit_err(renderer, &e);
            }
            ExitCode::from(exit as u8)
        }
        Err(e) => exit_err(renderer, &e),
    }
}

fn exit_err(renderer: &OutputRenderer<'_>, e: &CoreError) -> ExitCode {
    let code = renderer.write_error(e);
    ExitCode::from(code as u8)
}

fn run_bench_inner(opts: BenchCliOpts) -> CoreResult<(String, Value, i32)> {
    let root_path = opts
        .dir
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = ProjectRoot::new(&root_path)?;

    let bench_opts = BenchOptions {
        suite: opts.suite,
        baseline: opts.baseline,
        fail_on_regression: opts.fail_on_regression,
        filter: opts.filter,
    };

    let report = run_bench(&root, &bench_opts, &SystemProcessRunner)?;
    let human = format_human(&report);
    let data = serde_json::to_value(&report).map_err(|e| CoreError::internal(e.to_string()))?;

    let exit = if report
        .baseline
        .as_ref()
        .map(|b| b.regression_failed)
        .unwrap_or(false)
    {
        1
    } else {
        0
    };

    Ok((human, data, exit))
}

fn format_human(report: &BenchReport) -> String {
    let mut lines = vec![format!(
        "bench: suite={} fixRate={:.4} solveRate={:.4} fixtures={}",
        if report.suite_path.is_empty() {
            DEFAULT_SUITE_REL
        } else {
            report.suite_path.as_str()
        },
        report.fix_rate,
        report.solve_rate,
        report.fixtures.len()
    )];
    for f in &report.fixtures {
        lines.push(format!(
            "  {}  ok={}  fixRate={:.4}  ftp={}/{}  ptp_failed={}",
            f.id,
            f.ok,
            f.fix_rate,
            f.fail_to_pass_passed,
            f.fail_to_pass_total,
            f.pass_to_pass_failed
        ));
    }
    if let Some(b) = &report.baseline {
        lines.push(format!(
            "  baseline={}  dropSolvePp={:.4}  regressionFailed={}",
            b.path, b.drop_solve_pp, b.regression_failed
        ));
    }
    lines.join("\n")
}
