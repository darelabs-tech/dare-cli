//! Adapter: `cargo-mutants` (rust / rust-axum / leptos).

use dare_core::SafeCommand;

use super::parse_score_best_effort;

/// Build argv-only command for cargo-mutants.
pub fn build_command(full_mutation: bool) -> SafeCommand {
    let mut cmd = SafeCommand::new("cargo-mutants");
    if !full_mutation {
        // Incremental hint when supported by the tool; ignored if unsupported.
        cmd = cmd.arg("--in-diff");
    }
    cmd
}

/// Best-effort mutation score from cargo-mutants stdout.
pub fn parse_score(stdout: &str) -> Option<f64> {
    // Prefer "N caught, M missed" → caught / (caught + missed).
    if let Some(score) = parse_caught_missed(stdout) {
        return Some(score);
    }
    parse_score_best_effort(stdout)
}

fn parse_caught_missed(stdout: &str) -> Option<f64> {
    // e.g. "35 caught, 5 missed" or "caught 35 missed 5"
    let lower = stdout.to_ascii_lowercase();
    let caught = extract_count_near(&lower, "caught")?;
    let missed = extract_count_near(&lower, "missed").unwrap_or(0);
    let denom = caught + missed;
    if denom == 0 {
        return None;
    }
    Some(caught as f64 / denom as f64)
}

fn extract_count_near(hay: &str, label: &str) -> Option<u64> {
    // "35 caught" or "caught: 35" / "caught 35"
    if let Some(idx) = hay.find(label) {
        let before = &hay[..idx];
        if let Some(n) = trailing_number(before) {
            return Some(n);
        }
        let after = &hay[idx + label.len()..];
        if let Some(n) = leading_number(after) {
            return Some(n);
        }
    }
    None
}

fn trailing_number(s: &str) -> Option<u64> {
    let trimmed = s.trim_end_matches(|c: char| !c.is_ascii_digit());
    let start = trimmed
        .rfind(|c: char| !c.is_ascii_digit())
        .map(|i| i + 1)
        .unwrap_or(0);
    let digits = &trimmed[start..];
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn leading_number(s: &str) -> Option<u64> {
    let s = s.trim_start_matches(|c: char| c == ':' || c == '=' || c.is_whitespace());
    let end = s
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(s.len());
    let digits = &s[..end];
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_caught_missed_sample() {
        let out = "42 mutants tested: 35 caught, 5 missed, 2 unviable\n";
        let score = parse_score(out).unwrap();
        assert!((score - 35.0 / 40.0).abs() < 1e-9);
    }
}
