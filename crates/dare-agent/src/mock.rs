//! Mock / noop agent driver controlled by `DARE_AGENT_MOCK`.

use std::sync::atomic::Ordering;
use std::time::Instant;

use dare_core::{redact, truncate_chars, CancelFlag, CoreResult};

use crate::driver::{AgentDriver, AgentRequest, AgentRunResult, AgentRunStatus, DriverHealth};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MockMode {
    #[default]
    Success,
    Fail,
    Timeout,
}

impl MockMode {
    pub fn from_env() -> Self {
        match std::env::var("DARE_AGENT_MOCK")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "fail" => Self::Fail,
            "timeout" => Self::Timeout,
            _ => Self::Success,
        }
    }
}

#[derive(Debug, Default)]
pub struct MockDriver {
    mode: MockMode,
}

impl MockDriver {
    pub fn from_env() -> Self {
        Self {
            mode: MockMode::from_env(),
        }
    }

    pub fn with_mode(mode: MockMode) -> Self {
        Self { mode }
    }
}

impl AgentDriver for MockDriver {
    fn id(&self) -> &'static str {
        "mock"
    }

    fn doctor(&self) -> CoreResult<DriverHealth> {
        Ok(DriverHealth {
            driver: "mock".into(),
            ok: true,
            detail: "mock ready".into(),
        })
    }

    fn run(&self, req: &AgentRequest, cancel: &CancelFlag) -> CoreResult<AgentRunResult> {
        let start = Instant::now();
        if cancel.load(Ordering::SeqCst) {
            return Ok(AgentRunResult {
                status: AgentRunStatus::Cancelled,
                summary: "mock cancelled".into(),
                stdout: String::new(),
                stderr: String::new(),
                tokens: None,
                duration_ms: start.elapsed().as_millis() as u64,
            });
        }

        let (stdout_raw, _) = truncate_chars(String::new(), req.stdout_cap_chars);
        let stdout = redact(&stdout_raw);

        let result = match self.mode {
            MockMode::Success => AgentRunResult {
                status: AgentRunStatus::Success,
                summary: "mock success".into(),
                stdout,
                stderr: String::new(),
                tokens: Some(1),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            MockMode::Fail => AgentRunResult {
                status: AgentRunStatus::Failure,
                summary: "mock failure".into(),
                stdout,
                stderr: redact("mock failure"),
                // Some(1) enables budget accounting across Continue retries (Blueprint edge case).
                tokens: Some(1),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            MockMode::Timeout => AgentRunResult {
                status: AgentRunStatus::Timeout,
                summary: "mock timeout".into(),
                stdout,
                stderr: String::new(),
                tokens: None,
                duration_ms: start.elapsed().as_millis() as u64,
            },
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    use super::*;

    fn req() -> AgentRequest {
        AgentRequest {
            task_id: "t1".into(),
            prompt: "do it".into(),
            cwd: PathBuf::from("."),
            model: None,
            stdout_cap_chars: 4000,
        }
    }

    #[test]
    fn mock_success() {
        let d = MockDriver::with_mode(MockMode::Success);
        let cancel = Arc::new(AtomicBool::new(false));
        let r = d.run(&req(), &cancel).unwrap();
        assert_eq!(r.status, AgentRunStatus::Success);
        assert_eq!(r.summary, "mock success");
        assert_eq!(r.tokens, Some(1));
        assert!(d.doctor().unwrap().ok);
    }

    #[test]
    fn mock_fail() {
        let d = MockDriver::with_mode(MockMode::Fail);
        let cancel = Arc::new(AtomicBool::new(false));
        let r = d.run(&req(), &cancel).unwrap();
        assert_eq!(r.status, AgentRunStatus::Failure);
        assert_eq!(r.stderr, "mock failure");
        assert_eq!(r.tokens, Some(1));
    }

    #[test]
    fn mock_timeout() {
        let d = MockDriver::with_mode(MockMode::Timeout);
        let cancel = Arc::new(AtomicBool::new(false));
        let r = d.run(&req(), &cancel).unwrap();
        assert_eq!(r.status, AgentRunStatus::Timeout);
    }

    #[test]
    fn mock_cancel() {
        let d = MockDriver::with_mode(MockMode::Success);
        let cancel = Arc::new(AtomicBool::new(true));
        let r = d.run(&req(), &cancel).unwrap();
        assert_eq!(r.status, AgentRunStatus::Cancelled);
    }
}
