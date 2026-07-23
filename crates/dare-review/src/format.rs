//! Output formatters.

use dare_core::{to_canonical_json_string, CoreResult};
use serde_json::Value;

use crate::report::ReviewReport;
use crate::types::Severity;
use crate::{MSG_FAIL, MSG_PASS};

pub fn format_human(report: &ReviewReport, errors_only: bool) -> String {
    let mut out = String::new();
    let header = if report.ok { MSG_PASS } else { MSG_FAIL };
    out.push_str(header);
    out.push('\n');
    out.push_str(&format!(
        "task={} files={} errors={} warnings={} strict={} failOn={}\n",
        report.task_id,
        report.files_scanned,
        report.error_count,
        report.warning_count,
        report.strict,
        report.fail_on
    ));
    for f in &report.findings {
        if errors_only && f.severity != Severity::Error {
            continue;
        }
        out.push_str(&format!(
            "{}:{}:{} [{}] {}: {}\n",
            f.path,
            f.line,
            f.col,
            f.severity.as_str(),
            f.rule_id,
            f.message
        ));
    }
    if !report.unmet_criteria.is_empty() {
        out.push_str("unmetCriteria:\n");
        for u in &report.unmet_criteria {
            out.push_str(&format!("- {u}\n"));
        }
    }
    if let Some(c) = &report.comment_markdown {
        out.push('\n');
        out.push_str(c);
        if !c.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

pub fn format_github(report: &ReviewReport, errors_only: bool) -> String {
    let mut out = String::new();
    for f in &report.findings {
        if errors_only && f.severity != Severity::Error {
            continue;
        }
        // GitHub Actions workflow command
        out.push_str(&format!(
            "::{} file={},line={}::[{}] {}\n",
            f.severity.github_token(),
            f.path,
            f.line,
            f.rule_id,
            escape_github_msg(&f.message)
        ));
    }
    for u in &report.unmet_criteria {
        out.push_str(&format!("::error ::[semantic] {}\n", escape_github_msg(u)));
    }
    if out.is_empty() {
        out.push_str(&format!(
            "::notice ::DARE review {} for {}\n",
            if report.ok { "passed" } else { "failed" },
            report.task_id
        ));
    }
    out
}

fn escape_github_msg(s: &str) -> String {
    s.replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

pub fn report_to_json(report: &ReviewReport) -> CoreResult<Value> {
    let s =
        serde_json::to_value(report).map_err(|e| dare_core::CoreError::internal(e.to_string()))?;
    // Ensure canonical key order when stringified by callers
    let _ = to_canonical_json_string(&s)?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::new_report;
    use crate::types::{FailOn, Finding, Severity};

    #[test]
    fn github_format_prefix() {
        let report = new_report(
            "t1",
            vec![Finding {
                path: "src/a.rs".into(),
                line: 4,
                col: 2,
                severity: Severity::Error,
                rule_id: "todo_marker".into(),
                message: "forbidden marker `TODO`".into(),
            }],
            vec![],
            false,
            FailOn::Error,
            false,
            1,
            None,
            false,
        );
        let g = format_github(&report, false);
        assert!(g.starts_with("::error file=src/a.rs,line=4::"));
        assert!(g.contains("todo_marker"));
    }
}
