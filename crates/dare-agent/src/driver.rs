//! AgentDriver trait and shared request/result types.

use std::path::PathBuf;

use dare_core::{CancelFlag, CoreResult};
use serde::Serialize;

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

/// Resolve a driver by id. `mock` and `noop` map to [`MockDriver`]; others are not implemented.
pub fn resolve_driver(id: &str) -> CoreResult<Box<dyn AgentDriver>> {
    match id {
        "mock" | "noop" => Ok(Box::new(MockDriver::from_env())),
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
    fn resolve_unknown_not_implemented() {
        match resolve_driver("claude") {
            Ok(_) => panic!("expected error"),
            Err(err) => {
                let msg = err.to_string();
                assert!(msg.contains("driver not implemented: claude"), "msg={msg}");
            }
        }
    }
}
