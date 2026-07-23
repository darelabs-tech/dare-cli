//! AgentDriver trait and shared request/result types.

use std::path::PathBuf;

use dare_core::{CancelFlag, CoreResult};
use serde::Serialize;

use crate::drivers::{AntigravityDriver, ClaudeDriver, CodexDriver, CursorDriver};
use crate::mock::MockDriver;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriverHealth {
    pub driver: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentRunStatus {
    Success,
    Failure,
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct AgentRequest {
    pub task_id: String,
    pub prompt: String,
    pub cwd: PathBuf,
    pub model: Option<String>,
    pub stdout_cap_chars: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunResult {
    pub status: AgentRunStatus,
    pub summary: String,
    pub stdout: String,
    pub stderr: String,
    pub tokens: Option<u64>,
    pub duration_ms: u64,
}

pub trait AgentDriver: Send + Sync {
    fn id(&self) -> &'static str;
    fn doctor(&self) -> CoreResult<DriverHealth>;
    fn run(&self, req: &AgentRequest, cancel: &CancelFlag) -> CoreResult<AgentRunResult>;
}

/// Resolve a driver by id (case-sensitive lowercase).
///
/// `mock` / `noop` → [`MockDriver`]; `codex` / `claude` / `cursor` / `antigravity` → real CLI
/// drivers; anything else → InvalidInput `"driver not implemented: {id}"`.
pub fn resolve_driver(id: &str) -> CoreResult<Box<dyn AgentDriver>> {
    match id {
        "mock" | "noop" => Ok(Box::new(MockDriver::from_env())),
        "codex" => Ok(Box::new(CodexDriver::from_env()?)),
        "claude" => Ok(Box::new(ClaudeDriver::from_env()?)),
        "cursor" => Ok(Box::new(CursorDriver::from_env()?)),
        "antigravity" => Ok(Box::new(AntigravityDriver::from_env()?)),
        other => Err(dare_core::CoreError::invalid_input(format!(
            "driver not implemented: {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_mock_and_noop() {
        assert_eq!(resolve_driver("mock").unwrap().id(), "mock");
        assert_eq!(resolve_driver("noop").unwrap().id(), "mock");
    }

    #[test]
    fn resolve_real_drivers_from_env_defaults() {
        std::env::remove_var(crate::drivers::ENV_CODEX);
        std::env::remove_var(crate::drivers::ENV_CLAUDE);
        std::env::remove_var(crate::drivers::ENV_CURSOR);
        std::env::remove_var(crate::drivers::ENV_ANTIGRAVITY);
        assert_eq!(resolve_driver("codex").unwrap().id(), "codex");
        assert_eq!(resolve_driver("claude").unwrap().id(), "claude");
        assert_eq!(resolve_driver("cursor").unwrap().id(), "cursor");
        assert_eq!(resolve_driver("antigravity").unwrap().id(), "antigravity");
    }

    #[test]
    fn resolve_unknown_not_implemented() {
        match resolve_driver("not-a-driver") {
            Ok(_) => panic!("expected error"),
            Err(err) => {
                let msg = err.to_string();
                assert!(
                    msg.contains("driver not implemented: not-a-driver"),
                    "msg={msg}"
                );
            }
        }
    }

    #[test]
    fn resolve_mixed_case_rejected() {
        match resolve_driver("Claude") {
            Ok(_) => panic!("expected error"),
            Err(err) => {
                let msg = err.to_string();
                assert!(msg.contains("driver not implemented: Claude"), "msg={msg}");
            }
        }
    }
}
