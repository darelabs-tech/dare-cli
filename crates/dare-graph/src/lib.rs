//! GraphRAG storage + ingest/search (microplanos 040–041).
//!
//! Backends: SQLite (`.dare/graph.db`) and JSON (`.dare/graph.json`).
//! Ingest (contentHash + regex symbols), keyword LIKE, BFS, RRF k=60.
//! Semantic embeddings → 042; Neo4j → 043.

mod config;
mod ids;
mod ingest;
mod knowledge_graph;
mod migrations;
mod search;
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
pub use ingest::{
    content_hash_hex, ensure_fts5_table, extract_symbols, ingest_project, rebuild_fts5,
    IngestOptions, IngestReport, DEFAULT_MAX_FILES, DEFAULT_MAX_FILE_BYTES,
};
pub use knowledge_graph::{EdgeDirection, KnowledgeGraph};
pub use migrations::{detect_sqlite_schema_version, CURRENT_SCHEMA_VERSION, SCHEMA_SQL};
pub use search::{
    bfs_expand, hybrid_query, keyword_search, node_matches_keyword, render_mermaid_subset,
    rrf_fuse, RankedHit, SearchOptions, DEFAULT_FANOUT, DEFAULT_LIMIT, DEFAULT_MAX_HOPS,
    MAX_FANOUT_CAP, MAX_HOPS_CAP, MAX_LIMIT_CAP, RRF_K,
};
pub use storage::{JsonGraph, SqliteGraph};
pub use types::{
    empty_edges_by_type, empty_nodes_by_type, EdgeType, GraphEdge, GraphNode, GraphStatistics,
    GraphStoreDocument, NodeType, VectorRow, ALL_EDGE_TYPES, ALL_NODE_TYPES,
};
pub use vector::{deserialize_f32_le, serialize_f32_le};
