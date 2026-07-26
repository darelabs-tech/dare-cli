//! Fail-to-pass aspect: expected formerly-failing tests must now pass.

use crate::report::{AdvancedAspect, AspectResult, AspectStatus};

/// Evaluate fail-to-pass given an optional list of test ids and combined harness output.
///
/// # Pass heuristics (substring, documented)
///
/// For each `id`, at least one line in `combined_output` must contain `id` **and**
/// a pass marker. A line is treated as a pass hit when (case-sensitive unless noted):
/// - it contains ` ok` / ends with `ok` after the id region (cargo-style `test … ok`);
/// - it contains `PASSED` or `passed`;
/// - it contains `✓` or a `PASS ` token (Jest/Vitest-ish).
///
/// Missing / empty `test_ids` → [`AspectStatus::Skipped`] with reason `no_ftp_list`.
pub fn check_fail_to_pass(
    test_ids: Option<&[String]>,
    combined_output: &str,
) -> AspectResult {
    let Some(ids) = test_ids.filter(|s| !s.is_empty()) else {
        return AspectResult {
            aspect: AdvancedAspect::FailToPass,
            status: AspectStatus::Skipped,
            score: None,
            reason: Some("no_ftp_list".into()),
            exit_code: None,
            duration_ms: 0,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
        };
    };

    let mut missing = Vec::new();
    for id in ids {
        if !id_appears_passed(combined_output, id) {
            missing.push(id.clone());
        }
    }

    if missing.is_empty() {
        AspectResult {
            aspect: AdvancedAspect::FailToPass,
            status: AspectStatus::Pass,
            score: Some(1.0),
            reason: None,
            exit_code: Some(0),
            duration_ms: 0,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
        }
    } else {
        AspectResult {
            aspect: AdvancedAspect::FailToPass,
            status: AspectStatus::Fail,
            score: Some((ids.len() - missing.len()) as f64 / ids.len() as f64),
            reason: Some(format!("ftp_not_passed:{}", missing.join(","))),
            exit_code: Some(1),
            duration_ms: 0,
            stdout_tail: String::new(),
            stderr_tail: missing.join("\n"),
        }
    }
}

fn id_appears_passed(combined: &str, id: &str) -> bool {
    for line in combined.lines() {
        if !line.contains(id) {
            continue;
        }
        if line_has_pass_marker(line) {
            return true;
        }
    }
    false
}

fn line_has_pass_marker(line: &str) -> bool {
    if line.contains("PASSED") || line.contains("passed") {
        return true;
    }
    if line.contains('✓') {
        return true;
    }
    if line.contains("PASS ") || line.starts_with("PASS ") {
        return true;
    }
    // cargo: `test path::name ... ok` — require ` ok` or trailing `ok`
    if line.contains(" ok") || line.trim_end().ends_with(" ok") {
        return true;
    }
    let trimmed = line.trim_end();
    trimmed.ends_with("ok") && (trimmed.ends_with(" ok") || trimmed.contains(" ... ok"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ftp_all_pass() {
        let ids = vec![
            "suite::was_failing".to_string(),
            "another_test".to_string(),
        ];
        let out = "\
running 2 tests
test suite::was_failing ... ok
test another_test ... ok
";
        let r = check_fail_to_pass(Some(&ids), out);
        assert_eq!(r.aspect, AdvancedAspect::FailToPass);
        assert_eq!(r.status, AspectStatus::Pass);
        assert_eq!(r.score, Some(1.0));
        assert!(r.reason.is_none());
    }

    #[test]
    fn ftp_missing_skipped() {
        let r = check_fail_to_pass(None, "test foo ... ok\n");
        assert_eq!(r.status, AspectStatus::Skipped);
        assert_eq!(r.reason.as_deref(), Some("no_ftp_list"));

        let empty: &[String] = &[];
        let r2 = check_fail_to_pass(Some(empty), "test foo ... ok\n");
        assert_eq!(r2.status, AspectStatus::Skipped);
        assert_eq!(r2.reason.as_deref(), Some("no_ftp_list"));
    }

    #[test]
    fn ftp_partial_fail() {
        let ids = vec!["a".to_string(), "b".to_string()];
        let out = "test a ... ok\ntest b ... FAILED\n";
        let r = check_fail_to_pass(Some(&ids), out);
        assert_eq!(r.status, AspectStatus::Fail);
        assert!(r.reason.as_deref().unwrap_or("").contains("b"));
    }
}
