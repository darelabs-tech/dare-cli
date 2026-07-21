//! `dare-graph.yml` document model (not SQLite).

use dare_core::CoreResult;
use dare_core::{ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::io::{from_yaml_str, read_limited, write_yaml_atomic};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub metadata: Map<String, Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphEdge {
    pub id: String,
    #[serde(alias = "sourceId")]
    pub source_id: String,
    #[serde(alias = "targetId")]
    pub target_id: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    #[serde(default)]
    pub weight: f64,
    #[serde(default)]
    pub metadata: Map<String, Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GraphDocument {
    #[serde(default)]
    pub nodes: Vec<GraphNode>,
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub fn canonical_task_node_id(task_id: &str) -> String {
    format!("task:{task_id}")
}

pub fn canonical_file_node_id(posix_path: &str) -> String {
    format!("file:{posix_path}")
}

pub fn canonical_edge_id(kind: &str, from: &str, to: &str) -> String {
    format!("{kind}:{from}->{to}")
}

pub fn load_graph(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<GraphDocument> {
    let bytes = read_limited(root, rel)?;
    let text = String::from_utf8(bytes).map_err(|e| dare_core::CoreError::config(e.to_string()))?;
    from_yaml_str(&text)
}

pub fn save_graph(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    doc: &GraphDocument,
) -> CoreResult<()> {
    write_yaml_atomic(root, rel, doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_edge_id_format() {
        assert_eq!(
            canonical_edge_id("depends_on", "task:a", "task:b"),
            "depends_on:task:a->task:b"
        );
    }
}
