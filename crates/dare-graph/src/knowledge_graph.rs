//! `KnowledgeGraph` storage trait (search/traverse deferred to 041+).

use dare_core::CoreResult;

use crate::types::{
    GraphEdge, GraphNode, GraphStatistics, GraphStoreDocument, NodeType, VectorRow,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeDirection {
    Out,
    In,
    Both,
}

impl EdgeDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Out => "out",
            Self::In => "in",
            Self::Both => "both",
        }
    }
}

/// Storage interface shared by SQLite and JSON backends.
pub trait KnowledgeGraph {
    fn schema_version(&self) -> u32;

    /// Apply pending schema migrations. Must not run implicitly on open (ADR-006).
    fn migrate(&mut self) -> CoreResult<()>;

    fn add_node(&mut self, node: GraphNode) -> CoreResult<()>;
    fn get_node(&self, id: &str) -> CoreResult<Option<GraphNode>>;
    fn query_nodes(&self, ty: Option<NodeType>, limit: Option<usize>)
        -> CoreResult<Vec<GraphNode>>;
    fn delete_node(&mut self, id: &str) -> CoreResult<()>;

    fn add_edge(&mut self, edge: GraphEdge) -> CoreResult<()>;
    fn get_edges(&self, node_id: &str, direction: EdgeDirection) -> CoreResult<Vec<GraphEdge>>;

    fn load_vectors(&self) -> CoreResult<Vec<VectorRow>>;
    fn get_statistics(&self) -> CoreResult<GraphStatistics>;

    fn export_document(&self) -> CoreResult<GraphStoreDocument>;
    fn import_document(&mut self, doc: &GraphStoreDocument) -> CoreResult<()>;

    fn flush(&mut self) -> CoreResult<()>;
    fn close(self) -> CoreResult<()>
    where
        Self: Sized;
}
