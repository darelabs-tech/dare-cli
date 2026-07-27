//! Bench Fix·Rate, suite means, and report schema (schemaVersion 1).

pub mod baseline;
pub mod run;
pub mod suite;

use serde::{Deserialize, Serialize};

pub use baseline::{
    compare_baseline, compute_drop_pp, load_baseline, BaselineComparison, BaselineFile,
    MSG_BASELINE_INVALID,
};
pub use run::{run_bench, BenchOptions, BENCH_REPORT_SCHEMA};
pub use suite::{load_suite, LoadedCase, LoadedSuite, SuiteCase, SuiteFile, MSG_SUITE_INVALID};

/// Default relative suite directory (`fixtures/bench`).
pub const DEFAULT_SUITE_REL: &str = "fixtures/bench";

/// Per-fixture result inside a [`BenchReport`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FixtureResult {
    pub id: String,
    pub ok: bool,
    pub fix_rate: f64,
    pub fail_to_pass_total: u32,
    pub fail_to_pass_passed: u32,
    pub pass_to_pass_failed: u32,
}

/// Bench suite report (`schemaVersion` 1, camelCase JSON).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchReport {
    pub schema_version: u32,
    pub suite_path: String,
    pub fix_rate: f64,
    pub solve_rate: f64,
    pub fixtures: Vec<FixtureResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline: Option<BaselineComparison>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

/// Pure Fix·Rate for one fixture (§0.4 / composed prompt).
///
/// - `ptp_failed > 0` → `0.0`
/// - else if `ftp_total == 0` → `1.0`
/// - else → `ftp_passed / ftp_total`
pub fn compute_fixture_fix_rate(ftp_total: u32, ftp_passed: u32, ptp_failed: u32) -> f64 {
    if ptp_failed > 0 {
        0.0
    } else if ftp_total == 0 {
        1.0
    } else {
        f64::from(ftp_passed) / f64::from(ftp_total)
    }
}

/// Arithmetic mean of per-fixture Fix·Rates (internal f64; round only on serialize).
pub fn compute_suite_fix_rate(rates: &[f64]) -> f64 {
    if rates.is_empty() {
        return 0.0;
    }
    rates.iter().sum::<f64>() / rates.len() as f64
}

/// `solveRate = (count fixtureOk) / (count fixtures)`.
pub fn compute_solve_rate(ok_count: usize, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    ok_count as f64 / total as f64
}

/// Round to 4 decimal places with half-up (for positive rates: Rust `round` = half away from 0).
pub fn round_4dp(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixrate_ptp_zero() {
        assert_eq!(compute_fixture_fix_rate(4, 4, 1), 0.0);
        assert_eq!(compute_fixture_fix_rate(0, 0, 2), 0.0);
        assert_eq!(compute_fixture_fix_rate(2, 1, 0), 0.5);
        assert_eq!(compute_fixture_fix_rate(0, 0, 0), 1.0);
    }

    #[test]
    fn fixrate_mean() {
        let rates = [1.0, 0.5, 0.0];
        let mean = compute_suite_fix_rate(&rates);
        assert!((mean - 0.5).abs() < f64::EPSILON);
        assert_eq!(round_4dp(1.0 / 3.0), 0.3333);
        assert_eq!(round_4dp(0.12345), 0.1235);
        assert_eq!(round_4dp(0.12344), 0.1234);
        assert_eq!(compute_solve_rate(1, 2), 0.5);
        assert_eq!(compute_solve_rate(0, 0), 0.0);
    }

    #[test]
    fn bench_report_camel_case_json() {
        let report = BenchReport {
            schema_version: 1,
            suite_path: "fixtures/bench".to_string(),
            fix_rate: round_4dp(0.5),
            solve_rate: round_4dp(0.5),
            fixtures: vec![FixtureResult {
                id: "sample-ok".to_string(),
                ok: true,
                fix_rate: 1.0,
                fail_to_pass_total: 2,
                fail_to_pass_passed: 2,
                pass_to_pass_failed: 0,
            }],
            baseline: None,
            filter: None,
        };
        let v = serde_json::to_value(&report).expect("serialize");
        assert_eq!(v["schemaVersion"], 1);
        assert_eq!(v["suitePath"], "fixtures/bench");
        assert_eq!(v["fixRate"], 0.5);
        assert_eq!(v["solveRate"], 0.5);
        assert_eq!(v["fixtures"][0]["failToPassTotal"], 2);
        assert_eq!(v["fixtures"][0]["passToPassFailed"], 0);
        assert!(v.get("baseline").is_none());
    }
}
