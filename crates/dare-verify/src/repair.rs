//! Repair loop for agent / best-of paths (max [`REPAIR_MAX`] attempts).

use crate::report::{AspectResult, AspectStatus};

/// Maximum repair re-runs for a failing candidate (§0.10).
pub const REPAIR_MAX: u32 = 5;

/// Outcome of a bounded repair loop.
#[derive(Debug, Clone, PartialEq)]
pub struct RepairOutcome {
    /// One [`AspectResult`] per attempt (length 1..=[`REPAIR_MAX`]).
    pub attempts: Vec<AspectResult>,
    /// `true` if any attempt reached [`AspectStatus::Pass`].
    pub ok: bool,
}

/// Re-run `driver` until it returns pass or the attempt counter hits [`REPAIR_MAX`].
///
/// `driver` receives the 1-based attempt number. The loop never exceeds
/// [`REPAIR_MAX`] invocations — exhausted attempts → `ok: false`.
pub fn run_repair<F>(mut driver: F) -> RepairOutcome
where
    F: FnMut(u32) -> AspectResult,
{
    let mut attempts = Vec::with_capacity(REPAIR_MAX as usize);
    let mut n = 0u32;

    while n < REPAIR_MAX {
        n += 1;
        debug_assert!(n <= REPAIR_MAX);
        let result = driver(n);
        let passed = result.status == AspectStatus::Pass;
        attempts.push(result);
        if passed {
            return RepairOutcome { attempts, ok: true };
        }
    }

    RepairOutcome {
        attempts,
        ok: false,
    }
}

/// Whether another repair attempt is allowed given attempts already made.
pub fn can_repair(attempts_so_far: u32) -> bool {
    attempts_so_far < REPAIR_MAX
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::AdvancedAspect;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn fail_result() -> AspectResult {
        AspectResult {
            aspect: AdvancedAspect::Formal,
            status: AspectStatus::Fail,
            score: None,
            reason: Some("still_failing".into()),
            exit_code: Some(1),
            duration_ms: 0,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
        }
    }

    fn pass_result() -> AspectResult {
        AspectResult {
            aspect: AdvancedAspect::Formal,
            status: AspectStatus::Pass,
            score: Some(1.0),
            reason: None,
            exit_code: Some(0),
            duration_ms: 0,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
        }
    }

    #[test]
    fn repair_stops_at_five() {
        let calls = AtomicU32::new(0);
        let outcome = run_repair(|_| {
            calls.fetch_add(1, Ordering::SeqCst);
            fail_result()
        });
        assert!(!outcome.ok);
        assert_eq!(outcome.attempts.len(), REPAIR_MAX as usize);
        assert_eq!(calls.load(Ordering::SeqCst), REPAIR_MAX);
        assert!(!can_repair(REPAIR_MAX));
        assert!(can_repair(REPAIR_MAX - 1));
    }

    #[test]
    fn repair_stops_early_on_pass() {
        let calls = AtomicU32::new(0);
        let outcome = run_repair(|n| {
            calls.fetch_add(1, Ordering::SeqCst);
            if n >= 2 {
                pass_result()
            } else {
                fail_result()
            }
        });
        assert!(outcome.ok);
        assert_eq!(outcome.attempts.len(), 2);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}
