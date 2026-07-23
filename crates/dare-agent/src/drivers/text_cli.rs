//! Shared text-output CLI driver logic for Claude / Cursor / Antigravity (microplano 031).

use std::sync::atomic::Ordering;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use dare_core::{
    CancelFlag, CoreError, CoreResult, ErrorKind, ProcessOutput, ProcessRunner, ProjectRoot,
    SafeCommand, SafeRelativePath, SystemProcessRunner,
};
use regex::Regex;

use crate::driver::{AgentDriver, AgentRequest, AgentRunResult, AgentRunStatus, DriverHealth};

use super::{
    executable_not_found, finalize_result, parse_argv_override, AGENT_DRIVER_TIMEOUT, MSG_MALFORMED,
};

const DOCTOR_TIMEOUT: Duration = Duration::from_secs(5);

static TOKENS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\btokens?\s*[:=]\s*(\d+)\b").expect("tokens regex is valid")
});

/// Static defaults for a text CLI agent driver.
pub(crate) struct TextCliConfig {
    pub id: &'static str,
    pub env_key: &'static str,
    pub default_program: &'static str,
    pub default_args: &'static [&'static str],
    pub supports_model_flag: bool,
}

/// Shared implementation behind Claude / Cursor / Antigravity drivers.
pub(crate) struct TextCliDriver {
    id: &'static str,
    program: String,
    base_args: Vec<String>,
    supports_model_flag: bool,
    runner: Arc<dyn ProcessRunner>,
}

impl TextCliDriver {
    pub(crate) fn from_env(config: &TextCliConfig) -> CoreResult<Self> {
        Self::from_env_with_runner(config, Arc::new(SystemProcessRunner))
    }

    pub(crate) fn from_env_with_runner(
        config: &TextCliConfig,
        runner: Arc<dyn ProcessRunner>,
    ) -> CoreResult<Self> {
        let (program, base_args) = match std::env::var(config.env_key) {
            Ok(val) => parse_argv_override(&val)?,
            Err(_) => (
                config.default_program.to_string(),
                config
                    .default_args
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect(),
            ),
        };
        Ok(Self {
            id: config.id,
            program,
            base_args,
            supports_model_flag: config.supports_model_flag,
            runner,
        })
    }

    #[cfg(test)]
    pub(crate) fn for_test(config: &TextCliConfig, runner: Arc<dyn ProcessRunner>) -> Self {
        Self {
            id: config.id,
            program: config.default_program.to_string(),
            base_args: config
                .default_args
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            supports_model_flag: config.supports_model_flag,
            runner,
        }
    }

    pub(crate) fn doctor(&self) -> CoreResult<DriverHealth> {
        let cmd = SafeCommand::new(&self.program)
            .arg("--version")
            .timeout(DOCTOR_TIMEOUT)
            .stdout_limit(4000)
            .stderr_limit(4000);

        let health = match self.runner.run(&cmd) {
            Err(e) if e.kind() == ErrorKind::NotFound => DriverHealth {
                driver: self.id.into(),
                ok: false,
                detail: executable_not_found(&self.program),
            },
            Err(e) => DriverHealth {
                driver: self.id.into(),
                ok: false,
                detail: e.to_string(),
            },
            Ok(out) if out.timed_out => DriverHealth {
                driver: self.id.into(),
                ok: false,
                detail: "version probe timed out".into(),
            },
            Ok(out) if out.exit_code == 127 => DriverHealth {
                driver: self.id.into(),
                ok: false,
                detail: executable_not_found(&self.program),
            },
            Ok(out) => {
                let detail = out.stdout.trim();
                let detail = if detail.is_empty() {
                    out.stderr.trim()
                } else {
                    detail
                };
                let detail = if detail.is_empty() {
                    "available".to_string()
                } else {
                    detail.to_string()
                };
                DriverHealth {
                    driver: self.id.into(),
                    ok: true,
                    detail,
                }
            }
        };
        Ok(health)
    }

    pub(crate) fn run(
        &self,
        req: &AgentRequest,
        cancel: &CancelFlag,
    ) -> CoreResult<AgentRunResult> {
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

        let root = ProjectRoot::new(&req.cwd)?;
        let rel = SafeRelativePath::new(".")?;

        let mut cmd = SafeCommand::new(&self.program)
            .args(self.base_args.iter().cloned())
            .stdin(req.prompt.as_bytes().to_vec())
            .timeout(AGENT_DRIVER_TIMEOUT)
            .stdout_limit(req.stdout_cap_chars)
            .stderr_limit(req.stdout_cap_chars)
            .cwd(root, rel)
            .cancel_flag(Arc::clone(cancel));

        if self.supports_model_flag {
            if let Some(model) = req.model.as_deref() {
                cmd = cmd.arg("--model").arg(model);
            }
        }

        let output = match self.runner.run(&cmd) {
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return Err(CoreError::internal(executable_not_found(&self.program)));
            }
            Err(e) => return Err(e),
            Ok(out) => out,
        };

        let parsed = parse_text_output(&output);
        Ok(finalize_result(
            AgentRunResult {
                status: parsed.status,
                summary: parsed.summary,
                stdout: output.stdout,
                stderr: output.stderr,
                tokens: parsed.tokens,
                duration_ms: start.elapsed().as_millis() as u64,
            },
            req.stdout_cap_chars,
            output.timed_out,
            output.exit_code,
            output.cancelled || cancel.load(Ordering::SeqCst),
        ))
    }
}

struct ParsedText {
    status: AgentRunStatus,
    summary: String,
    tokens: Option<u64>,
}

fn parse_tokens(stdout: &str) -> Option<u64> {
    TOKENS_RE
        .captures(stdout)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u64>().ok())
}

fn parse_text_output(out: &ProcessOutput) -> ParsedText {
    let tokens = parse_tokens(&out.stdout);

    if out.cancelled {
        return ParsedText {
            status: AgentRunStatus::Cancelled,
            summary: "cancelled".into(),
            tokens,
        };
    }
    if out.timed_out || out.exit_code == 124 {
        return ParsedText {
            status: AgentRunStatus::Timeout,
            summary: "timed out".into(),
            tokens,
        };
    }

    if out.exit_code == 0 {
        let trimmed = out.stdout.trim();
        let summary = if trimmed.is_empty() {
            "(empty)".to_string()
        } else {
            trimmed.to_string()
        };
        return ParsedText {
            status: AgentRunStatus::Success,
            summary,
            tokens,
        };
    }

    let stderr = out.stderr.trim();
    let stdout = out.stdout.trim();
    let summary = if !stderr.is_empty() {
        stderr.to_string()
    } else if !stdout.is_empty() {
        stdout.to_string()
    } else {
        MSG_MALFORMED.to_string()
    };
    ParsedText {
        status: AgentRunStatus::Failure,
        summary,
        tokens,
    }
}

/// Thin AgentDriver wrapper around [`TextCliDriver`].
pub(crate) struct TextAgentDriver {
    inner: TextCliDriver,
}

impl TextAgentDriver {
    pub(crate) fn from_env(config: &TextCliConfig) -> CoreResult<Self> {
        Ok(Self {
            inner: TextCliDriver::from_env(config)?,
        })
    }

    pub(crate) fn from_env_with_runner(
        config: &TextCliConfig,
        runner: Arc<dyn ProcessRunner>,
    ) -> CoreResult<Self> {
        Ok(Self {
            inner: TextCliDriver::from_env_with_runner(config, runner)?,
        })
    }

    #[cfg(test)]
    pub(crate) fn for_test(config: &TextCliConfig, runner: Arc<dyn ProcessRunner>) -> Self {
        Self {
            inner: TextCliDriver::for_test(config, runner),
        }
    }
}

impl AgentDriver for TextAgentDriver {
    fn id(&self) -> &'static str {
        self.inner.id
    }

    fn doctor(&self) -> CoreResult<DriverHealth> {
        self.inner.doctor()
    }

    fn run(&self, req: &AgentRequest, cancel: &CancelFlag) -> CoreResult<AgentRunResult> {
        self.inner.run(req, cancel)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    use dare_core::{MockProcessRunner, ProcessOutput};
    use tempfile::tempdir;

    use super::*;
    use crate::drivers::{ENV_ANTIGRAVITY, ENV_CLAUDE, ENV_CURSOR};

    pub(crate) const CLAUDE_CFG: TextCliConfig = TextCliConfig {
        id: "claude",
        env_key: ENV_CLAUDE,
        default_program: "claude",
        default_args: &["-p", "--output-format", "text"],
        supports_model_flag: true,
    };

    pub(crate) const CURSOR_CFG: TextCliConfig = TextCliConfig {
        id: "cursor",
        env_key: ENV_CURSOR,
        default_program: "cursor-agent",
        default_args: &["--print"],
        supports_model_flag: false,
    };

    pub(crate) const ANTIGRAVITY_CFG: TextCliConfig = TextCliConfig {
        id: "antigravity",
        env_key: ENV_ANTIGRAVITY,
        default_program: "antigravity",
        default_args: &["agent", "--print"],
        supports_model_flag: false,
    };

    const ALL_CFGS: &[TextCliConfig] = &[CLAUDE_CFG, CURSOR_CFG, ANTIGRAVITY_CFG];

    fn out(
        exit_code: i32,
        stdout: &str,
        stderr: &str,
        timed_out: bool,
        cancelled: bool,
    ) -> ProcessOutput {
        ProcessOutput {
            exit_code,
            stdout: stdout.into(),
            stderr: stderr.into(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out,
            cancelled,
        }
    }

    fn req_in(dir: &std::path::Path) -> AgentRequest {
        AgentRequest {
            task_id: "t1".into(),
            prompt: "do the task".into(),
            cwd: PathBuf::from(dir),
            model: None,
            stdout_cap_chars: 4000,
        }
    }

    fn driver(cfg: &TextCliConfig, runner: Arc<dyn ProcessRunner>) -> TextAgentDriver {
        TextAgentDriver::for_test(cfg, runner)
    }

    #[test]
    fn doctor_ok_all_drivers() {
        for cfg in ALL_CFGS {
            let mock = MockProcessRunner::new();
            mock.push(out(0, "v1.0.0\n", "", false, false));
            let d = driver(cfg, Arc::new(mock));
            let h = d.doctor().unwrap();
            assert!(h.ok, "id={}", cfg.id);
            assert_eq!(h.driver, cfg.id);
            assert!(h.detail.contains("v1.0.0"), "detail={}", h.detail);
        }
    }

    #[test]
    fn doctor_missing_all_drivers() {
        for cfg in ALL_CFGS {
            let mock = MockProcessRunner::new();
            mock.push_err(CoreError::not_found("executable not found"));
            let d = driver(cfg, Arc::new(mock));
            let h = d.doctor().unwrap();
            assert!(!h.ok, "id={}", cfg.id);
            assert!(
                h.detail.contains(&format!("executable not found: {}", cfg.default_program)),
                "detail={}",
                h.detail
            );
        }
    }

    #[test]
    fn run_success_all_drivers() {
        let dir = tempdir().unwrap();
        for cfg in ALL_CFGS {
            let mock = MockProcessRunner::new();
            mock.push(out(0, "  done task\ntokens: 42\n  ", "", false, false));
            let d = driver(cfg, Arc::new(mock));
            let cancel = Arc::new(AtomicBool::new(false));
            let r = d.run(&req_in(dir.path()), &cancel).unwrap();
            assert_eq!(r.status, AgentRunStatus::Success, "id={}", cfg.id);
            assert!(r.summary.starts_with("done task"), "summary={}", r.summary);
            assert_eq!(r.tokens, Some(42), "id={}", cfg.id);
        }
    }

    #[test]
    fn run_success_empty_stdout_summary() {
        let dir = tempdir().unwrap();
        let mock = MockProcessRunner::new();
        mock.push(out(0, "   \n", "", false, false));
        let d = driver(&CLAUDE_CFG, Arc::new(mock));
        let cancel = Arc::new(AtomicBool::new(false));
        let r = d.run(&req_in(dir.path()), &cancel).unwrap();
        assert_eq!(r.status, AgentRunStatus::Success);
        assert_eq!(r.summary, "(empty)");
        assert_eq!(r.tokens, None);
    }

    #[test]
    fn run_failure_all_drivers() {
        let dir = tempdir().unwrap();
        for cfg in ALL_CFGS {
            let mock = MockProcessRunner::new();
            mock.push(out(2, "", "boom failed", false, false));
            let d = driver(cfg, Arc::new(mock));
            let cancel = Arc::new(AtomicBool::new(false));
            let r = d.run(&req_in(dir.path()), &cancel).unwrap();
            assert_eq!(r.status, AgentRunStatus::Failure, "id={}", cfg.id);
            assert_eq!(r.summary, "boom failed");
        }
    }

    #[test]
    fn run_timeout_all_drivers() {
        let dir = tempdir().unwrap();
        for cfg in ALL_CFGS {
            let mock = MockProcessRunner::new();
            mock.push(out(124, "", "", true, false));
            let d = driver(cfg, Arc::new(mock));
            let cancel = Arc::new(AtomicBool::new(false));
            let r = d.run(&req_in(dir.path()), &cancel).unwrap();
            assert_eq!(r.status, AgentRunStatus::Timeout, "id={}", cfg.id);
        }
    }

    #[test]
    fn run_cancel_all_drivers() {
        let dir = tempdir().unwrap();
        for cfg in ALL_CFGS {
            // Early cancel — runner must not be consulted.
            let mock = MockProcessRunner::new();
            let d = driver(cfg, Arc::new(mock));
            let cancel = Arc::new(AtomicBool::new(true));
            let r = d.run(&req_in(dir.path()), &cancel).unwrap();
            assert_eq!(r.status, AgentRunStatus::Cancelled, "id={}", cfg.id);
        }
    }

    #[test]
    fn run_malformed_all_drivers() {
        let dir = tempdir().unwrap();
        for cfg in ALL_CFGS {
            let mock = MockProcessRunner::new();
            // Non-zero exit with empty streams → failure + MSG_MALFORMED.
            mock.push(out(1, "", "", false, false));
            let d = driver(cfg, Arc::new(mock));
            let cancel = Arc::new(AtomicBool::new(false));
            let r = d.run(&req_in(dir.path()), &cancel).unwrap();
            assert_eq!(r.status, AgentRunStatus::Failure, "id={}", cfg.id);
            assert_eq!(r.summary, MSG_MALFORMED, "id={}", cfg.id);
        }
    }

    #[test]
    fn run_missing_exe_all_drivers() {
        let dir = tempdir().unwrap();
        for cfg in ALL_CFGS {
            let mock = MockProcessRunner::new();
            mock.push_err(CoreError::not_found("executable not found"));
            let d = driver(cfg, Arc::new(mock));
            let cancel = Arc::new(AtomicBool::new(false));
            let err = d.run(&req_in(dir.path()), &cancel).unwrap_err();
            assert_eq!(err.kind(), ErrorKind::Internal, "id={}", cfg.id);
            assert!(
                err.to_string()
                    .contains(&format!("executable not found: {}", cfg.default_program)),
                "msg={}",
                err
            );
        }
    }

    #[test]
    fn run_secret_redaction_all_drivers() {
        let dir = tempdir().unwrap();
        for cfg in ALL_CFGS {
            let mock = MockProcessRunner::new();
            mock.push(out(
                0,
                "Authorization Bearer sk-secret done",
                "Authorization Bearer sk-secret",
                false,
                false,
            ));
            let d = driver(cfg, Arc::new(mock));
            let cancel = Arc::new(AtomicBool::new(false));
            let r = d.run(&req_in(dir.path()), &cancel).unwrap();
            assert_eq!(r.status, AgentRunStatus::Success, "id={}", cfg.id);
            assert!(!r.stdout.contains("sk-secret"), "stdout={}", r.stdout);
            assert!(!r.stderr.contains("sk-secret"), "stderr={}", r.stderr);
            assert!(!r.summary.contains("sk-secret"), "summary={}", r.summary);
            assert!(
                r.stdout.contains("Bearer [REDACTED]"),
                "stdout={}",
                r.stdout
            );
        }
    }

    #[test]
    fn claude_appends_model_flag() {
        use dare_core::SafeCommand;

        struct CaptureRunner {
            last: std::sync::Mutex<Option<SafeCommand>>,
            inner: MockProcessRunner,
        }

        impl ProcessRunner for CaptureRunner {
            fn run(&self, cmd: &SafeCommand) -> CoreResult<ProcessOutput> {
                *self.last.lock().unwrap() = Some(cmd.clone());
                self.inner.run(cmd)
            }
        }

        let dir = tempdir().unwrap();
        let mock = MockProcessRunner::new();
        mock.push(out(0, "ok", "", false, false));
        let capture = Arc::new(CaptureRunner {
            last: std::sync::Mutex::new(None),
            inner: mock,
        });
        let d = driver(&CLAUDE_CFG, Arc::clone(&capture) as Arc<dyn ProcessRunner>);
        let mut req = req_in(dir.path());
        req.model = Some("sonnet".into());
        let cancel = Arc::new(AtomicBool::new(false));
        let _ = d.run(&req, &cancel).unwrap();
        let cmd = capture.last.lock().unwrap().clone().expect("captured");
        let args = cmd.arg_list();
        assert!(
            args.windows(2).any(|w| w[0] == "--model" && w[1] == "sonnet"),
            "args={args:?}"
        );
    }

    #[test]
    fn cursor_ignores_model_flag() {
        use dare_core::SafeCommand;

        struct CaptureRunner {
            last: std::sync::Mutex<Option<SafeCommand>>,
            inner: MockProcessRunner,
        }

        impl ProcessRunner for CaptureRunner {
            fn run(&self, cmd: &SafeCommand) -> CoreResult<ProcessOutput> {
                *self.last.lock().unwrap() = Some(cmd.clone());
                self.inner.run(cmd)
            }
        }

        let dir = tempdir().unwrap();
        let mock = MockProcessRunner::new();
        mock.push(out(0, "ok", "", false, false));
        let capture = Arc::new(CaptureRunner {
            last: std::sync::Mutex::new(None),
            inner: mock,
        });
        let d = driver(&CURSOR_CFG, Arc::clone(&capture) as Arc<dyn ProcessRunner>);
        let mut req = req_in(dir.path());
        req.model = Some("ignored".into());
        let cancel = Arc::new(AtomicBool::new(false));
        let _ = d.run(&req, &cancel).unwrap();
        let cmd = capture.last.lock().unwrap().clone().expect("captured");
        assert!(!cmd.arg_list().iter().any(|a| a == "--model"));
    }

    #[test]
    fn parse_tokens_variants() {
        assert_eq!(parse_tokens("tokens: 7"), Some(7));
        assert_eq!(parse_tokens("token = 99"), Some(99));
        assert_eq!(parse_tokens("no usage here"), None);
    }
}
