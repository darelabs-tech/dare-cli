//! Review report and fail policy.

use serde::{Deserialize, Serialize};

use crate::types::{FailOn, Finding, Severity};
use crate::REPORT_SCHEMA;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewReport {
    pub schema_version: u32,
    pub task_id: String,
    pub ok: bool,
    pub error_count: u32,
    pub warning_count: u32,
    pub strict: bool,
    pub fail_on: String,
    pub enriched: bool,
    pub files_scanned: u32,
    pub findings: Vec<Finding>,
    pub unmet_criteria: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_markdown: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

pub fn compute_ok(error_count: u32, warning_count: u32, strict: bool, unmet: &[String]) -> bool {
    error_count == 0 && unmet.is_empty() && (!strict || warning_count == 0)
}

pub fn should_fail_exit(report: &ReviewReport, fail_on: FailOn) -> bool {
    if fail_on == FailOn::Never {
        return false;
    }
    if !report.unmet_criteria.is_empty() {
        return true;
    }
    if report.strict && report.warning_count > 0 {
        return true;
    }
    match fail_on {
        FailOn::Error => report.error_count > 0,
        FailOn::Warning => report.error_count + report.warning_count > 0,
        FailOn::Never => false,
    }
}

pub fn count_severities(findings: &[Finding]) -> (u32, u32) {
    let mut errors = 0u32;
    let mut warnings = 0u32;
    for f in findings {
        match f.severity {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
        }
    }
    (errors, warnings)
}

pub fn sort_findings(findings: &mut [Finding]) {
    findings.sort_by(|a, b| {
        (&a.path, a.line, a.col, &a.rule_id).cmp(&(&b.path, b.line, b.col, &b.rule_id))
    });
}

#[allow(clippy::too_many_arguments)]
pub fn new_report(
    task_id: &str,
    findings: Vec<Finding>,
    unmet: Vec<String>,
    strict: bool,
    fail_on: FailOn,
    enriched: bool,
    files_scanned: u32,
    notes: Option<String>,
    comment: bool,
) -> ReviewReport {
    let (error_count, warning_count) = count_severities(&findings);
    let ok = compute_ok(error_count, warning_count, strict, &unmet);
    let comment_markdown = if comment {
        Some(build_comment_markdown(
            task_id,
            ok,
            error_count,
            warning_count,
            &findings,
            &unmet,
        ))
    } else {
        None
    };
    ReviewReport {
        schema_version: REPORT_SCHEMA,
        task_id: task_id.to_string(),
        ok,
        error_count,
        warning_count,
        strict,
        fail_on: fail_on.as_str().to_string(),
        enriched,
        files_scanned,
        findings,
        unmet_criteria: unmet,
        comment_markdown,
        notes,
    }
}

fn build_comment_markdown(
    task_id: &str,
    ok: bool,
    errors: u32,
    warnings: u32,
    findings: &[Finding],
    unmet: &[String],
) -> String {
    let mut s = String::new();
    s.push_str("## DARE review\n\n");
    s.push_str(&format!(
        "**Task:** `{task_id}` — {}\n\n",
        if ok { "PASSED" } else { "FAILED" }
    ));
    s.push_str(&format!("Errors: {errors} · Warnings: {warnings}\n\n"));
    if !unmet.is_empty() {
        s.push_str("### Unmet criteria\n\n");
        for u in unmet {
            s.push_str(&format!("- {u}\n"));
        }
        s.push('\n');
    }
    if !findings.is_empty() {
        s.push_str("### Findings\n\n");
        for f in findings.iter().take(50) {
            s.push_str(&format!(
                "- `{}:{}` [{}] {}: {}\n",
                f.path,
                f.line,
                f.severity.as_str(),
                f.rule_id,
                f.message
            ));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Severity;

    #[test]
    fn strict_fails_on_warning() {
        assert!(!compute_ok(0, 1, true, &[]));
        assert!(compute_ok(0, 1, false, &[]));
    }

    #[test]
    fn fail_on_never_exits_ok_policy() {
        let report = new_report(
            "t1",
            vec![Finding {
                path: "a.rs".into(),
                line: 1,
                col: 1,
                severity: Severity::Error,
                rule_id: "todo_marker".into(),
                message: "x".into(),
            }],
            vec![],
            false,
            FailOn::Never,
            false,
            1,
            None,
            false,
        );
        assert!(!report.ok);
        assert!(!should_fail_exit(&report, FailOn::Never));
        assert!(should_fail_exit(&report, FailOn::Error));
    }

    #[test]
    fn deterministic_sort() {
        let mut f = vec![
            Finding {
                path: "b.rs".into(),
                line: 1,
                col: 1,
                severity: Severity::Error,
                rule_id: "a".into(),
                message: "m".into(),
            },
            Finding {
                path: "a.rs".into(),
                line: 2,
                col: 1,
                severity: Severity::Error,
                rule_id: "a".into(),
                message: "m".into(),
            },
            Finding {
                path: "a.rs".into(),
                line: 1,
                col: 1,
                severity: Severity::Error,
                rule_id: "a".into(),
                message: "m".into(),
            },
        ];
        sort_findings(&mut f);
        assert_eq!(f[0].path, "a.rs");
        assert_eq!(f[0].line, 1);
        assert_eq!(f[1].line, 2);
        assert_eq!(f[2].path, "b.rs");
    }
}
