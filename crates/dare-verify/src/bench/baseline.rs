//! Bench baseline file parse and regression drop (percentage points).

use std::fs;
use std::path::Path;

use dare_core::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

/// Usage message prefix: `invalid bench baseline: {reason}`.
pub const MSG_BASELINE_INVALID: &str = "invalid bench baseline";

fn baseline_invalid(reason: impl AsRef<str>) -> CoreError {
    CoreError::usage(format!("{MSG_BASELINE_INVALID}: {}", reason.as_ref()))
}

/// On-disk baseline JSON (`schemaVersion` 1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BaselineFile {
    pub schema_version: u32,
    pub solve_rate: f64,
    pub fix_rate: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suite_name: Option<String>,
}

/// Baseline comparison block embedded in [`super::BenchReport`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BaselineComparison {
    pub path: String,
    pub solve_rate: f64,
    pub fix_rate: f64,
    pub drop_solve_pp: f64,
    pub regression_failed: bool,
}

/// `drop_pp = (baseline.solveRate - current.solveRate) * 100.0`
pub fn compute_drop_pp(baseline_solve_rate: f64, current_solve_rate: f64) -> f64 {
    (baseline_solve_rate - current_solve_rate) * 100.0
}

/// Parse a baseline JSON file from disk.
pub fn load_baseline(path: &Path) -> CoreResult<BaselineFile> {
    let bytes = fs::read(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            baseline_invalid(format!("file not found: {}", path.display()))
        } else {
            CoreError::io(e.to_string())
        }
    })?;
    let file: BaselineFile = serde_json::from_slice(&bytes)
        .map_err(|e| baseline_invalid(format!("malformed baseline json: {e}")))?;
    if file.schema_version != 1 {
        return Err(baseline_invalid(format!(
            "unsupported schemaVersion {}",
            file.schema_version
        )));
    }
    if !(0.0..=1.0).contains(&file.solve_rate) || !(0.0..=1.0).contains(&file.fix_rate) {
        return Err(baseline_invalid(
            "solveRate and fixRate must be in [0.0, 1.0]",
        ));
    }
    Ok(file)
}

/// Build a comparison block given baseline rates and current suite rates.
pub fn compare_baseline(
    path: impl Into<String>,
    baseline: &BaselineFile,
    current_solve_rate: f64,
    _current_fix_rate: f64,
    fail_on_regression_pp: Option<u32>,
) -> BaselineComparison {
    let drop_solve_pp = compute_drop_pp(baseline.solve_rate, current_solve_rate);
    let regression_failed = fail_on_regression_pp
        .map(|n| drop_solve_pp > f64::from(n))
        .unwrap_or(false);
    BaselineComparison {
        path: path.into(),
        solve_rate: baseline.solve_rate,
        fix_rate: baseline.fix_rate,
        drop_solve_pp,
        regression_failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_drop() {
        // baseline 0.75, current 0.50 → drop 25 pp
        let drop = compute_drop_pp(0.75, 0.50);
        assert!((drop - 25.0).abs() < 1e-9);

        let baseline = BaselineFile {
            schema_version: 1,
            solve_rate: 0.75,
            fix_rate: 0.8,
            suite_name: Some("dare-bench-default".to_string()),
        };
        let cmp = compare_baseline("bench-baseline.json", &baseline, 0.5, 0.5, Some(10));
        assert!((cmp.drop_solve_pp - 25.0).abs() < 1e-9);
        assert!(cmp.regression_failed);

        let cmp_ok = compare_baseline("bench-baseline.json", &baseline, 0.7, 0.7, Some(10));
        assert!((cmp_ok.drop_solve_pp - 5.0).abs() < 1e-9);
        assert!(!cmp_ok.regression_failed);

        let v = serde_json::to_value(&cmp).expect("serialize");
        assert_eq!(v["dropSolvePp"], 25.0);
        assert_eq!(v["regressionFailed"], true);
        assert_eq!(v["solveRate"], 0.75);
    }

    #[test]
    fn baseline_invalid_schema() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("bad.json");
        fs::write(
            &path,
            r#"{"schemaVersion":2,"solveRate":0.5,"fixRate":0.5}"#,
        )
        .expect("write");
        let err = load_baseline(&path).expect_err("schema 2");
        assert!(matches!(err, CoreError::Usage(_)));
        assert!(err.message().starts_with("invalid bench baseline:"));
    }
}
