//! Diff report JSON (`schemaVersion` 1) for golden suite runs.

use serde::{Deserialize, Serialize};

use crate::axis::CompareAxis;
use crate::case::DiffClass;

/// Required `schemaVersion` for [`DiffReport`].
pub const REPORT_SCHEMA_VERSION: u32 = 1;

/// Per-case outcome status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaseStatus {
    Pass,
    Fail,
    Skip,
}

/// One case row in a [`DiffReport`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseResult {
    pub id: String,
    pub status: CaseStatus,
    #[serde(rename = "failedAxes", default)]
    pub failed_axes: Vec<CompareAxis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<DiffClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Aggregate counts for a suite run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffSummary {
    pub pass: u32,
    pub fail: u32,
    pub skip: u32,
}

/// Suite-level diff report (`schemaVersion` 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffReport {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
    pub cases: Vec<CaseResult>,
    pub summary: DiffSummary,
}

impl DiffReport {
    /// Build a report from case results, recomputing summary counts.
    pub fn from_cases(generated_at: impl Into<String>, cases: Vec<CaseResult>) -> Self {
        let mut summary = DiffSummary::default();
        for c in &cases {
            match c.status {
                CaseStatus::Pass => summary.pass += 1,
                CaseStatus::Fail => summary.fail += 1,
                CaseStatus::Skip => summary.skip += 1,
            }
        }
        Self {
            schema_version: REPORT_SCHEMA_VERSION,
            generated_at: generated_at.into(),
            cases,
            summary,
        }
    }

    /// Serialize to pretty JSON.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
