//! Validation report types for `dare validate` (microplano 020).

use serde::{Deserialize, Serialize};

pub const VALIDATION_SCHEMA_VERSION: u32 = 1;
pub const MSG_MAX: usize = 200;
pub const DEFAULT_DAG_REL: &str = "DARE/dare-dag.yaml";
pub const COMPLEXITY_ALLOWED: &[&str] = &["LOW", "MED", "HIGH"];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ValidateOptions {
    pub strict: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub code: String,
    pub task_id: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub path: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub schema_version: u32,
    pub mode: String,
    pub ok: bool,
    pub dag_path: String,
    pub format: String,
    pub task_count: u32,
    pub error_count: u32,
    pub warning_count: u32,
    pub strict: bool,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn compute_ok(error_count: u32, warning_count: u32, strict: bool) -> bool {
        error_count == 0 && (!strict || warning_count == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn report_schema_version_1() {
        let report = ValidationReport {
            schema_version: VALIDATION_SCHEMA_VERSION,
            mode: "validate".into(),
            ok: true,
            dag_path: DEFAULT_DAG_REL.into(),
            format: "v2.1".into(),
            task_count: 0,
            error_count: 0,
            warning_count: 0,
            strict: false,
            issues: vec![],
        };
        let v: Value = serde_json::to_value(&report).expect("json");
        assert_eq!(v["schemaVersion"], 1);
        assert_eq!(v["mode"], "validate");
        assert_eq!(v["dagPath"], DEFAULT_DAG_REL);
        assert_eq!(v["taskCount"], 0);
        assert_eq!(v["errorCount"], 0);
        assert_eq!(v["warningCount"], 0);
        assert!(v["issues"].is_array());
        let round: ValidationReport = serde_json::from_value(v).expect("from");
        assert_eq!(round.schema_version, 1);
        assert_eq!(round.mode, "validate");
    }
}
