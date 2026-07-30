//! Codex CLI agent driver — JSONL-tolerant parse (microplano 031).

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dare_core::{
    CancelFlag, CoreError, CoreResult, ErrorKind, ProcessOutput, ProcessRunner, ProjectRoot,
    SafeCommand, SafeRelativePath, SystemProcessRunner,
};
use serde_json::Value;

use super::argv::{parse_argv_override, ENV_CODEX};
use super::common::{executable_not_found, finalize_result, AGENT_DRIVER_TIMEOUT, MSG_MALFORMED};
use crate::driver::{AgentDriver, AgentRequest, AgentRunResult, AgentRunStatus, DriverHealth};

const DOCTOR_TIMEOUT: Duration = Duration::from_secs(5);

const DEFAULT_ARGS: &[&str] = &[
    "exec",
    "--json",
    "--sandbox",
    "read-only",
    "--ask-for-approval",
    "never",
];

/// Codex CLI driver (`id = "codex"`).
pub struct CodexDriver {
    program: String,
    base_args: Vec<String>,
    supports_model_flag: bool,
    runner: Arc<dyn ProcessRunner>,
}

impl CodexDriver {
    pub fn from_env() -> CoreResult<Self> {
        Self::from_env_with_runner(Arc::new(SystemProcessRunner))
    }

    pub fn from_env_with_runner(runner: Arc<dyn ProcessRunner>) -> CoreResult<Self> {
        let (program, base_args) = match std::env::var(ENV_CODEX) {
            Ok(val) => parse_argv_override(&val)?,
            Err(_) => (
                "codex".to_string(),
                DEFAULT_ARGS.iter().map(|s| (*s).to_string()).collect(),
            ),
        };
        Ok(Self {
            program,
            base_args,
            supports_model_flag: true,
            runner,
        })
    }

    fn map_missing_exe(&self, err: CoreError) -> CoreError {
        if err.kind() == ErrorKind::NotFound || err.message().contains("executable not found") {
            CoreError::internal(executable_not_found(&self.program))
        } else {
            err
        }
    }

    fn build_run_cmd(&self, req: &AgentRequest, cancel: &CancelFlag) -> CoreResult<SafeCommand> {
        let root = ProjectRoot::new(&req.cwd)?;
        let rel = SafeRelativePath::new(".")?;

        let mut args = self.base_args.clone();
        if self.supports_model_flag {
            if let Some(model) = req.model.as_ref() {
                args.push("--model".to_string());
                args.push(model.clone());
            }
        }

        Ok(SafeCommand::new(&self.program)
            .args(args)
            .stdin(req.prompt.as_bytes().to_vec())
            .timeout(AGENT_DRIVER_TIMEOUT)
            .stdout_limit(req.stdout_cap_chars)
            .stderr_limit(req.stdout_cap_chars)
            .cancel_flag(Arc::clone(cancel))
            .cwd(root, rel))
    }
}

impl AgentDriver for CodexDriver {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn doctor(&self) -> CoreResult<DriverHealth> {
        let cmd = SafeCommand::new(&self.program)
            .arg("--version")
            .timeout(DOCTOR_TIMEOUT);

        let health = match self.runner.run(&cmd) {
            Ok(out) => {
                if out.exit_code == 127 || out.timed_out || looks_like_missing_exe_output(&out) {
                    DriverHealth {
                        driver: "codex".into(),
                        ok: false,
                        detail: executable_not_found(&self.program),
                    }
                } else {
                    let detail = out.stdout.trim();
                    DriverHealth {
                        driver: "codex".into(),
                        ok: true,
                        detail: if detail.is_empty() {
                            "ok".into()
                        } else {
                            detail.chars().take(200).collect()
                        },
                    }
                }
            }
            Err(_) => DriverHealth {
                driver: "codex".into(),
                ok: false,
                detail: executable_not_found(&self.program),
            },
        };
        Ok(health)
    }

    fn run(&self, req: &AgentRequest, cancel: &CancelFlag) -> CoreResult<AgentRunResult> {
        let start = Instant::now();

        if cancel.load(Ordering::SeqCst) {
            return Ok(finalize_result(
                AgentRunResult {
                    status: AgentRunStatus::Cancelled,
                    summary: "cancelled".into(),
                    stdout: String::new(),
                    stderr: String::new(),
                    tokens: None,
                    duration_ms: start.elapsed().as_millis() as u64,
                },
                req.stdout_cap_chars,
                false,
                0,
                true,
            ));
        }

        let cmd = self.build_run_cmd(req, cancel)?;
        let out = match self.runner.run(&cmd) {
            Ok(o) => o,
            Err(e) => return Err(self.map_missing_exe(e)),
        };

        if out.exit_code == 127 {
            return Err(CoreError::internal(executable_not_found(&self.program)));
        }

        let parsed = parse_codex_jsonl(&out.stdout, out.exit_code, &out.stderr);
        let cancelled = cancel.load(Ordering::SeqCst) || out.cancelled;

        Ok(finalize_result(
            AgentRunResult {
                status: parsed.status,
                summary: parsed.summary,
                stdout: out.stdout,
                stderr: out.stderr,
                tokens: parsed.tokens,
                duration_ms: start.elapsed().as_millis() as u64,
            },
            req.stdout_cap_chars,
            out.timed_out,
            out.exit_code,
            cancelled,
        ))
    }
}

fn looks_like_missing_exe_output(out: &ProcessOutput) -> bool {
    let combined = format!("{}\n{}", out.stderr, out.stdout).to_ascii_lowercase();
    combined.contains("not found") || combined.contains("no such file")
}

struct ParsedCodex {
    status: AgentRunStatus,
    summary: String,
    tokens: Option<u64>,
}

fn parse_codex_jsonl(stdout: &str, exit_code: i32, stderr: &str) -> ParsedCodex {
    let mut tokens: Option<u64> = None;
    let mut summary_candidate: Option<String> = None;
    let mut terminal_success = false;
    let mut terminal_failure = false;
    let mut saw_json = false;
    let mut failure_summary: Option<String> = None;

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        saw_json = true;

        if let Some(t) = extract_tokens(&value) {
            tokens = Some(t);
        }
        if let Some(s) = extract_summary_candidate(&value) {
            summary_candidate = Some(s);
        }

        if is_failure_terminal(&value) {
            terminal_failure = true;
            if let Some(s) = extract_summary_candidate(&value) {
                failure_summary = Some(s);
            }
        } else if is_success_terminal(&value) {
            terminal_success = true;
        }
    }

    // (1) failure event → Failure
    if terminal_failure {
        let summary = failure_summary.or(summary_candidate).unwrap_or_else(|| {
            let trimmed = stderr.trim();
            if trimmed.is_empty() {
                "failure".into()
            } else {
                trimmed.chars().take(512).collect()
            }
        });
        return ParsedCodex {
            status: AgentRunStatus::Failure,
            summary,
            tokens,
        };
    }

    // (2) success event → Success
    if terminal_success {
        return ParsedCodex {
            status: AgentRunStatus::Success,
            summary: summary_candidate.unwrap_or_else(|| "(empty)".into()),
            tokens,
        };
    }

    // empty stdout + exit 0 → Success "(empty)"
    if exit_code == 0 && stdout.trim().is_empty() {
        return ParsedCodex {
            status: AgentRunStatus::Success,
            summary: "(empty)".into(),
            tokens: None,
        };
    }

    // (3) exit 0 + non-JSON text → Success
    if exit_code == 0 && !saw_json {
        let summary = stdout.trim().chars().take(512).collect::<String>();
        return ParsedCodex {
            status: AgentRunStatus::Success,
            summary: if summary.is_empty() {
                "(empty)".into()
            } else {
                summary
            },
            tokens: None,
        };
    }

    // (4) exit ≠ 0 → Failure
    if exit_code != 0 {
        let summary = summary_candidate.unwrap_or_else(|| {
            let trimmed = stderr.trim();
            if trimmed.is_empty() {
                format!("exit {exit_code}")
            } else {
                trimmed.chars().take(512).collect()
            }
        });
        return ParsedCodex {
            status: AgentRunStatus::Failure,
            summary,
            tokens,
        };
    }

    // (5) JSON without terminal (or other) → Failure + MSG_MALFORMED
    // Also covers: exit 0 + valid JSON lines without success/failure terminal.
    ParsedCodex {
        status: AgentRunStatus::Failure,
        summary: MSG_MALFORMED.into(),
        tokens,
    }
}

fn is_success_terminal(v: &Value) -> bool {
    if let Some(t) = v.get("type").and_then(|x| x.as_str()) {
        if matches!(t, "turn.completed" | "agent.completed" | "result") {
            return true;
        }
    }
    if let Some(s) = v.get("status").and_then(|x| x.as_str()) {
        if matches!(s, "success" | "completed") {
            return true;
        }
    }
    false
}

fn is_failure_terminal(v: &Value) -> bool {
    if let Some(t) = v.get("type").and_then(|x| x.as_str()) {
        if matches!(t, "error" | "turn.failed") {
            return true;
        }
    }
    if let Some(s) = v.get("status").and_then(|x| x.as_str()) {
        if matches!(s, "failed" | "error") {
            return true;
        }
    }
    false
}

fn extract_tokens(v: &Value) -> Option<u64> {
    if let Some(t) = v.pointer("/usage/total_tokens").and_then(|x| x.as_u64()) {
        return Some(t);
    }
    if let Some(t) = v.get("tokens").and_then(|x| x.as_u64()) {
        return Some(t);
    }
    let input = v
        .pointer("/usage/input_tokens")
        .and_then(|x| x.as_u64())
        .or_else(|| v.get("input_tokens").and_then(|x| x.as_u64()))
        .or_else(|| v.pointer("/usage/input").and_then(|x| x.as_u64()));
    let output = v
        .pointer("/usage/output_tokens")
        .and_then(|x| x.as_u64())
        .or_else(|| v.get("output_tokens").and_then(|x| x.as_u64()))
        .or_else(|| v.pointer("/usage/output").and_then(|x| x.as_u64()));
    match (input, output) {
        (Some(i), Some(o)) => Some(i.saturating_add(o)),
        (Some(i), None) => Some(i),
        (None, Some(o)) => Some(o),
        (None, None) => None,
    }
}

fn extract_summary_candidate(v: &Value) -> Option<String> {
    for key in ["message", "text", "content"] {
        if let Some(s) = v.get(key).and_then(|x| x.as_str()) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::{MockProcessRunner, ProcessOutput};
    use std::sync::atomic::AtomicBool;
    use tempfile::tempdir;

    fn out_ok(stdout: &str) -> ProcessOutput {
        ProcessOutput {
            exit_code: 0,
            stdout: stdout.into(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        }
    }

    fn out_fail(code: i32, stderr: &str) -> ProcessOutput {
        ProcessOutput {
            exit_code: code,
            stdout: String::new(),
            stderr: stderr.into(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        }
    }

    fn sample_req(cwd: std::path::PathBuf) -> AgentRequest {
        AgentRequest {
            task_id: "t1".into(),
            prompt: "do the thing".into(),
            cwd,
            model: None,
            stdout_cap_chars: 4000,
        }
    }

    fn driver_with(mock: MockProcessRunner) -> CodexDriver {
        std::env::remove_var(ENV_CODEX);
        CodexDriver::from_env_with_runner(Arc::new(mock) as Arc<dyn ProcessRunner>)
            .expect("from_env_with_runner")
    }

    #[test]
    fn doctor_ok() {
        let mock = MockProcessRunner::new();
        mock.push(out_ok("codex 0.1.0"));
        let driver = driver_with(mock);
        let h = driver.doctor().expect("doctor");
        assert!(h.ok, "detail={}", h.detail);
        assert_eq!(h.driver, "codex");
        assert!(h.detail.contains("codex") || h.detail == "ok");
    }

    #[test]
    fn doctor_missing() {
        let mock = MockProcessRunner::new();
        mock.push_err(CoreError::not_found("executable not found"));
        let driver = driver_with(mock);
        let h = driver.doctor().expect("doctor always Ok");
        assert!(!h.ok);
        assert!(
            h.detail.contains("executable not found: codex"),
            "detail={}",
            h.detail
        );
    }

    #[test]
    fn run_success_jsonl() {
        let dir = tempdir().expect("temp");
        let mock = MockProcessRunner::new();
        mock.push(out_ok(
            r#"{"type":"turn.completed","usage":{"total_tokens":42},"message":"done task"}"#,
        ));
        let driver = driver_with(mock);
        let cancel = Arc::new(AtomicBool::new(false));
        let result = driver
            .run(&sample_req(dir.path().to_path_buf()), &cancel)
            .expect("run");
        assert_eq!(result.status, AgentRunStatus::Success);
        assert_eq!(result.tokens, Some(42));
        assert_eq!(result.summary, "done task");
    }

    #[test]
    fn run_failure_event() {
        let dir = tempdir().expect("temp");
        let mock = MockProcessRunner::new();
        mock.push(out_ok(r#"{"type":"turn.failed","message":"boom"}"#));
        let driver = driver_with(mock);
        let cancel = Arc::new(AtomicBool::new(false));
        let result = driver
            .run(&sample_req(dir.path().to_path_buf()), &cancel)
            .expect("run");
        assert_eq!(result.status, AgentRunStatus::Failure);
        assert_eq!(result.summary, "boom");
    }

    #[test]
    fn run_timeout() {
        let dir = tempdir().expect("temp");
        let mock = MockProcessRunner::new();
        mock.push(ProcessOutput {
            exit_code: 124,
            stdout: String::new(),
            stderr: "timed out".into(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: true,
            cancelled: false,
        });
        let driver = driver_with(mock);
        let cancel = Arc::new(AtomicBool::new(false));
        let result = driver
            .run(&sample_req(dir.path().to_path_buf()), &cancel)
            .expect("run");
        assert_eq!(result.status, AgentRunStatus::Timeout);
    }

    #[test]
    fn run_cancel_early() {
        let dir = tempdir().expect("temp");
        let mock = MockProcessRunner::new();
        // no push — must not call runner
        let driver = driver_with(mock);
        let cancel = Arc::new(AtomicBool::new(true));
        let result = driver
            .run(&sample_req(dir.path().to_path_buf()), &cancel)
            .expect("run");
        assert_eq!(result.status, AgentRunStatus::Cancelled);
    }

    #[test]
    fn run_malformed_json_without_terminal() {
        let dir = tempdir().expect("temp");
        let mock = MockProcessRunner::new();
        mock.push(out_ok(r#"{"type":"item.updated","message":"partial"}"#));
        let driver = driver_with(mock);
        let cancel = Arc::new(AtomicBool::new(false));
        let result = driver
            .run(&sample_req(dir.path().to_path_buf()), &cancel)
            .expect("run");
        assert_eq!(result.status, AgentRunStatus::Failure);
        assert_eq!(result.summary, MSG_MALFORMED);
    }

    #[test]
    fn run_missing_exe() {
        let dir = tempdir().expect("temp");
        let mock = MockProcessRunner::new();
        mock.push_err(CoreError::not_found("executable not found"));
        let driver = driver_with(mock);
        let cancel = Arc::new(AtomicBool::new(false));
        let err = driver
            .run(&sample_req(dir.path().to_path_buf()), &cancel)
            .expect_err("missing");
        assert_eq!(err.kind(), ErrorKind::Internal);
        assert!(
            err.to_string().contains("executable not found: codex"),
            "msg={err}"
        );
    }

    #[test]
    fn run_secret_redaction() {
        let dir = tempdir().expect("temp");
        let mock = MockProcessRunner::new();
        mock.push(out_ok(
            r#"{"type":"turn.completed","message":"Authorization Bearer sk-secret done"}"#,
        ));
        let driver = driver_with(mock);
        let cancel = Arc::new(AtomicBool::new(false));
        let result = driver
            .run(&sample_req(dir.path().to_path_buf()), &cancel)
            .expect("run");
        assert_eq!(result.status, AgentRunStatus::Success);
        assert!(
            !result.summary.contains("sk-secret"),
            "summary={}",
            result.summary
        );
        assert!(
            !result.stdout.contains("sk-secret"),
            "stdout={}",
            result.stdout
        );
        assert!(
            result.summary.contains("[REDACTED]") || result.stdout.contains("[REDACTED]"),
            "summary={} stdout={}",
            result.summary,
            result.stdout
        );
    }

    #[test]
    fn run_nonzero_exit_is_failure() {
        let dir = tempdir().expect("temp");
        let mock = MockProcessRunner::new();
        mock.push(out_fail(1, "cli exploded"));
        let driver = driver_with(mock);
        let cancel = Arc::new(AtomicBool::new(false));
        let result = driver
            .run(&sample_req(dir.path().to_path_buf()), &cancel)
            .expect("run");
        assert_eq!(result.status, AgentRunStatus::Failure);
        assert!(result.summary.contains("cli exploded"));
    }

    #[test]
    fn run_appends_model_flag() {
        use std::sync::Mutex;

        struct RecordingRunner {
            inner: MockProcessRunner,
            last_cmd: Mutex<Option<SafeCommand>>,
        }

        impl ProcessRunner for RecordingRunner {
            fn run(&self, cmd: &SafeCommand) -> CoreResult<ProcessOutput> {
                *self.last_cmd.lock().expect("lock") = Some(cmd.clone());
                self.inner.run(cmd)
            }
        }

        let dir = tempdir().expect("temp");
        let mock = MockProcessRunner::new();
        mock.push(out_ok(
            r#"{"type":"result","status":"success","message":"ok"}"#,
        ));
        std::env::remove_var(ENV_CODEX);
        let recording = Arc::new(RecordingRunner {
            inner: mock,
            last_cmd: Mutex::new(None),
        });
        let driver =
            CodexDriver::from_env_with_runner(Arc::clone(&recording) as Arc<dyn ProcessRunner>)
                .unwrap();

        let mut req = sample_req(dir.path().to_path_buf());
        req.model = Some("gpt-4.1".into());
        let cancel = Arc::new(AtomicBool::new(false));
        driver.run(&req, &cancel).expect("run");

        let cmd = recording
            .last_cmd
            .lock()
            .expect("lock")
            .clone()
            .expect("cmd");
        assert_eq!(cmd.program(), "codex");
        let args = cmd.arg_list();
        assert!(args
            .windows(2)
            .any(|w| w[0] == "--model" && w[1] == "gpt-4.1"));
        assert!(args.contains(&"read-only".to_string()));
        assert!(cmd.stdin_bytes().is_some());
    }
}
