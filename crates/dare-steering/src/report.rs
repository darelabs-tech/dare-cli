//! Steering list/show JSON report structs (camelCase).

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteeringListItem {
    pub path: String,
    pub scope: String,
    pub glob: Option<String>,
    pub priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteeringListReport {
    pub schema_version: u32,
    pub files: Vec<SteeringListItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteeringBlock {
    pub path: String,
    pub scope: String,
    pub glob: Option<String>,
    pub priority: i32,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteeringShowReport {
    pub schema_version: u32,
    pub target: String,
    pub blocks: Vec<SteeringBlock>,
}
