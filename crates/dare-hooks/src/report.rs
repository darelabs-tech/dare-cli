//! Hooks list/validate JSON report structs (camelCase).

use serde::Serialize;

pub const HOOKS_LIST_SCHEMA: u32 = 1;
pub const HOOKS_VALIDATE_SCHEMA: u32 = 1;

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
