//! Adapter: `infection` (php / laravel).

use dare_core::SafeCommand;

use super::parse_score_best_effort;

/// Build argv-only command for Infection.
pub fn build_command(_full_mutation: bool) -> SafeCommand {
    SafeCommand::new("infection")
}

/// Best-effort mutation score from Infection stdout (MSI).
pub fn parse_score(stdout: &str) -> Option<f64> {
    for line in stdout.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("msi") || lower.contains("mutation score") {
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
    fn parse_infection_msi() {
        let out = "Mutation Score Indicator (MSI): 70%\n";
        let score = parse_score(out).unwrap();
        assert!((score - 0.70).abs() < 1e-9);
    }
}
