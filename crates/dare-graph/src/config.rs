//! Graph backend config from `dare-graph.yml` (factory, no Neo4j in 040).

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::Deserialize;

use crate::knowledge_graph::KnowledgeGraph;
use crate::storage::{JsonGraph, SqliteGraph};

pub const GRAPH_DB_REL: &str = ".dare/graph.db";
pub const GRAPH_JSON_REL: &str = ".dare/graph.json";
pub const GRAPH_YML_REL: &str = "dare-graph.yml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GraphBackend {
    Sqlite,
    Json,
    Neo4j,
}

impl Default for GraphBackend {
    fn default() -> Self {
        Self::Sqlite
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphConfig {
    pub backend: GraphBackend,
    /// Relative path under project root.
    pub path: String,
}

#[derive(Debug, Deserialize)]
struct GraphYml {
    #[serde(default)]
    backend: Option<String>,
    #[serde(default)]
    sqlite: Option<BackendBlock>,
    #[serde(default)]
    json: Option<BackendBlock>,
    #[serde(default)]
    neo4j: Option<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
struct BackendBlock {
    #[serde(default)]
    path: Option<String>,
}

/// Load graph config: explicit override, else `dare-graph.yml`, else sqlite default.
pub fn load_graph_config(
    root: &ProjectRoot,
    explicit: Option<GraphConfig>,
) -> CoreResult<GraphConfig> {
    if let Some(c) = explicit {
        return Ok(c);
    }
    let rel = SafeRelativePath::new(GRAPH_YML_REL)?;
    let abs = root.resolve(&rel)?;
    if !abs.as_path().as_std_path().exists() {
        return Ok(GraphConfig {
            backend: GraphBackend::Sqlite,
            path: GRAPH_DB_REL.to_string(),
        });
    }
    let text = std::fs::read_to_string(abs.as_path().as_std_path())
        .map_err(|e| CoreError::io(e.to_string()))?;
    let parsed: GraphYml = serde_yaml::from_str(&text)
        .map_err(|e| CoreError::config(format!("invalid dare-graph.yml: {e}")))?;
    let backend_str = parsed
        .backend
        .as_deref()
        .unwrap_or("sqlite")
        .to_ascii_lowercase();
    let backend = match backend_str.as_str() {
        "sqlite" => GraphBackend::Sqlite,
        "json" => GraphBackend::Json,
        "neo4j" => GraphBackend::Neo4j,
        other => {
            return Err(CoreError::invalid_input(format!(
                "unknown graph backend: {other}"
            )));
        }
    };
    if backend == GraphBackend::Neo4j || parsed.neo4j.is_some() && backend_str == "neo4j" {
        return Err(CoreError::invalid_input(
            "neo4j backend not implemented (microplano 043)",
        ));
    }
    let path = match backend {
        GraphBackend::Sqlite => parsed
            .sqlite
            .and_then(|b| b.path)
            .unwrap_or_else(|| GRAPH_DB_REL.to_string()),
        GraphBackend::Json => parsed
            .json
            .and_then(|b| b.path)
            .unwrap_or_else(|| GRAPH_JSON_REL.to_string()),
        GraphBackend::Neo4j => unreachable!(),
    };
    Ok(GraphConfig { backend, path })
}

/// Open the configured backend. Neo4j rejected.
pub fn open_graph(root: &ProjectRoot, config: &GraphConfig) -> CoreResult<GraphHandle> {
    match config.backend {
        GraphBackend::Sqlite => {
            let rel = SafeRelativePath::new(&config.path)?;
            Ok(GraphHandle::Sqlite(SqliteGraph::open(root, &rel)?))
        }
        GraphBackend::Json => {
            let rel = SafeRelativePath::new(&config.path)?;
            Ok(GraphHandle::Json(JsonGraph::open(root, &rel)?))
        }
        GraphBackend::Neo4j => Err(CoreError::invalid_input(
            "neo4j backend not implemented (microplano 043)",
        )),
    }
}

pub enum GraphHandle {
    Sqlite(SqliteGraph),
    Json(JsonGraph),
}

impl KnowledgeGraph for GraphHandle {
    fn schema_version(&self) -> u32 {
        match self {
            Self::Sqlite(g) => g.schema_version(),
            Self::Json(g) => g.schema_version(),
        }
    }

    fn migrate(&mut self) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.migrate(),
            Self::Json(g) => g.migrate(),
        }
    }

    fn add_node(&mut self, node: crate::types::GraphNode) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.add_node(node),
            Self::Json(g) => g.add_node(node),
        }
    }

    fn get_node(&self, id: &str) -> CoreResult<Option<crate::types::GraphNode>> {
        match self {
            Self::Sqlite(g) => g.get_node(id),
            Self::Json(g) => g.get_node(id),
        }
    }

    fn query_nodes(
        &self,
        ty: Option<crate::types::NodeType>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<crate::types::GraphNode>> {
        match self {
            Self::Sqlite(g) => g.query_nodes(ty, limit),
            Self::Json(g) => g.query_nodes(ty, limit),
        }
    }

    fn delete_node(&mut self, id: &str) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.delete_node(id),
            Self::Json(g) => g.delete_node(id),
        }
    }

    fn add_edge(&mut self, edge: crate::types::GraphEdge) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.add_edge(edge),
            Self::Json(g) => g.add_edge(edge),
        }
    }

    fn get_edges(
        &self,
        node_id: &str,
        direction: crate::knowledge_graph::EdgeDirection,
    ) -> CoreResult<Vec<crate::types::GraphEdge>> {
        match self {
            Self::Sqlite(g) => g.get_edges(node_id, direction),
            Self::Json(g) => g.get_edges(node_id, direction),
        }
    }

    fn load_vectors(&self) -> CoreResult<Vec<crate::types::VectorRow>> {
        match self {
            Self::Sqlite(g) => g.load_vectors(),
            Self::Json(g) => g.load_vectors(),
        }
    }

    fn get_statistics(&self) -> CoreResult<crate::types::GraphStatistics> {
        match self {
            Self::Sqlite(g) => g.get_statistics(),
            Self::Json(g) => g.get_statistics(),
        }
    }

    fn export_document(&self) -> CoreResult<crate::types::GraphStoreDocument> {
        match self {
            Self::Sqlite(g) => g.export_document(),
            Self::Json(g) => g.export_document(),
        }
    }

    fn import_document(&mut self, doc: &crate::types::GraphStoreDocument) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.import_document(doc),
            Self::Json(g) => g.import_document(doc),
        }
    }

    fn flush(&mut self) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.flush(),
            Self::Json(g) => g.flush(),
        }
    }

    fn close(self) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.close(),
            Self::Json(g) => g.close(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn default_sqlite_when_yml_missing() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let cfg = load_graph_config(&root, None).unwrap();
        assert_eq!(cfg.backend, GraphBackend::Sqlite);
        assert_eq!(cfg.path, GRAPH_DB_REL);
    }

    #[test]
    fn neo4j_rejected() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        std::fs::write(
            dir.path().join("dare-graph.yml"),
            "backend: neo4j\nneo4j:\n  url: http://localhost:7474\n",
        )
        .unwrap();
        let err = load_graph_config(&root, None).unwrap_err();
        assert!(err.to_string().contains("not implemented"));
    }
}
