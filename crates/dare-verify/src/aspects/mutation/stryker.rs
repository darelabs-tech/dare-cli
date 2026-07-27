//! Adapter: `stryker` (node / nest / react / vue / mcp-node). Prefer PATH binary.

use dare_core::SafeCommand;

use super::parse_score_best_effort;

/// Build argv-only command for Stryker (Class B: prefer `stryker` on PATH).
pub fn build_command(_full_mutation: bool) -> SafeCommand {
    SafeCommand::new("stryker").arg("run")
}

/// Best-effort mutation score from Stryker stdout.
pub fn parse_score(stdout: &str) -> Option<f64> {
    // Prefer explicit "Mutation score: 75.00%" style lines.
    for line in stdout.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("mutation score") || lower.contains("mutationscore") {
            if let Some(score) = parse_score_best_effort(line) {
                return Some(score);
            }
        }
    }
    parse_score_best_effort(stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stryker_percent() {
        let out = "Mutation score: 82.50%\nAll done.\n";
        let score = parse_score(out).unwrap();
        assert!((score - 0.825).abs() < 1e-9);
    }
}
