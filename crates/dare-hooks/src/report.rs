//! Hooks list/validate/run JSON report structs (camelCase).

use serde::Serialize;

pub const HOOKS_LIST_SCHEMA: u32 = 1;
pub const HOOKS_VALIDATE_SCHEMA: u32 = 1;
pub const HOOKS_RUN_SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookListItem {
    pub event: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HooksListReport {
    pub schema_version: u32,
    pub project_root: String,
    pub trusted: bool,
    pub enabled: bool,
    pub source: String,
    pub hooks: Vec<HookListItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HooksValidateReport {
    pub schema_version: u32,
    pub ok: bool,
    pub source: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookActionResult {
    pub action: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub skipped: bool,
    pub reason: Option<String>,
    pub idempotency_key: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HooksRunReport {
    pub schema_version: u32,
    pub event: String,
    pub file: Option<String>,
    pub task: Option<String>,
    pub trusted: bool,
    pub results: Vec<HookActionResult>,
}
