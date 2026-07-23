//! Ralph Loop runner: build → test → lint with short-circuit.

use std::time::Instant;

use dare_core::{
    truncate_chars, CoreResult, ProcessOutput, ProcessRunner, ProjectRoot, SafeCommand,
    SafeRelativePath,
};
use serde::{Deserialize, Serialize};

use crate::stacks::gate_commands;

/// Per-gate timeout (seconds).
pub const RALPH_TIMEOUT_SECS: u64 = 600;

/// Gate aspect in Ralph order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GateAspect {
    Build,
    Test,
    Lint,
}

impl GateAspect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Test => "test",
            Self::Lint => "lint",
        }
    }
}

/// One executed Ralph gate step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GateStep {
    pub aspect: GateAspect,
    pub program: String,
    pub args: Vec<String>,
    pub exit_code: i32,
    pub timed_out: bool,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub duration_ms: u64,
}

/// Aggregate Ralph report (no state I/O).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RalphReport {
    pub ok: bool,
    pub timed_out: bool,
    pub stack: String,
    pub steps: Vec<GateStep>,
    pub total_duration_ms: u64,
}

/// Run Ralph gates for an **implemented** stack.
///
/// Short-circuits on the first step with `exit_code != 0` or `timed_out`
/// (timeout → exit **124**). Truncates stdio tails with
/// [`dare_core::truncate_chars`]. Never touches DAG/state.
pub fn run_ralph(
    root: &ProjectRoot,
    stack: &str,
    runner: &dyn ProcessRunner,
    stdout_cap_chars: usize,
) -> CoreResult<RalphReport> {
    let gates = gate_commands(stack)?;
    let planned = gates.len();
    let root_rel = SafeRelativePath::new(".")?;
    let mut steps = Vec::with_capacity(planned);
    let mut timed_out = false;
    let total_start = Instant::now();

    for (aspect, template) in gates {
        let program = template.program().to_string();
        let args: Vec<String> = template.arg_list().to_vec();
        let cmd = SafeCommand::new(program.clone())
            .args(args.iter().cloned())
            .cwd(root.clone(), root_rel.clone())
            .timeout(std::time::Duration::from_secs(RALPH_TIMEOUT_SECS))
            .stdout_limit(stdout_cap_chars)
            .stderr_limit(stdout_cap_chars);

        let step_start = Instant::now();
        let out: ProcessOutput = runner.run(&cmd)?;
        let duration_ms = step_start.elapsed().as_millis() as u64;

        let (stdout_tail, _) = truncate_chars(out.stdout, stdout_cap_chars);
        let (stderr_tail, _) = truncate_chars(out.stderr, stdout_cap_chars);

        let step_timed_out = out.timed_out || out.exit_code == 124;
        let exit_code = if step_timed_out { 124 } else { out.exit_code };

        steps.push(GateStep {
            aspect,
            program,
            args,
            exit_code,
            timed_out: step_timed_out,
            stdout_tail,
            stderr_tail,
            duration_ms,
        });

        if step_timed_out {
            timed_out = true;
            break;
        }
        if exit_code != 0 {
            break;
        }
    }

    // ok iff all planned gates ran with exit 0 and none timed out.
    let ok = !timed_out
        && steps.len() == planned
        && steps.iter().all(|s| s.exit_code == 0 && !s.timed_out);

    Ok(RalphReport {
        ok,
        timed_out,
        stack: stack.to_string(),
        steps,
        total_duration_ms: total_start.elapsed().as_millis() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::{CoreError, MockProcessRunner, ProcessOutput};

    fn ok_out() -> ProcessOutput {
        ProcessOutput {
            exit_code: 0,
            stdout: "ok".into(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        }
    }

    fn fail_out(code: i32) -> ProcessOutput {
        ProcessOutput {
            exit_code: code,
            stdout: String::new(),
            stderr: "failed".into(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        }
    }

    fn timeout_out() -> ProcessOutput {
        ProcessOutput {
            exit_code: 124,
            stdout: String::new(),
            stderr: "timeout".into(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: true,
            cancelled: false,
        }
    }

    fn test_root() -> (tempfile::TempDir, ProjectRoot) {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        (dir, root)
    }

    #[test]
    fn run_ralph_all_pass() {
        let (_dir, root) = test_root();
        let mock = MockProcessRunner::new();
        mock.push(ok_out());
        mock.push(ok_out());
        mock.push(ok_out());

        let report = run_ralph(&root, "rust-axum", &mock, 4000).expect("ralph");
        assert!(report.ok);
        assert!(!report.timed_out);
        assert_eq!(report.stack, "rust-axum");
        assert_eq!(report.steps.len(), 3);
        assert_eq!(report.steps[0].aspect, GateAspect::Build);
        assert_eq!(report.steps[1].aspect, GateAspect::Test);
        assert_eq!(report.steps[2].aspect, GateAspect::Lint);
        assert!(report
            .steps
            .iter()
            .all(|s| s.exit_code == 0 && !s.timed_out));
    }

    #[test]
    fn run_ralph_fail_short_circuits() {
        let (_dir, root) = test_root();
        let mock = MockProcessRunner::new();
        mock.push(ok_out());
        mock.push(fail_out(1));
        mock.push(ok_out()); // must not be consumed

        let report = run_ralph(&root, "rust", &mock, 4000).expect("ralph");
        assert!(!report.ok);
        assert!(!report.timed_out);
        assert_eq!(report.steps.len(), 2);
        assert_eq!(report.steps[1].aspect, GateAspect::Test);
        assert_eq!(report.steps[1].exit_code, 1);

        // Remaining mock response proves lint did not run.
        let leftover = mock.run(&SafeCommand::new("probe")).expect("leftover");
        assert_eq!(leftover.exit_code, 0);
    }

    #[test]
    fn run_ralph_timeout_exit_124() {
        let (_dir, root) = test_root();
        let mock = MockProcessRunner::new();
        mock.push(ok_out());
        mock.push(timeout_out());

        let report = run_ralph(&root, "rust-axum", &mock, 4000).expect("ralph");
        assert!(!report.ok);
        assert!(report.timed_out);
        assert_eq!(report.steps.len(), 2);
        assert_eq!(report.steps[1].aspect, GateAspect::Test);
        assert_eq!(report.steps[1].exit_code, 124);
        assert!(report.steps[1].timed_out);
    }

    #[test]
    fn run_ralph_rejects_unimplemented_stack() {
        let (_dir, root) = test_root();
        let mock = MockProcessRunner::new();
        let err = run_ralph(&root, "node-nestjs", &mock, 4000).expect_err("unimplemented");
        assert!(matches!(err, CoreError::InvalidInput(_)));
        assert!(err.to_string().contains("not implemented"));
    }
}
