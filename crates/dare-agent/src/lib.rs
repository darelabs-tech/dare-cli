//! Agent drivers, mock, budget, fixed policy, failure signatures, and worktrees (microplano 030+031).

mod budget;
mod driver;
pub mod drivers;
mod mock;
mod policy;
mod signature;
mod worktree;

pub use budget::BudgetTracker;
pub use driver::{
    resolve_driver, AgentDriver, AgentRequest, AgentRunResult, AgentRunStatus, DriverHealth,
};
pub use mock::{MockDriver, MockMode};
pub use policy::{apply_fixed, FixedDecision, MAX_AGENT_ATTEMPTS};
pub use signature::{failure_signature, normalize_stderr};
pub use worktree::{WorktreeManager, WorktreeSpec, AGENT_WORKTREES_REL};
