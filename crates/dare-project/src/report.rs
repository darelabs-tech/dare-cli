//! Detection report schema (schemaVersion 1 — frozen).

use serde::{Deserialize, Serialize};

/// Frozen JSON schema version for `DetectionReport`.
pub const DETECTION_SCHEMA_VERSION: u32 = 1;

/// Max bytes read from manifests (Cargo.toml workspace scan, etc.).
pub const MANIFEST_READ_CAP: usize = 262_144;

/// Max directory depth when scanning child manifests for monorepo.
pub const MONOREPO_MAX_DEPTH: usize = 3;

/// Max directories visited during monorepo child walk.
pub const MONOREPO_MAX_ENTRIES: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionReport {
    pub schema_version: u32,
    pub mode: String,
    pub project_root: Option<String>,
    pub git_root: Option<String>,
    pub stacks: Vec<StackHit>,
    pub conflicts: Vec<StackConflict>,
    pub monorepo: bool,
    pub monorepo_evidence: Vec<String>,
    pub harnesses: Vec<HarnessHit>,
    pub dare_already_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackHit {
    pub id: String,
    pub family: String,
    pub confidence: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackConflict {
    pub kinds: Vec<String>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessHit {
    pub id: String,
    pub present: bool,
    pub evidence: Vec<String>,
}

impl DetectionReport {
    /// Empty / no-root check report (harnesses filled by caller or empty).
    pub fn empty_check(git_root: Option<String>, harnesses: Vec<HarnessHit>) -> Self {
        Self {
            schema_version: DETECTION_SCHEMA_VERSION,
            mode: "check".to_string(),
            project_root: None,
            git_root,
            stacks: Vec::new(),
            conflicts: Vec::new(),
            monorepo: false,
            monorepo_evidence: Vec::new(),
            harnesses,
            dare_already_present: false,
        }
    }
}
