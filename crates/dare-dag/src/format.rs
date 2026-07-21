//! Human and JSON formatting for ValidationReport.

use dare_core::{CoreError, CoreResult};
use serde_json::Value;

use crate::report::{IssueSeverity, ValidationReport};

pub fn format_human(report: &ValidationReport) -> String {
    let mut out = String::new();
    if report.ok {
        out.push_str("validate: ok\n");
    } else {
        out.push_str("validate: FAILED\n");
    }
    out.push_str(&format!("dagPath: {}\n", report.dag_path));
    out.push_str(&format!("format: {}\n", report.format));
    out.push_str(&format!("taskCount: {}\n", report.task_count));
    out.push_str(&format!("errorCount: {}\n", report.error_count));
    out.push_str(&format!("warningCount: {}\n", report.warning_count));
    out.push_str(&format!("strict: {}\n", report.strict));
    if !report.issues.is_empty() {
        out.push_str("issues:\n");
        for i in &report.issues {
            let sev = match i.severity {
                IssueSeverity::Error => "error",
                IssueSeverity::Warning => "warning",
            };
            let tid = if i.task_id.is_empty() {
                String::new()
            } else {
                format!(" {}", i.task_id)
            };
            out.push_str(&format!("  - [{sev}] {}{tid}: {}\n", i.code, i.message));
        }
    }
    out.push_str("mode: validate (zero mutations)\n");
    out
}

pub fn report_to_json(report: &ValidationReport) -> CoreResult<Value> {
    serde_json::to_value(report).map_err(|e| CoreError::internal(e.to_string()))
}
