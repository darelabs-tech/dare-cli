//! Frozen scaffold types (BLUEPRINT-046 §0.4).

use serde::{Deserialize, Serialize};

/// Schema version for plan / apply report payloads.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StackKind {
    Backend,
    Mcp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Toolchain {
    #[default]
    None,
    Docker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Transport {
    #[default]
    Stdio,
    Http,
    Sse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FrontendKind {
    React,
    Vue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConflictPolicy {
    #[default]
    FailFast,
    SkipExisting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackMetadata {
    pub id: String,
    pub kind: StackKind,
    pub language: String,
    pub default_toolchain: Toolchain,
    pub default_transport: Option<Transport>,
    pub template_root: String,
    pub rate_limit_rel: String,
}

#[derive(Debug, Clone)]
pub struct ScaffoldRequest {
    pub project_name: String,
    pub stack_id: String,
    pub toolchain: Toolchain,
    pub transport: Option<Transport>,
    pub frontend: Option<FrontendKind>,
    pub conflict_policy: ConflictPolicy,
    pub force: bool,
    pub check: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlanAction {
    Create,
    Skip,
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlanItemKind {
    Template,
    Ax,
    Meta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldPlanItem {
    pub path: String,
    pub action: PlanAction,
    pub kind: PlanItemKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldPlan {
    pub schema_version: u32,
    pub stack_id: String,
    pub project_name: String,
    pub frontend: Option<FrontendKind>,
    pub items: Vec<ScaffoldPlanItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldApplyReport {
    pub schema_version: u32,
    pub stack_id: String,
    pub created: Vec<String>,
    pub replaced: Vec<String>,
    pub skipped: Vec<String>,
    pub rolled_back: bool,
    pub check: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationReport {
    pub stack_id: String,
    pub ok: bool,
    pub missing: Vec<String>,
    pub secret_hits: Vec<String>,
}
