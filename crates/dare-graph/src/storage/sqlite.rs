//! SQLite backend (`.dare/graph.db`) via rusqlite bundled.

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{Map, Value};

use crate::knowledge_graph::{EdgeDirection, KnowledgeGraph};
use crate::migrations::{
    detect_sqlite_schema_version, ensure_baseline_schema, migrate_sqlite, CURRENT_SCHEMA_VERSION,
};
use crate::types::{
    empty_edges_by_type, empty_nodes_by_type, GraphEdge, GraphNode, GraphStatistics,
    GraphStoreDocument, NodeType, VectorRow,
};
use crate::vector::{deserialize_f32_le, serialize_f32_le};

pub struct SqliteGraph {
    conn: Connection,
    version: u32,
}

impl SqliteGraph {
    /// Open or create a SQLite graph under the project jail.
    ///
    /// Does **not** run migrations (ADR-006). New empty files get baseline DDL so the
    /// store is usable; version table is only written by [`KnowledgeGraph::migrate`].
    pub fn open(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<Self> {
        let abs = root.resolve(rel)?;
        if let Some(parent) = abs.as_path().parent() {
            std::fs::create_dir_all(parent.as_std_path())
                .map_err(|e| CoreError::io(e.to_string()))?;
        }
        let path = abs.as_path().as_std_path();
        let is_new = !path.exists();
        let conn = Connection::open(path).map_err(|e| CoreError::io(e.to_string()))?;
        if is_new {
            ensure_baseline_schema(&conn)?;
        }
        let version = detect_sqlite_schema_version(&conn)?;
        Ok(Self { conn, version })
    }

    /// Open an existing absolute path (tests / fixtures). Still no silent migrate.
    pub fn open_path(path: &std::path::Path) -> CoreResult<Self> {
        let conn = Connection::open(path).map_err(|e| CoreError::io(e.to_string()))?;
        let version = detect_sqlite_schema_version(&conn)?;
        Ok(Self { conn, version })
    }

    fn require_usable(&self) -> CoreResult<()> {
        if self.version == 0 {
            return Err(CoreError::config(
                "graph schema version 0 requires explicit migrate() before use",
            ));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn row_to_node(
        id: String,
        ty: String,
        label: String,
        description: Option<String>,
        vector: Option<Vec<u8>>,
        metadata: String,
        created_at: Option<String>,
        updated_at: Option<String>,
    ) -> CoreResult<GraphNode> {
        let meta: Map<String, Value> = serde_json::from_str(&metadata).unwrap_or_default();
        let vector = vector.as_deref().and_then(deserialize_f32_le);
        Ok(GraphNode {
            id,
            node_type: ty,
            label,
            description,
            vector,
            metadata: meta,
            created_at,
            updated_at,
        })
    }

    fn row_to_edge(
        id: String,
        source_id: String,
        target_id: String,
        ty: String,
        weight: f64,
        metadata: String,
    ) -> CoreResult<GraphEdge> {
        let meta: Map<String, Value> = serde_json::from_str(&metadata).unwrap_or_default();
        Ok(GraphEdge {
            id,
            source_id,
            target_id,
            edge_type: ty,
            weight,
            metadata: meta,
        })
    }
}

impl SqliteGraph {
    /// Best-effort FTS5 index rebuild (keyword acceleration; LIKE remains SoT).
    pub fn try_rebuild_fts5(&mut self) -> CoreResult<()> {
        crate::ingest::rebuild_fts5(&self.conn)
    }
}

impl KnowledgeGraph for SqliteGraph {
    fn schema_version(&self) -> u32 {
        self.version
    }

    fn migrate(&mut self) -> CoreResult<()> {
        self.version = migrate_sqlite(&self.conn)?;
        Ok(())
    }

    fn add_node(&mut self, node: GraphNode) -> CoreResult<()> {
        self.require_usable()?;
        let meta = serde_json::to_string(&node.metadata)
            .map_err(|e| CoreError::invalid_input(e.to_string()))?;
        let desc = node.description.clone().unwrap_or_default();
        let existing: Option<String> = self
            .conn
            .query_row("SELECT id FROM nodes WHERE id = ?1", [&node.id], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|e| CoreError::io(e.to_string()))?;

        if existing.is_some() {
            if let Some(ref v) = node.vector {
                let blob = serialize_f32_le(v);
                self.conn
                    .execute(
                        "UPDATE nodes SET label = ?1, description = ?2, vector = ?3, metadata = ?4, updated_at = datetime('now') WHERE id = ?5",
                        params![node.label, desc, blob, meta, node.id],
                    )
                    .map_err(|e| CoreError::io(e.to_string()))?;
            } else {
                self.conn
                    .execute(
                        "UPDATE nodes SET label = ?1, description = ?2, metadata = ?3, updated_at = datetime('now') WHERE id = ?4",
                        params![node.label, desc, meta, node.id],
                    )
                    .map_err(|e| CoreError::io(e.to_string()))?;
            }
        } else {
            let blob = node.vector.as_ref().map(|v| serialize_f32_le(v));
            self.conn
                .execute(
                    "INSERT INTO nodes (id, type, label, description, vector, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![node.id, node.node_type, node.label, desc, blob, meta],
                )
                .map_err(|e| CoreError::io(e.to_string()))?;
        }
        Ok(())
    }

    fn get_node(&self, id: &str) -> CoreResult<Option<GraphNode>> {
        self.require_usable()?;
        let row = self
            .conn
            .query_row(
                "SELECT id, type, label, description, vector, metadata, created_at, updated_at FROM nodes WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<Vec<u8>>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                    ))
                },
            )
            .optional()
            .map_err(|e| CoreError::io(e.to_string()))?;
        match row {
            None => Ok(None),
            Some((id, ty, label, description, vector, metadata, created_at, updated_at)) => {
                Ok(Some(Self::row_to_node(
                    id,
                    ty,
                    label,
                    description,
                    vector,
                    metadata,
                    created_at,
                    updated_at,
                )?))
            }
        }
    }

    fn query_nodes(
        &self,
        ty: Option<NodeType>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<GraphNode>> {
        self.require_usable()?;
        let lim = limit.unwrap_or(usize::MAX);
        let mut out = Vec::new();
        if let Some(t) = ty {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT id, type, label, description, vector, metadata, created_at, updated_at FROM nodes WHERE type = ?1 ORDER BY id ASC",
                )
                .map_err(|e| CoreError::io(e.to_string()))?;
            let rows = stmt
                .query_map([t.as_str()], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<Vec<u8>>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                    ))
                })
                .map_err(|e| CoreError::io(e.to_string()))?;
            for r in rows {
                let (id, ty, label, description, vector, metadata, created_at, updated_at) =
                    r.map_err(|e| CoreError::io(e.to_string()))?;
                out.push(Self::row_to_node(
                    id,
                    ty,
                    label,
                    description,
                    vector,
                    metadata,
                    created_at,
                    updated_at,
                )?);
                if out.len() >= lim {
                    break;
                }
            }
        } else {
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT id, type, label, description, vector, metadata, created_at, updated_at FROM nodes ORDER BY id ASC",
                )
                .map_err(|e| CoreError::io(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<Vec<u8>>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                    ))
                })
                .map_err(|e| CoreError::io(e.to_string()))?;
            for r in rows {
                let (id, ty, label, description, vector, metadata, created_at, updated_at) =
                    r.map_err(|e| CoreError::io(e.to_string()))?;
                out.push(Self::row_to_node(
                    id,
                    ty,
                    label,
                    description,
                    vector,
                    metadata,
                    created_at,
                    updated_at,
                )?);
                if out.len() >= lim {
                    break;
                }
            }
        }
        Ok(out)
    }

    fn delete_node(&mut self, id: &str) -> CoreResult<()> {
        self.require_usable()?;
        self.conn
            .execute(
                "DELETE FROM edges WHERE source_id = ?1 OR target_id = ?1",
                [id],
            )
            .map_err(|e| CoreError::io(e.to_string()))?;
        self.conn
            .execute("DELETE FROM nodes WHERE id = ?1", [id])
            .map_err(|e| CoreError::io(e.to_string()))?;
        Ok(())
    }

    fn add_edge(&mut self, edge: GraphEdge) -> CoreResult<()> {
        self.require_usable()?;
        let meta = serde_json::to_string(&edge.metadata)
            .map_err(|e| CoreError::invalid_input(e.to_string()))?;
        self.conn
            .execute(
                "INSERT OR REPLACE INTO edges (id, source_id, target_id, type, weight, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    edge.id,
                    edge.source_id,
                    edge.target_id,
                    edge.edge_type,
                    edge.weight,
                    meta
                ],
            )
            .map_err(|e| CoreError::io(e.to_string()))?;
        Ok(())
    }

    fn get_edges(&self, node_id: &str, direction: EdgeDirection) -> CoreResult<Vec<GraphEdge>> {
        self.require_usable()?;
        let sql = match direction {
            EdgeDirection::Out => {
                "SELECT id, source_id, target_id, type, weight, metadata FROM edges WHERE source_id = ?1 ORDER BY id ASC"
            }
            EdgeDirection::In => {
                "SELECT id, source_id, target_id, type, weight, metadata FROM edges WHERE target_id = ?1 ORDER BY id ASC"
            }
            EdgeDirection::Both => {
                "SELECT id, source_id, target_id, type, weight, metadata FROM edges WHERE source_id = ?1 OR target_id = ?1 ORDER BY id ASC"
            }
        };
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| CoreError::io(e.to_string()))?;
        let rows = stmt
            .query_map([node_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(|e| CoreError::io(e.to_string()))?;
        let mut out = Vec::new();
        for r in rows {
            let (id, source_id, target_id, ty, weight, metadata) =
                r.map_err(|e| CoreError::io(e.to_string()))?;
            out.push(Self::row_to_edge(
                id, source_id, target_id, ty, weight, metadata,
            )?);
        }
        Ok(out)
    }

    fn load_vectors(&self) -> CoreResult<Vec<VectorRow>> {
        self.require_usable()?;
        let mut stmt = self
            .conn
            .prepare("SELECT id, vector FROM nodes WHERE vector IS NOT NULL ORDER BY id ASC")
            .map_err(|e| CoreError::io(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
            })
            .map_err(|e| CoreError::io(e.to_string()))?;
        let mut out = Vec::new();
        for r in rows {
            let (id, blob) = r.map_err(|e| CoreError::io(e.to_string()))?;
            if let Some(v) = deserialize_f32_le(&blob) {
                if !v.is_empty() {
                    out.push(VectorRow { id, v });
                }
            }
        }
        Ok(out)
    }

    fn get_statistics(&self) -> CoreResult<GraphStatistics> {
        self.require_usable()?;
        let total_nodes: u64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))
            .map_err(|e| CoreError::io(e.to_string()))?;
        let total_edges: u64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM edges", [], |r| r.get(0))
            .map_err(|e| CoreError::io(e.to_string()))?;
        let mut nodes_by_type = empty_nodes_by_type();
        let mut edges_by_type = empty_edges_by_type();
        {
            let mut stmt = self
                .conn
                .prepare("SELECT type, COUNT(*) FROM nodes GROUP BY type")
                .map_err(|e| CoreError::io(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
                })
                .map_err(|e| CoreError::io(e.to_string()))?;
            for r in rows {
                let (ty, n) = r.map_err(|e| CoreError::io(e.to_string()))?;
                *nodes_by_type.entry(ty).or_insert(0) = n;
            }
        }
        {
            let mut stmt = self
                .conn
                .prepare("SELECT type, COUNT(*) FROM edges GROUP BY type")
                .map_err(|e| CoreError::io(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
                })
                .map_err(|e| CoreError::io(e.to_string()))?;
            for r in rows {
                let (ty, n) = r.map_err(|e| CoreError::io(e.to_string()))?;
                *edges_by_type.entry(ty).or_insert(0) = n;
            }
        }
        Ok(GraphStatistics {
            total_nodes,
            total_edges,
            nodes_by_type,
            edges_by_type,
        })
    }

    fn export_document(&self) -> CoreResult<GraphStoreDocument> {
        Ok(GraphStoreDocument {
            nodes: self.query_nodes(None, None)?,
            edges: {
                self.require_usable()?;
                let mut stmt = self
                    .conn
                    .prepare(
                        "SELECT id, source_id, target_id, type, weight, metadata FROM edges ORDER BY id ASC",
                    )
                    .map_err(|e| CoreError::io(e.to_string()))?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, String>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, f64>(4)?,
                            row.get::<_, String>(5)?,
                        ))
                    })
                    .map_err(|e| CoreError::io(e.to_string()))?;
                let mut edges = Vec::new();
                for r in rows {
                    let (id, source_id, target_id, ty, weight, metadata) =
                        r.map_err(|e| CoreError::io(e.to_string()))?;
                    edges.push(Self::row_to_edge(
                        id, source_id, target_id, ty, weight, metadata,
                    )?);
                }
                edges
            },
        })
    }

    fn import_document(&mut self, doc: &GraphStoreDocument) -> CoreResult<()> {
        for n in &doc.nodes {
            self.add_node(n.clone())?;
        }
        for e in &doc.edges {
            self.add_edge(e.clone())?;
        }
        Ok(())
    }

    fn flush(&mut self) -> CoreResult<()> {
        // Native SQLite persistence — no sql.js full-file export.
        Ok(())
    }

    fn close(self) -> CoreResult<()> {
        self.conn
            .close()
            .map_err(|(_, e)| CoreError::io(e.to_string()))?;
        Ok(())
    }
}

impl SqliteGraph {
    pub fn current_schema_version_const() -> u32 {
        CURRENT_SCHEMA_VERSION
    }
}
