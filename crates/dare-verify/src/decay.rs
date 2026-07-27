//! Decay retry policy for agent runs (Blueprint-049 §0.7).

use dare_agent::{failure_signature, AgentRunStatus};

/// Consecutive identical failure signatures before FreshStart.
pub const DECAY_WINDOW: u32 = 3;

/// Maximum attempts under decay (then Escalate / Stop).
pub const DECAY_MAX_ATTEMPTS: u32 = 5;

/// Usage message template for unknown `--policy` values.
pub const MSG_POLICY_UNKNOWN_PREFIX: &str = "unknown policy: ";

/// Format [`MSG_POLICY_UNKNOWN`](crate::decay::msg_policy_unknown).
pub fn msg_policy_unknown(p: &str) -> String {
    format!("unknown policy: {p} (expected fixed|decay)")
}

/// Decay policy decision after one agent attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecayAction {
    Done,
    Continue,
    FreshStart,
    Replan,
    Escalate,
    Stop,
}

impl DecayAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Continue => "continue",
            Self::FreshStart => "fresh_start",
            Self::Replan => "replan",
            Self::Escalate => "escalate",
            Self::Stop => "stop",
        }
    }
}

/// Count trailing identical signatures ending with `last_sig`.
///
/// `recent_signatures` is prior history (does **not** include the current `last_sig`).
fn consecutive_identical(recent_signatures: &[String], last_sig: &str) -> u32 {
    let mut count = 1u32;
    for s in recent_signatures.iter().rev() {
        if s == last_sig {
            count = count.saturating_add(1);
        } else {
            break;
        }
    }
    count
}

/// Apply decay policy after a run (§0.7).
///
/// - Success → [`DecayAction::Done`]
/// - Timeout / cancel → [`DecayAction::Stop`]
/// - Failure: streak of identical signatures (via [`failure_signature`] at call site):
///   - `< DECAY_WINDOW` → Continue
///   - `== DECAY_WINDOW` → FreshStart
///   - `== DECAY_WINDOW + 1` → Replan
///   - `>= DECAY_WINDOW + 2` or `attempt_n >= DECAY_MAX_ATTEMPTS` → Escalate
///   - next failure after Escalate band (`> DECAY_WINDOW + 2` or `attempt_n > DECAY_MAX`) → Stop
pub fn apply_decay(
    status: AgentRunStatus,
    attempt_n: u32,
    recent_signatures: &[String],
    last_sig: &str,
) -> DecayAction {
    match status {
        AgentRunStatus::Success => DecayAction::Done,
        AgentRunStatus::Timeout | AgentRunStatus::Cancelled => DecayAction::Stop,
        AgentRunStatus::Failure => {
            let streak = consecutive_identical(recent_signatures, last_sig);
            if attempt_n > DECAY_MAX_ATTEMPTS || streak > DECAY_WINDOW + 2 {
                return DecayAction::Stop;
            }
            if attempt_n >= DECAY_MAX_ATTEMPTS || streak >= DECAY_WINDOW + 2 {
                return DecayAction::Escalate;
            }
            if streak == DECAY_WINDOW + 1 {
                return DecayAction::Replan;
            }
            if streak == DECAY_WINDOW {
                return DecayAction::FreshStart;
            }
            DecayAction::Continue
        }
    }
}

/// Convenience: hash stderr the same way the agent loop does.
pub fn signature_for(aspect: &str, stderr: &str) -> String {
    failure_signature(aspect, stderr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sigs(n: usize, s: &str) -> Vec<String> {
        std::iter::repeat_n(s.to_string(), n).collect()
    }

    #[test]
    fn decay_success_and_timeout() {
        assert_eq!(
            apply_decay(AgentRunStatus::Success, 1, &[], "x"),
            DecayAction::Done
        );
        assert_eq!(
            apply_decay(AgentRunStatus::Timeout, 1, &[], "x"),
            DecayAction::Stop
        );
        assert_eq!(
            apply_decay(AgentRunStatus::Cancelled, 2, &[], "x"),
            DecayAction::Stop
        );
    }

    #[test]
    fn decay_matrix_continue_fresh_replan_escalate_stop() {
        let s = "aabb";
        // streak 1 (no history)
        assert_eq!(
            apply_decay(AgentRunStatus::Failure, 1, &[], s),
            DecayAction::Continue
        );
        // streak 2
        assert_eq!(
            apply_decay(AgentRunStatus::Failure, 2, &sigs(1, s), s),
            DecayAction::Continue
        );
        // streak 3 == WINDOW → FreshStart
        assert_eq!(
            apply_decay(AgentRunStatus::Failure, 3, &sigs(2, s), s),
            DecayAction::FreshStart
        );
        // streak 4 → Replan
        assert_eq!(
            apply_decay(AgentRunStatus::Failure, 4, &sigs(3, s), s),
            DecayAction::Replan
        );
        // streak 5 → Escalate
        assert_eq!(
            apply_decay(AgentRunStatus::Failure, 4, &sigs(4, s), s),
            DecayAction::Escalate
        );
        // streak 6 → Stop
        assert_eq!(
            apply_decay(AgentRunStatus::Failure, 4, &sigs(5, s), s),
            DecayAction::Stop
        );
        // attempt_n == MAX → Escalate (even with short streak)
        assert_eq!(
            apply_decay(AgentRunStatus::Failure, DECAY_MAX_ATTEMPTS, &[], "other"),
            DecayAction::Escalate
        );
        // attempt_n > MAX → Stop
        assert_eq!(
            apply_decay(
                AgentRunStatus::Failure,
                DECAY_MAX_ATTEMPTS + 1,
                &[],
                "other"
            ),
            DecayAction::Stop
        );
    }

    #[test]
    fn different_signature_resets_streak() {
        let recent = vec!["aaa".into(), "bbb".into()];
        assert_eq!(
            apply_decay(AgentRunStatus::Failure, 3, &recent, "ccc"),
            DecayAction::Continue
        );
    }

    #[test]
    fn failure_signature_reused() {
        let a = signature_for("agent", "MOCK Failure\n");
        let b = failure_signature("agent", "mock   failure");
        assert_eq!(a, b);
    }

    #[test]
    fn msg_policy_unknown_format() {
        assert_eq!(
            msg_policy_unknown("foo"),
            "unknown policy: foo (expected fixed|decay)"
        );
    }
}
