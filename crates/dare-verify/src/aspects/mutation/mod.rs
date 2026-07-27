//! Mutation aspect: stack-family adapters + threshold gate.

mod cargo_mutants;
mod infection;
mod mutmut;
mod stryker;

use dare_core::{CoreError, ProcessRunner, SafeCommand, SystemProcessRunner};

use crate::report::{AdvancedAspect, AspectResult, AspectStatus};

/// Minimum mutation score for [`AspectStatus::Pass`] (§0.5).
pub const MUTATION_THRESHOLD: f64 = 0.70;

/// Message when mutation tool is missing under `--full-mutation`.
pub const MSG_MUTATION_MISSING: &str =
    "mutation tool not found on PATH [MUTATION_TOOL_MISSING]";

/// Run mutation for `stack`. When `stdout_fixture` is set, skip spawn and parse that text.
///
/// Tool missing + `!full_mutation` → skipped / `tool_missing`.
/// Tool missing + `full_mutation` → fail / [`MSG_MUTATION_MISSING`].
/// `score >= MUTATION_THRESHOLD` → pass; else fail / `below_threshold`.
pub fn run_mutation(
    stack: &str,
    full_mutation: bool,
    stdout_fixture: Option<&str>,
) -> AspectResult {
    run_mutation_with(
        stack,
        full_mutation,
        stdout_fixture,
        &SystemProcessRunner,
    )
}

/// Like [`run_mutation`] but injects a [`ProcessRunner`] (tests / missing-tool simulation).
pub fn run_mutation_with(
    stack: &str,
    full_mutation: bool,
    stdout_fixture: Option<&str>,
    runner: &dyn ProcessRunner,
) -> AspectResult {
    let Some(family) = MutationFamily::from_stack(stack) else {
        return aspect(
            AspectStatus::Skipped,
            None,
            Some("unsupported_stack".into()),
            None,
            0,
            String::new(),
            String::new(),
        );
    };

    if let Some(fixture) = stdout_fixture {
        return verdict_from_stdout(family, fixture, 0, String::new());
    }

    let cmd = family.build_command(full_mutation);
    match runner.run(&cmd) {
        Ok(out) => {
            let duration_ms = 0;
            verdict_from_stdout(family, &out.stdout, duration_ms, out.stderr)
        }
        Err(e) if is_tool_missing(&e) => missing_tool_result(full_mutation),
        Err(e) => aspect(
            AspectStatus::Fail,
            None,
            Some(format!("mutation_runner_error:{e}")),
            Some(1),
            0,
            String::new(),
            e.to_string(),
        ),
    }
}

fn missing_tool_result(full_mutation: bool) -> AspectResult {
    if full_mutation {
        aspect(
            AspectStatus::Fail,
            None,
            Some(MSG_MUTATION_MISSING.into()),
            Some(1),
            0,
            String::new(),
            String::new(),
        )
    } else {
        aspect(
            AspectStatus::Skipped,
            None,
            Some("tool_missing".into()),
            None,
            0,
            String::new(),
            String::new(),
        )
    }
}

fn verdict_from_stdout(
    family: MutationFamily,
    stdout: &str,
    duration_ms: u64,
    stderr: String,
) -> AspectResult {
    let Some(score) = family.parse_score(stdout) else {
        return aspect(
            AspectStatus::Fail,
            None,
            Some("parse_failed".into()),
            Some(1),
            duration_ms,
            stdout.to_string(),
            stderr,
        );
    };
    let score = clamp01(score);
    if score >= MUTATION_THRESHOLD {
        aspect(
            AspectStatus::Pass,
            Some(score),
            None,
            Some(0),
            duration_ms,
            stdout.to_string(),
            stderr,
        )
    } else {
        aspect(
            AspectStatus::Fail,
            Some(score),
            Some("below_threshold".into()),
            Some(1),
            duration_ms,
            stdout.to_string(),
            stderr,
        )
    }
}

fn aspect(
    status: AspectStatus,
    score: Option<f64>,
    reason: Option<String>,
    exit_code: Option<i32>,
    duration_ms: u64,
    stdout_tail: String,
    stderr_tail: String,
) -> AspectResult {
    AspectResult {
        aspect: AdvancedAspect::Mutation,
        status,
        score,
        reason,
        exit_code,
        duration_ms,
        stdout_tail,
        stderr_tail,
    }
}

fn is_tool_missing(err: &CoreError) -> bool {
    matches!(err, CoreError::NotFound(_))
}

fn clamp01(v: f64) -> f64 {
    if v.is_nan() {
        return 0.0;
    }
    v.clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy)]
enum MutationFamily {
    CargoMutants,
    Stryker,
    Mutmut,
    Infection,
}

impl MutationFamily {
    fn from_stack(stack: &str) -> Option<Self> {
        let s = stack.trim().to_ascii_lowercase();
        match s.as_str() {
            "rust" | "rust-axum" | "leptos" | "rust-leptos" | "rust-leptos-csr" => {
                Some(Self::CargoMutants)
            }
            "node" | "nest" | "node-nestjs" | "react" | "vue" | "mcp-node" | "mcp-node-ts" => {
                Some(Self::Stryker)
            }
            "python" | "fastapi" | "python-fastapi" => Some(Self::Mutmut),
            "php" | "laravel" | "php-laravel" => Some(Self::Infection),
            _ => None,
        }
    }

    fn build_command(self, full_mutation: bool) -> SafeCommand {
        match self {
            Self::CargoMutants => cargo_mutants::build_command(full_mutation),
            Self::Stryker => stryker::build_command(full_mutation),
            Self::Mutmut => mutmut::build_command(full_mutation),
            Self::Infection => infection::build_command(full_mutation),
        }
    }

    fn parse_score(self, stdout: &str) -> Option<f64> {
        match self {
            Self::CargoMutants => cargo_mutants::parse_score(stdout),
            Self::Stryker => stryker::parse_score(stdout),
            Self::Mutmut => mutmut::parse_score(stdout),
            Self::Infection => infection::parse_score(stdout),
        }
    }
}

/// Shared best-effort score parser: percentages, ratios, and decimal scores in `[0,1]`.
pub(crate) fn parse_score_best_effort(stdout: &str) -> Option<f64> {
    // Prefer explicit percent tokens.
    if let Some(p) = find_percent(stdout) {
        return Some(if p > 1.0 { p / 100.0 } else { p });
    }
    // Ratio a/b
    if let Some(r) = find_ratio(stdout) {
        return Some(r);
    }
    // Decimal 0.xx labeled score
    find_decimal_score(stdout)
}

fn find_percent(s: &str) -> Option<f64> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            let num_str = &s[start..i];
            // skip spaces
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'%' {
                if let Ok(v) = num_str.parse::<f64>() {
                    return Some(v);
                }
            }
        } else {
            i += 1;
        }
    }
    None
}

fn find_ratio(s: &str) -> Option<f64> {
    // first "N/M" where M > 0
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let a_str = &s[start..i];
            if i < bytes.len() && bytes[i] == b'/' {
                i += 1;
                let b_start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if i > b_start {
                    let b_str = &s[b_start..i];
                    if let (Ok(a), Ok(b)) = (a_str.parse::<f64>(), b_str.parse::<f64>()) {
                        if b > 0.0 {
                            return Some(a / b);
                        }
                    }
                }
            }
        } else {
            i += 1;
        }
    }
    None
}

fn find_decimal_score(s: &str) -> Option<f64> {
    let lower = s.to_ascii_lowercase();
    for key in ["mutation_score", "mutation score", "score"] {
        if let Some(idx) = lower.find(key) {
            let after = &s[idx + key.len()..];
            let trimmed = after.trim_start_matches(|c: char| {
                c == ':' || c == '=' || c.is_whitespace()
            });
            let end = trimmed
                .find(|c: char| !(c.is_ascii_digit() || c == '.'))
                .unwrap_or(trimmed.len());
            let num = &trimmed[..end];
            if let Ok(v) = num.parse::<f64>() {
                if (0.0..=1.0).contains(&v) {
                    return Some(v);
                }
                if (1.0..=100.0).contains(&v) {
                    return Some(v / 100.0);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::MockProcessRunner;

    #[test]
    fn threshold_boundary() {
        let pass = run_mutation(
            "rust",
            false,
            Some("Mutation Score Indicator (MSI): 70%\n"),
        );
        assert_eq!(pass.status, AspectStatus::Pass);
        assert_eq!(pass.aspect, AdvancedAspect::Mutation);
        assert!(pass.score.unwrap() >= MUTATION_THRESHOLD);

        let fail = run_mutation("rust", false, Some("Mutation score: 69%\n"));
        assert_eq!(fail.status, AspectStatus::Fail);
        assert_eq!(fail.reason.as_deref(), Some("below_threshold"));
        assert!(fail.score.unwrap() < MUTATION_THRESHOLD);
    }

    #[test]
    fn missing_tool_skip() {
        let mock = MockProcessRunner::new();
        mock.push_err(CoreError::not_found("executable not found"));
        let r = run_mutation_with("rust-axum", false, None, &mock);
        assert_eq!(r.status, AspectStatus::Skipped);
        assert_eq!(r.reason.as_deref(), Some("tool_missing"));
        assert!(r.score.is_none());
    }

    #[test]
    fn missing_tool_full_fail() {
        let mock = MockProcessRunner::new();
        mock.push_err(CoreError::not_found("executable not found"));
        let r = run_mutation_with("python", true, None, &mock);
        assert_eq!(r.status, AspectStatus::Fail);
        assert_eq!(r.reason.as_deref(), Some(MSG_MUTATION_MISSING));
        assert!(r.reason.as_deref().unwrap().contains("MUTATION_TOOL_MISSING"));
    }

    #[test]
    fn parse_score_sample() {
        let cargo = cargo_mutants::parse_score(
            "42 mutants tested: 28 caught, 12 missed, 2 unviable\n",
        )
        .unwrap();
        assert!((cargo - 28.0 / 40.0).abs() < 1e-9);

        let stryker_score = stryker::parse_score("Mutation score: 91%\n").unwrap();
        assert!((stryker_score - 0.91).abs() < 1e-9);

        let mutmut_score =
            mutmut::parse_score("Killed 7 out of 10 mutants\n").unwrap();
        assert!((mutmut_score - 0.7).abs() < 1e-9);

        let infection_score =
            infection::parse_score("Mutation Score Indicator (MSI): 55%\n").unwrap();
        assert!((infection_score - 0.55).abs() < 1e-9);

        // End-to-end via fixture → pass at threshold via mutmut sample
        let r = run_mutation("fastapi", false, Some("Killed 7 out of 10 mutants\n"));
        assert_eq!(r.status, AspectStatus::Pass);
        assert!((r.score.unwrap() - 0.7).abs() < 1e-9);
    }

    #[test]
    fn stack_dispatch_builds_safe_commands() {
        assert_eq!(
            MutationFamily::from_stack("rust")
                .unwrap()
                .build_command(true)
                .program(),
            "cargo-mutants"
        );
        assert_eq!(
            MutationFamily::from_stack("react")
                .unwrap()
                .build_command(false)
                .program(),
            "stryker"
        );
        assert_eq!(
            MutationFamily::from_stack("laravel")
                .unwrap()
                .build_command(false)
                .program(),
            "infection"
        );
    }
}
