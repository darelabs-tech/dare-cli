//! Cursor Agent CLI text driver (microplano 031).

use std::sync::Arc;

use dare_core::{CancelFlag, CoreResult, ProcessRunner};

use super::text_cli::{TextAgentDriver, TextCliConfig};
use super::ENV_CURSOR;
use crate::driver::{AgentDriver, AgentRequest, AgentRunResult, DriverHealth};

const CONFIG: TextCliConfig = TextCliConfig {
    id: "cursor",
    env_key: ENV_CURSOR,
    default_program: "cursor-agent",
    default_args: &["--print"],
    supports_model_flag: false,
};

/// Cursor Agent CLI driver (`cursor-agent --print`).
pub struct CursorDriver {
    inner: TextAgentDriver,
}

impl CursorDriver {
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

impl AgentDriver for CursorDriver {
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
