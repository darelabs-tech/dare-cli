//! Guard report types.

use serde::{Deserialize, Serialize};

use crate::provenance::Provenance;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GuardVerdict {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    Info,
    Warn,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub path: String,
    pub layer: String,
    pub rule_id: String,
    pub severity: FindingSeverity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardReport {
    pub schema_version: u32,
    pub verdict: GuardVerdict,
    pub findings: Vec<Finding>,
    pub scanned: usize,
}

impl GuardReport {
    pub fn new(findings: Vec<Finding>, scanned: usize) -> Self {
        let verdict = compute_verdict(&findings);
        Self {
            schema_version: 1,
            verdict,
            findings,
            scanned,
        }
    }

    pub fn is_fail(&self) -> bool {
        matches!(self.verdict, GuardVerdict::Fail)
    }

    pub fn has_warn(&self) -> bool {
        matches!(self.verdict, GuardVerdict::Warn)
            || self
                .findings
                .iter()
                .any(|f| matches!(f.severity, FindingSeverity::Warn))
    }
}

fn compute_verdict(findings: &[Finding]) -> GuardVerdict {
    let mut warn = false;
    for f in findings {
        match f.severity {
            FindingSeverity::Fail => return GuardVerdict::Fail,
            FindingSeverity::Warn => warn = true,
            FindingSeverity::Info => {}
        }
    }
    if warn {
        GuardVerdict::Warn
    } else {
        GuardVerdict::Pass
    }
}
