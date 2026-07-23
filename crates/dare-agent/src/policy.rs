//! Fixed retry policy for agent runs.

use crate::driver::AgentRunStatus;

/// Maximum agent attempts per task (Documento Mestre §5.5).
pub const MAX_AGENT_ATTEMPTS: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixedDecision {
    Done,
    Continue,
    Stop,
}

/// Apply fixed policy after a run.
/// `attempt_n` is 1-based (count of attempts including the current one).
pub fn apply_fixed(status: AgentRunStatus, attempt_n: u32, max_attempts: u32) -> FixedDecision {
    match status {
        AgentRunStatus::Success => FixedDecision::Done,
        AgentRunStatus::Failure => {
            if attempt_n < max_attempts {
                FixedDecision::Continue
            } else {
                FixedDecision::Stop
            }
        }
        AgentRunStatus::Timeout | AgentRunStatus::Cancelled => FixedDecision::Stop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_fixed_matrix() {
        assert_eq!(
            apply_fixed(AgentRunStatus::Success, 1, 5),
            FixedDecision::Done
        );
        assert_eq!(
            apply_fixed(AgentRunStatus::Failure, 1, 5),
            FixedDecision::Continue
        );
        assert_eq!(
            apply_fixed(AgentRunStatus::Failure, 4, 5),
            FixedDecision::Continue
        );
        assert_eq!(
            apply_fixed(AgentRunStatus::Failure, 5, 5),
            FixedDecision::Stop
        );
        assert_eq!(
            apply_fixed(AgentRunStatus::Timeout, 1, 5),
            FixedDecision::Stop
        );
        assert_eq!(
            apply_fixed(AgentRunStatus::Cancelled, 2, 5),
            FixedDecision::Stop
        );
    }
}
