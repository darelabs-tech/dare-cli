//! Relative performance regression gate (DESIGN Analyst / T-08).

/// Max allowed ratio above baseline (`0.15` = 15%).
pub const PERF_REGRESSION_MAX: f64 = 0.15;

/// Returns `true` when `measured` is within `baseline * (1.0 + max_ratio)`.
///
/// When `baseline == 0.0`: both zero is ok; any positive `measured` fails.
pub fn within_regression(baseline: f64, measured: f64, max_ratio: f64) -> bool {
    if baseline == 0.0 {
        return measured == 0.0;
    }
    measured <= baseline * (1.0 + max_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regression_over_15_percent_fails() {
        assert!(!within_regression(100.0, 116.0, PERF_REGRESSION_MAX));
        // 100 * 1.15 is not exactly 115.0 in IEEE754; stay clearly under the gate.
        assert!(within_regression(100.0, 114.0, PERF_REGRESSION_MAX));
        assert!(within_regression(0.0, 0.0, PERF_REGRESSION_MAX));
        assert!(!within_regression(0.0, 1.0, PERF_REGRESSION_MAX));
    }
}
