//! GraphRAG storage layer (microplano 040).
//!
//! Backends: SQLite (`.dare/graph.db`) and JSON (`.dare/graph.json`).
//! Search / ingest / RRF / Neo4j are out of scope (041–043).

mod config;
mod ids;
mod knowledge_graph;
mod migrations;
mod storage;
mod types;
mod vector;

pub use config::{
    load_graph_config, open_graph, GraphBackend, GraphConfig, GraphHandle, GRAPH_DB_REL,
    GRAPH_JSON_REL, GRAPH_YML_REL,
};
pub use ids::{
    canonical_code_symbol_node_id, canonical_edge_id, canonical_file_node_id,
    canonical_pattern_node_id, canonical_requirement_node_id, canonical_task_node_id,
    normalize_graph_path, to_qualified_name,
};
pub use knowledge_graph::{EdgeDirection, KnowledgeGraph};
pub use migrations::{detect_sqlite_schema_version, CURRENT_SCHEMA_VERSION, SCHEMA_SQL};
pub use storage::{JsonGraph, SqliteGraph};
pub use types::{
    empty_edges_by_type, empty_nodes_by_type, EdgeType, GraphEdge, GraphNode, GraphStatistics,
    GraphStoreDocument, NodeType, VectorRow, ALL_EDGE_TYPES, ALL_NODE_TYPES,
};
pub use vector::{deserialize_f32_le, serialize_f32_le};
