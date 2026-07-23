//! Integration: legacy SQLite open/mutate + JSON↔SQLite contracts.

use dare_core::{ProjectRoot, SafeRelativePath};
use dare_graph::{
    canonical_edge_id, canonical_task_node_id, deserialize_f32_le, open_graph, serialize_f32_le,
    EdgeDirection, EdgeType, GraphBackend, GraphConfig, GraphEdge, GraphNode, KnowledgeGraph,
    NodeType, SqliteGraph, GRAPH_DB_REL, GRAPH_JSON_REL, SCHEMA_SQL,
};
use rusqlite::Connection;
use tempfile::tempdir;

fn write_legacy_db(path: &std::path::Path) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(SCHEMA_SQL).unwrap();
    let blob = serialize_f32_le(&[0.25_f32, 0.5, 0.75]);
    conn.execute(
        "INSERT INTO nodes (id, type, label, description, vector, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            "task:legacy",
            "task",
            "Legacy Task",
            "from fixture",
            blob,
            "{}",
        ],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO edges (id, source_id, target_id, type, weight, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            "depends_on:task:legacy->task:other",
            "task:legacy",
            "task:other",
            "depends_on",
            1.0,
            "{}",
        ],
    )
    .unwrap();
}

#[test]
fn open_and_mutate_legacy_db_copy() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("legacy.db");
    write_legacy_db(&src);
    let copy = dir.path().join("copy.db");
    std::fs::copy(&src, &copy).unwrap();

    let mut g = SqliteGraph::open_path(&copy).unwrap();
    assert_eq!(g.schema_version(), 1);
    // Open must NOT create migrations table (ADR-006).
    {
        let probe = Connection::open(&copy).unwrap();
        let has_mig: bool = probe
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name='dare_schema_migrations'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        assert!(!has_mig);
    }

    let node = g.get_node("task:legacy").unwrap().expect("legacy node");
    assert_eq!(node.label, "Legacy Task");
    assert_eq!(
        node.vector.as_deref(),
        Some([0.25_f32, 0.5, 0.75].as_slice())
    );

    let mut updated = node.clone();
    updated.label = "Mutated".into();
    g.add_node(updated).unwrap();
    assert_eq!(g.get_node("task:legacy").unwrap().unwrap().label, "Mutated");

    let extra = GraphNode::new(canonical_task_node_id("new"), NodeType::Task, "New");
    g.add_node(extra).unwrap();
    g.add_edge(GraphEdge::new(
        canonical_edge_id(
            "depends_on",
            &canonical_task_node_id("legacy"),
            &canonical_task_node_id("new"),
        ),
        canonical_task_node_id("legacy"),
        canonical_task_node_id("new"),
        EdgeType::DependsOn,
    ))
    .unwrap();

    let stats = g.get_statistics().unwrap();
    assert_eq!(stats.total_nodes, 2);
    assert!(stats.total_edges >= 2);
    assert_eq!(stats.nodes_by_type.get("task").copied(), Some(2));
    assert_eq!(stats.nodes_by_type.get("file").copied(), Some(0));

    g.migrate().unwrap();
    assert_eq!(g.schema_version(), 1);
    g.close().unwrap();
}

#[test]
fn migrate_adds_vector_column_explicitly() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("v0.db");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE nodes (
              id TEXT PRIMARY KEY,
              type TEXT NOT NULL,
              label TEXT NOT NULL,
              description TEXT,
              metadata TEXT DEFAULT '{}',
              created_at TEXT DEFAULT (datetime('now')),
              updated_at TEXT DEFAULT (datetime('now'))
            );
            CREATE TABLE edges (
              id TEXT PRIMARY KEY,
              source_id TEXT NOT NULL,
              target_id TEXT NOT NULL,
              type TEXT NOT NULL,
              weight REAL DEFAULT 1.0,
              metadata TEXT DEFAULT '{}'
            );
            "#,
        )
        .unwrap();
        conn.execute(
            "INSERT INTO nodes (id, type, label) VALUES ('task:a', 'task', 'A')",
            [],
        )
        .unwrap();
    }
    let mut g = SqliteGraph::open_path(&path).unwrap();
    assert_eq!(g.schema_version(), 0);
    assert!(g.get_node("task:a").is_err());
    g.migrate().unwrap();
    assert_eq!(g.schema_version(), 1);
    assert!(g.get_node("task:a").unwrap().is_some());
}

#[test]
fn json_sqlite_contract_parity() {
    let dir = tempdir().unwrap();
    let root = ProjectRoot::new(dir.path()).unwrap();
    std::fs::create_dir_all(dir.path().join(".dare")).unwrap();

    let mut sqlite = open_graph(
        &root,
        &GraphConfig {
            backend: GraphBackend::Sqlite,
            path: GRAPH_DB_REL.into(),
        },
    )
    .unwrap();
    let mut json = open_graph(
        &root,
        &GraphConfig {
            backend: GraphBackend::Json,
            path: GRAPH_JSON_REL.into(),
        },
    )
    .unwrap();

    let mut n = GraphNode::new("task:alpha", NodeType::Task, "Alpha");
    n.vector = Some(vec![1.0, 2.0, 3.0]);
    let e = GraphEdge::new(
        "depends_on:task:alpha->task:beta",
        "task:alpha",
        "task:beta",
        EdgeType::DependsOn,
    );
    let beta = GraphNode::new("task:beta", NodeType::Task, "Beta");

    for g in [&mut sqlite, &mut json] {
        g.add_node(n.clone()).unwrap();
        g.add_node(beta.clone()).unwrap();
        g.add_edge(e.clone()).unwrap();
    }

    let s_doc = sqlite.export_document().unwrap();
    let j_doc = json.export_document().unwrap();
    assert_eq!(s_doc.nodes.len(), j_doc.nodes.len());
    assert_eq!(s_doc.edges.len(), j_doc.edges.len());
    assert_eq!(
        s_doc.nodes.iter().map(|n| &n.id).collect::<Vec<_>>(),
        j_doc.nodes.iter().map(|n| &n.id).collect::<Vec<_>>()
    );
    assert_eq!(
        sqlite.load_vectors().unwrap()[0].v,
        json.load_vectors().unwrap()[0].v
    );

    let edges_s = sqlite.get_edges("task:alpha", EdgeDirection::Out).unwrap();
    let edges_j = json.get_edges("task:alpha", EdgeDirection::Out).unwrap();
    assert_eq!(edges_s.len(), 1);
    assert_eq!(edges_j[0].id, edges_s[0].id);

    sqlite.delete_node("task:alpha").unwrap();
    json.delete_node("task:alpha").unwrap();
    assert!(sqlite.get_node("task:alpha").unwrap().is_none());
    assert!(json.get_node("task:alpha").unwrap().is_none());
    assert!(sqlite
        .get_edges("task:beta", EdgeDirection::Both)
        .unwrap()
        .is_empty());
}

#[test]
fn path_jail_rejects_escape() {
    let dir = tempdir().unwrap();
    let root = ProjectRoot::new(dir.path()).unwrap();
    assert!(SafeRelativePath::new("../outside.db").is_err());
    let _ = root;
}

#[test]
fn vector_bytes_stable() {
    let bytes = serialize_f32_le(&[1.0, 2.0]);
    let back = deserialize_f32_le(&bytes).unwrap();
    assert_eq!(back, vec![1.0, 2.0]);
}
