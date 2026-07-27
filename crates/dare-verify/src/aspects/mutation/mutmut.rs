//! Adapter: `mutmut` (python / fastapi).

use dare_core::SafeCommand;

use super::parse_score_best_effort;

/// Build argv-only command for mutmut.
pub fn build_command(_full_mutation: bool) -> SafeCommand {
    SafeCommand::new("mutmut").arg("run")
}

/// Best-effort mutation score from mutmut stdout.
pub fn parse_score(stdout: &str) -> Option<f64> {
    // mutmut often prints "Killed X out of Y" or "X/Y".
    if let Some(score) = parse_killed_of(stdout) {
        return Some(score);
    }
    parse_score_best_effort(stdout)
}

fn parse_killed_of(stdout: &str) -> Option<f64> {
    let lower = stdout.to_ascii_lowercase();
    // "killed 34 out of 40" / "34/40"
    if let Some(idx) = lower.find("killed") {
        let after = &lower[idx + "killed".len()..];
        let nums: Vec<u64> = after
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .take(2)
            .filter_map(|s| s.parse().ok())
            .collect();
        if nums.len() == 2 && nums[1] > 0 {
            return Some(nums[0] as f64 / nums[1] as f64);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mutmut_killed() {
        let out = "Killed 34 out of 40 mutants\n";
        let score = parse_score(out).unwrap();
        assert!((score - 0.85).abs() < 1e-9);
    }
}
