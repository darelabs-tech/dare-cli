//! Antigravity CLI text driver (microplano 031).

use std::sync::Arc;

use dare_core::{CancelFlag, CoreResult, ProcessRunner};

use super::text_cli::{TextAgentDriver, TextCliConfig};
use super::ENV_ANTIGRAVITY;
use crate::driver::{AgentDriver, AgentRequest, AgentRunResult, DriverHealth};

const CONFIG: TextCliConfig = TextCliConfig {
    id: "antigravity",
    env_key: ENV_ANTIGRAVITY,
    default_program: "antigravity",
    default_args: &["agent", "--print"],
    supports_model_flag: false,
};

/// Antigravity CLI agent driver (`antigravity agent --print`).
pub struct AntigravityDriver {
    inner: TextAgentDriver,
}

impl AntigravityDriver {
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

impl AgentDriver for AntigravityDriver {
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
