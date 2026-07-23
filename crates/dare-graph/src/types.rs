//! Node/edge types and store document model (parity with TS graphrag/types).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Task,
    File,
    Schema,
    Endpoint,
    Component,
    Entity,
    Concept,
    Gate,
    #[serde(rename = "code_symbol")]
    CodeSymbol,
    Requirement,
    Pattern,
    #[serde(rename = "formal-gate")]
    FormalGate,
}

impl NodeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::File => "file",
            Self::Schema => "schema",
            Self::Endpoint => "endpoint",
            Self::Component => "component",
            Self::Entity => "entity",
            Self::Concept => "concept",
            Self::Gate => "gate",
            Self::CodeSymbol => "code_symbol",
            Self::Requirement => "requirement",
            Self::Pattern => "pattern",
            Self::FormalGate => "formal-gate",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "task" => Self::Task,
            "file" => Self::File,
            "schema" => Self::Schema,
            "endpoint" => Self::Endpoint,
            "component" => Self::Component,
            "entity" => Self::Entity,
            "concept" => Self::Concept,
            "gate" => Self::Gate,
            "code_symbol" => Self::CodeSymbol,
            "requirement" => Self::Requirement,
            "pattern" => Self::Pattern,
            "formal-gate" => Self::FormalGate,
            _ => return None,
        })
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    DependsOn,
    Implements,
    Uses,
    References,
    RelatedTo,
    Contains,
    Extends,
    VerifiedBy,
    Affects,
    DerivesFrom,
    EvidencedBy,
    Exhibits,
    ProvenBy,
}

impl EdgeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DependsOn => "depends_on",
            Self::Implements => "implements",
            Self::Uses => "uses",
            Self::References => "references",
            Self::RelatedTo => "related_to",
            Self::Contains => "contains",
            Self::Extends => "extends",
            Self::VerifiedBy => "verified_by",
            Self::Affects => "affects",
            Self::DerivesFrom => "derives_from",
            Self::EvidencedBy => "evidenced_by",
            Self::Exhibits => "exhibits",
            Self::ProvenBy => "proven_by",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "depends_on" => Self::DependsOn,
            "implements" => Self::Implements,
            "uses" => Self::Uses,
            "references" => Self::References,
            "related_to" => Self::RelatedTo,
            "contains" => Self::Contains,
            "extends" => Self::Extends,
            "verified_by" => Self::VerifiedBy,
            "affects" => Self::Affects,
            "derives_from" => Self::DerivesFrom,
            "evidenced_by" => Self::EvidencedBy,
            "exhibits" => Self::Exhibits,
            "proven_by" => Self::ProvenBy,
            _ => return None,
        })
    }
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub const ALL_NODE_TYPES: &[NodeType] = &[
    NodeType::Task,
    NodeType::File,
    NodeType::Schema,
    NodeType::Endpoint,
    NodeType::Component,
    NodeType::Entity,
    NodeType::Concept,
    NodeType::Gate,
    NodeType::CodeSymbol,
    NodeType::Requirement,
    NodeType::Pattern,
    NodeType::FormalGate,
];

pub const ALL_EDGE_TYPES: &[EdgeType] = &[
    EdgeType::DependsOn,
    EdgeType::Implements,
    EdgeType::Uses,
    EdgeType::References,
    EdgeType::RelatedTo,
    EdgeType::Contains,
    EdgeType::Extends,
    EdgeType::VerifiedBy,
    EdgeType::Affects,
    EdgeType::DerivesFrom,
    EdgeType::EvidencedBy,
    EdgeType::Exhibits,
    EdgeType::ProvenBy,
];

pub fn empty_nodes_by_type() -> BTreeMap<String, u64> {
    ALL_NODE_TYPES
        .iter()
        .map(|t| (t.as_str().to_string(), 0))
        .collect()
}

pub fn empty_edges_by_type() -> BTreeMap<String, u64> {
    ALL_EDGE_TYPES
        .iter()
        .map(|t| (t.as_str().to_string(), 0))
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vec<f32>>,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub metadata: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl GraphNode {
    pub fn new(id: impl Into<String>, node_type: NodeType, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            node_type: node_type.as_str().to_string(),
            label: label.into(),
            description: None,
            vector: None,
            metadata: Map::new(),
            created_at: None,
            updated_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    #[serde(default = "default_weight")]
    pub weight: f64,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub metadata: Map<String, Value>,
}

fn default_weight() -> f64 {
    1.0
}

impl GraphEdge {
    pub fn new(
        id: impl Into<String>,
        source_id: impl Into<String>,
        target_id: impl Into<String>,
        edge_type: EdgeType,
    ) -> Self {
        Self {
            id: id.into(),
            source_id: source_id.into(),
            target_id: target_id.into(),
            edge_type: edge_type.as_str().to_string(),
            weight: 1.0,
            metadata: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GraphStoreDocument {
    #[serde(default)]
    pub nodes: Vec<GraphNode>,
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorRow {
    pub id: String,
    pub v: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GraphStatistics {
    pub total_nodes: u64,
    pub total_edges: u64,
    pub nodes_by_type: BTreeMap<String, u64>,
    pub edges_by_type: BTreeMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_node_and_edge_counts() {
        assert_eq!(ALL_NODE_TYPES.len(), 12);
        assert_eq!(ALL_EDGE_TYPES.len(), 13);
        assert_eq!(empty_nodes_by_type().len(), 12);
        assert_eq!(empty_edges_by_type().len(), 13);
    }

    #[test]
    fn formal_gate_roundtrip() {
        assert_eq!(NodeType::parse("formal-gate"), Some(NodeType::FormalGate));
        assert_eq!(NodeType::FormalGate.as_str(), "formal-gate");
    }
}
