//! Claude Code CLI text driver (microplano 031).

use std::sync::Arc;

use dare_core::{CancelFlag, CoreResult, ProcessRunner};

use super::text_cli::{TextAgentDriver, TextCliConfig};
use super::ENV_CLAUDE;
use crate::driver::{AgentDriver, AgentRequest, AgentRunResult, DriverHealth};

const CONFIG: TextCliConfig = TextCliConfig {
    id: "claude",
    env_key: ENV_CLAUDE,
    default_program: "claude",
    default_args: &["-p", "--output-format", "text"],
    supports_model_flag: true,
};

/// Claude Code CLI agent driver (`--output-format text`).
pub struct ClaudeDriver {
    inner: TextAgentDriver,
}

impl ClaudeDriver {
    pub fn from_env() -> CoreResult<Self> {
        Ok(Self {
            inner: TextAgentDriver::from_env(&CONFIG)?,
        })
    }

    pub fn from_env_with_runner(runner: Arc<dyn ProcessRunner>) -> CoreResult<Self> {
        Ok(Self {
            inner: TextAgentDriver::from_env_with_runner(&CONFIG, runner)?,
        })
    }
}

impl AgentDriver for ClaudeDriver {
    fn id(&self) -> &'static str {
        self.inner.id()
    }

    fn doctor(&self) -> CoreResult<DriverHealth> {
        self.inner.doctor()
    }

    fn run(&self, req: &AgentRequest, cancel: &CancelFlag) -> CoreResult<AgentRunResult> {
        self.inner.run(req, cancel)
    }
}
