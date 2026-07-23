//! JSON backend (`.dare/graph.json`).

use std::collections::BTreeMap;

use dare_core::fs::atomic_write;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};

use crate::knowledge_graph::{EdgeDirection, KnowledgeGraph};
use crate::migrations::CURRENT_SCHEMA_VERSION;
use crate::types::{
    empty_edges_by_type, empty_nodes_by_type, GraphEdge, GraphNode, GraphStatistics,
    GraphStoreDocument, NodeType, VectorRow,
};

pub struct JsonGraph {
    root: ProjectRoot,
    rel: SafeRelativePath,
    nodes: BTreeMap<String, GraphNode>,
    edges: BTreeMap<String, GraphEdge>,
    dirty: bool,
}

impl JsonGraph {
    pub fn open(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<Self> {
        let abs = root.resolve(rel)?;
        let path = abs.as_path().as_std_path();
        let mut nodes = BTreeMap::new();
        let mut edges = BTreeMap::new();
        if path.exists() {
            let text = std::fs::read_to_string(path).map_err(|e| CoreError::io(e.to_string()))?;
            let doc: GraphStoreDocument = serde_json::from_str(&text)
                .map_err(|e| CoreError::config(format!("invalid graph.json: {e}")))?;
            for n in doc.nodes {
                nodes.insert(n.id.clone(), n);
            }
            for e in doc.edges {
                edges.insert(e.id.clone(), e);
            }
        }
        Ok(Self {
            root: root.clone(),
            rel: rel.clone(),
            nodes,
            edges,
            dirty: false,
        })
    }

    fn persist(&mut self) -> CoreResult<()> {
        let doc = GraphStoreDocument {
            nodes: self.nodes.values().cloned().collect(),
            edges: self.edges.values().cloned().collect(),
        };
        let bytes = serde_json::to_vec_pretty(&doc).map_err(|e| CoreError::io(e.to_string()))?;
        atomic_write(&self.root, &self.rel, &bytes)?;
        self.dirty = false;
        Ok(())
    }
}

impl KnowledgeGraph for JsonGraph {
    fn schema_version(&self) -> u32 {
        CURRENT_SCHEMA_VERSION
    }

    fn migrate(&mut self) -> CoreResult<()> {
        // JSON store has no DDL; version is always current.
        Ok(())
    }

    fn add_node(&mut self, node: GraphNode) -> CoreResult<()> {
        if let Some(existing) = self.nodes.get_mut(&node.id) {
            existing.label = node.label;
            existing.description = node.description;
            existing.metadata = node.metadata;
            existing.node_type = node.node_type;
            if node.vector.is_some() {
                existing.vector = node.vector;
            }
            existing.updated_at = node.updated_at.or_else(|| existing.updated_at.clone());
        } else {
            self.nodes.insert(node.id.clone(), node);
        }
        self.dirty = true;
        self.persist()
    }

    fn get_node(&self, id: &str) -> CoreResult<Option<GraphNode>> {
        Ok(self.nodes.get(id).cloned())
    }

    fn query_nodes(
        &self,
        ty: Option<NodeType>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<GraphNode>> {
        let lim = limit.unwrap_or(usize::MAX);
        let mut out: Vec<_> = self
            .nodes
            .values()
            .filter(|n| ty.map(|t| n.node_type == t.as_str()).unwrap_or(true))
            .cloned()
            .collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out.truncate(lim);
        Ok(out)
    }

    fn delete_node(&mut self, id: &str) -> CoreResult<()> {
        self.nodes.remove(id);
        self.edges
            .retain(|_, e| e.source_id != id && e.target_id != id);
        self.dirty = true;
        self.persist()
    }

    fn add_edge(&mut self, edge: GraphEdge) -> CoreResult<()> {
        self.edges.insert(edge.id.clone(), edge);
        self.dirty = true;
        self.persist()
    }

    fn get_edges(&self, node_id: &str, direction: EdgeDirection) -> CoreResult<Vec<GraphEdge>> {
        let mut out: Vec<_> = self
            .edges
            .values()
            .filter(|e| match direction {
                EdgeDirection::Out => e.source_id == node_id,
                EdgeDirection::In => e.target_id == node_id,
                EdgeDirection::Both => e.source_id == node_id || e.target_id == node_id,
            })
            .cloned()
            .collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }

    fn load_vectors(&self) -> CoreResult<Vec<VectorRow>> {
        let mut out: Vec<_> = self
            .nodes
            .values()
            .filter_map(|n| {
                n.vector.as_ref().and_then(|v| {
                    if v.is_empty() {
                        None
                    } else {
                        Some(VectorRow {
                            id: n.id.clone(),
                            v: v.clone(),
                        })
                    }
                })
            })
            .collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }

    fn get_statistics(&self) -> CoreResult<GraphStatistics> {
        let mut nodes_by_type = empty_nodes_by_type();
        let mut edges_by_type = empty_edges_by_type();
        for n in self.nodes.values() {
            *nodes_by_type.entry(n.node_type.clone()).or_insert(0) += 1;
        }
        for e in self.edges.values() {
            *edges_by_type.entry(e.edge_type.clone()).or_insert(0) += 1;
        }
        Ok(GraphStatistics {
            total_nodes: self.nodes.len() as u64,
            total_edges: self.edges.len() as u64,
            nodes_by_type,
            edges_by_type,
        })
    }

    fn export_document(&self) -> CoreResult<GraphStoreDocument> {
        let mut nodes: Vec<_> = self.nodes.values().cloned().collect();
        let mut edges: Vec<_> = self.edges.values().cloned().collect();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        edges.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(GraphStoreDocument { nodes, edges })
    }

    fn import_document(&mut self, doc: &GraphStoreDocument) -> CoreResult<()> {
        for n in &doc.nodes {
            self.nodes.insert(n.id.clone(), n.clone());
        }
        for e in &doc.edges {
            self.edges.insert(e.id.clone(), e.clone());
        }
        self.dirty = true;
        self.persist()
    }

    fn flush(&mut self) -> CoreResult<()> {
        if self.dirty {
            self.persist()?;
        }
        Ok(())
    }

    fn close(mut self) -> CoreResult<()> {
        self.flush()
    }
}
