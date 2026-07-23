//! Shared constants and finalize helpers for CLI agent drivers (microplano 031).

use std::time::Duration;

use dare_core::{redact, truncate_chars};

use crate::driver::{AgentRunResult, AgentRunStatus};

/// Process timeout for real agent drivers (parity with enrich timeout).
pub const AGENT_DRIVER_TIMEOUT: Duration = Duration::from_secs(20 * 60);

/// Max Unicode scalar values kept in [`AgentRunResult::summary`] after finalize.
pub const SUMMARY_MAX_CHARS: usize = 512;

/// Summary when the driver produced no parseable terminal event.
pub const MSG_MALFORMED: &str = "malformed driver output";

/// Format the canonical missing-executable diagnostic.
pub fn executable_not_found(program: &str) -> String {
    format!("executable not found: {program}")
}

/// Apply timeout/cancel status mapping, then truncate + redact stdout/stderr/summary.
pub fn finalize_result(
    mut result: AgentRunResult,
    stdout_cap_chars: usize,
    timed_out: bool,
    exit_code: i32,
    cancelled: bool,
) -> AgentRunResult {
    result.status = if cancelled {
        AgentRunStatus::Cancelled
    } else if timed_out || exit_code == 124 {
        AgentRunStatus::Timeout
    } else {
        result.status
    };

    let (stdout, _) = truncate_chars(result.stdout, stdout_cap_chars);
    result.stdout = redact(&stdout);
    let (stderr, _) = truncate_chars(result.stderr, stdout_cap_chars);
    result.stderr = redact(&stderr);
    let (summary, _) = truncate_chars(result.summary, SUMMARY_MAX_CHARS);
    result.summary = redact(&summary);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(status: AgentRunStatus, summary: &str, stdout: &str) -> AgentRunResult {
        AgentRunResult {
            status,
            summary: summary.into(),
            stdout: stdout.into(),
            stderr: String::new(),
            tokens: None,
            duration_ms: 10,
        }
    }

    #[test]
    fn finalize_redacts_bearer_secret_from_stdout() {
        let r = finalize_result(
            base(
                AgentRunStatus::Success,
                "ok",
                "Authorization Bearer sk-secret done",
            ),
            4000,
            false,
            0,
            false,
        );
        assert_eq!(r.status, AgentRunStatus::Success);
        assert!(!r.stdout.contains("sk-secret"), "stdout={}", r.stdout);
        assert!(
            r.stdout.contains("Bearer [REDACTED]"),
            "stdout={}",
            r.stdout
        );
    }

    #[test]
    fn finalize_maps_timeout_and_cancel() {
        let t = finalize_result(base(AgentRunStatus::Success, "x", ""), 100, true, 0, false);
        assert_eq!(t.status, AgentRunStatus::Timeout);

        let e124 = finalize_result(
            base(AgentRunStatus::Success, "x", ""),
            100,
            false,
            124,
            false,
        );
        assert_eq!(e124.status, AgentRunStatus::Timeout);

        let c = finalize_result(base(AgentRunStatus::Success, "x", ""), 100, true, 124, true);
        assert_eq!(c.status, AgentRunStatus::Cancelled);
    }

    #[test]
    fn executable_not_found_formats_program() {
        assert_eq!(
            executable_not_found("/no/such/bin"),
            "executable not found: /no/such/bin"
        );
    }
}
