//! `DARE/dare-dag.yaml` v2.1 and legacy flat.

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

use crate::io::{read_limited, write_yaml_atomic};

fn default_parent_ctx() -> u32 {
    2000
}
fn default_task_out() -> u32 {
    4000
}
fn default_timeout() -> u32 {
    600
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DagLimits {
    #[serde(default = "default_parent_ctx")]
    pub parent_context_chars: u32,
    #[serde(default = "default_task_out")]
    pub task_output_chars: u32,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for DagLimits {
    fn default() -> Self {
        Self {
            parent_context_chars: default_parent_ctx(),
            task_output_chars: default_task_out(),
            timeout_seconds: default_timeout(),
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DagTask {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub complexity: String,
    #[serde(default)]
    pub subtask_prompt: String,
    #[serde(default)]
    pub spec_file: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DagV21 {
    pub title: String,
    pub version: String,
    #[serde(default)]
    pub limits: DagLimits,
    #[serde(default)]
    pub models: BTreeMap<String, BTreeMap<String, String>>,
    pub tasks: Vec<DagTask>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegacyTask {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub complexity: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegacyDag {
    #[serde(flatten)]
    pub tasks: BTreeMap<String, LegacyTask>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DagDocument {
    V21(DagV21),
    Legacy(LegacyDag),
}

pub fn parse_dag_yaml(text: &str) -> CoreResult<DagDocument> {
    let value: Value =
        serde_yaml::from_str(text).map_err(|e| CoreError::config(format!("invalid dare-dag.yaml: {e}")))?;
    let Value::Object(map) = &value else {
        return Err(CoreError::config("invalid dare-dag.yaml"));
    };
    if let Some(tasks) = map.get("tasks") {
        if tasks.is_array() {
            let dag: DagV21 = serde_yaml::from_str(text)
                .map_err(|e| CoreError::config(format!("invalid dare-dag.yaml: {e}")))?;
            return Ok(DagDocument::V21(dag));
        }
    }
    // Legacy: mapping of task id -> body (no tasks sequence)
    let legacy: LegacyDag = serde_yaml::from_str(text)
        .map_err(|e| CoreError::config(format!("invalid dare-dag.yaml: {e}")))?;
    if legacy.tasks.is_empty() {
        return Err(CoreError::config("invalid dare-dag.yaml"));
    }
    Ok(DagDocument::Legacy(legacy))
}

pub fn load_dag(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<DagDocument> {
    let bytes = read_limited(root, rel)?;
    let text = String::from_utf8(bytes).map_err(|e| CoreError::config(e.to_string()))?;
    parse_dag_yaml(&text)
}

pub fn save_dag(root: &ProjectRoot, rel: &SafeRelativePath, doc: &DagDocument) -> CoreResult<()> {
    match doc {
        DagDocument::V21(d) => write_yaml_atomic(root, rel, d),
        DagDocument::Legacy(d) => write_yaml_atomic(root, rel, d),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dag_v21_and_legacy() {
        let v21 = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: t1
    title: One
    complexity: LOW
"#;
        match parse_dag_yaml(v21).unwrap() {
            DagDocument::V21(d) => {
                assert_eq!(d.tasks.len(), 1);
                assert_eq!(d.tasks[0].id, "t1");
            }
            _ => panic!("expected v21"),
        }

        let legacy = r#"
task-001:
  title: Legacy
  depends_on: []
  complexity: MED
"#;
        match parse_dag_yaml(legacy).unwrap() {
            DagDocument::Legacy(d) => {
                assert!(d.tasks.contains_key("task-001"));
            }
            _ => panic!("expected legacy"),
        }
    }
}
