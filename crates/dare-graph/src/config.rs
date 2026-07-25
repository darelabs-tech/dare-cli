//! Graph backend config from `dare-graph.yml`.

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::Deserialize;

use crate::knowledge_graph::KnowledgeGraph;
use crate::storage::{JsonGraph, SqliteGraph};
#[cfg(feature = "neo4j")]
use crate::neo4j::Neo4jGraph;

pub const GRAPH_DB_REL: &str = ".dare/graph.db";
pub const GRAPH_JSON_REL: &str = ".dare/graph.json";
pub const GRAPH_YML_REL: &str = "dare-graph.yml";

/// Error when Neo4j is selected without the Cargo feature.
pub const MSG_NEO4J_FEATURE_REQUIRED: &str = "neo4j backend requires the neo4j feature";

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

/// Neo4j HTTP connection settings (password redacted in Debug).
#[derive(Clone, PartialEq, Eq)]
pub struct Neo4jConnectConfig {
    pub url: String,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl std::fmt::Debug for Neo4jConnectConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Neo4jConnectConfig")
            .field("url", &self.url)
            .field("user", &self.user)
            .field("password", &"<redacted>")
            .field("database", &self.database)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphConfig {
    pub backend: GraphBackend,
    /// Relative path under project root (sqlite/json). Unused for Neo4j.
    pub path: String,
    /// Present when `backend == Neo4j` (feature `neo4j` required to open).
    pub neo4j: Option<Neo4jConnectConfig>,
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
    #[allow(dead_code)] // read when feature = "neo4j"
    neo4j: Option<Neo4jYml>,
}

#[derive(Debug, Deserialize)]
struct BackendBlock {
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[allow(dead_code)] // fields read when feature = "neo4j"
struct Neo4jYml {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    password: Option<String>,
    #[serde(default)]
    database: Option<String>,
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
            neo4j: None,
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

    if backend == GraphBackend::Neo4j {
        #[cfg(not(feature = "neo4j"))]
        {
            return Err(CoreError::invalid_input(MSG_NEO4J_FEATURE_REQUIRED));
        }
        #[cfg(feature = "neo4j")]
        {
            let neo4j = resolve_neo4j_connect(parsed.neo4j.as_ref())?;
            return Ok(GraphConfig {
                backend: GraphBackend::Neo4j,
                path: String::new(),
                neo4j: Some(neo4j),
            });
        }
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
        GraphBackend::Neo4j => String::new(),
    };
    Ok(GraphConfig {
        backend,
        path,
        neo4j: None,
    })
}

#[cfg(feature = "neo4j")]
fn resolve_neo4j_connect(yml: Option<&Neo4jYml>) -> CoreResult<Neo4jConnectConfig> {
    use crate::neo4j::{validate_neo4j_url, NEO4J_DEFAULT_DB};

    let y = yml.cloned().unwrap_or_default();
    let url = std::env::var("NEO4J_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| y.url.filter(|s| !s.trim().is_empty()))
        .ok_or_else(|| CoreError::invalid_input("neo4j url is required (yaml or NEO4J_URL)"))?;
    validate_neo4j_url(&url)?;

    let user = std::env::var("NEO4J_USER")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| y.user.filter(|s| !s.is_empty()))
        .unwrap_or_else(|| "neo4j".to_string());

    let password = std::env::var("NEO4J_PASSWORD")
        .ok()
        .or(y.password)
        .unwrap_or_default();

    let database = std::env::var("NEO4J_DATABASE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| y.database.filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| NEO4J_DEFAULT_DB.to_string());

    Ok(Neo4jConnectConfig {
        url,
        user,
        password,
        database,
    })
}

/// Open the configured backend.
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
        GraphBackend::Neo4j => {
            #[cfg(not(feature = "neo4j"))]
            {
                let _ = root;
                Err(CoreError::invalid_input(MSG_NEO4J_FEATURE_REQUIRED))
            }
            #[cfg(feature = "neo4j")]
            {
                let neo = config.neo4j.as_ref().ok_or_else(|| {
                    CoreError::invalid_input("neo4j config missing (url/database/credentials)")
                })?;
                Ok(GraphHandle::Neo4j(Neo4jGraph::connect(neo)?))
            }
        }
    }
}

pub enum GraphHandle {
    Sqlite(SqliteGraph),
    Json(JsonGraph),
    #[cfg(feature = "neo4j")]
    Neo4j(Neo4jGraph),
}

impl std::fmt::Debug for GraphHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite(_) => f.write_str("GraphHandle::Sqlite(..)"),
            Self::Json(_) => f.write_str("GraphHandle::Json(..)"),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => f.debug_tuple("GraphHandle::Neo4j").field(g).finish(),
        }
    }
}

impl KnowledgeGraph for GraphHandle {
    fn schema_version(&self) -> u32 {
        match self {
            Self::Sqlite(g) => g.schema_version(),
            Self::Json(g) => g.schema_version(),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.schema_version(),
        }
    }

    fn migrate(&mut self) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.migrate(),
            Self::Json(g) => g.migrate(),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.migrate(),
        }
    }

    fn add_node(&mut self, node: crate::types::GraphNode) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.add_node(node),
            Self::Json(g) => g.add_node(node),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.add_node(node),
        }
    }

    fn get_node(&self, id: &str) -> CoreResult<Option<crate::types::GraphNode>> {
        match self {
            Self::Sqlite(g) => g.get_node(id),
            Self::Json(g) => g.get_node(id),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.get_node(id),
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
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.query_nodes(ty, limit),
        }
    }

    fn delete_node(&mut self, id: &str) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.delete_node(id),
            Self::Json(g) => g.delete_node(id),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.delete_node(id),
        }
    }

    fn add_edge(&mut self, edge: crate::types::GraphEdge) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.add_edge(edge),
            Self::Json(g) => g.add_edge(edge),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.add_edge(edge),
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
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.get_edges(node_id, direction),
        }
    }

    fn load_vectors(&self) -> CoreResult<Vec<crate::types::VectorRow>> {
        match self {
            Self::Sqlite(g) => g.load_vectors(),
            Self::Json(g) => g.load_vectors(),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.load_vectors(),
        }
    }

    fn get_statistics(&self) -> CoreResult<crate::types::GraphStatistics> {
        match self {
            Self::Sqlite(g) => g.get_statistics(),
            Self::Json(g) => g.get_statistics(),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.get_statistics(),
        }
    }

    fn export_document(&self) -> CoreResult<crate::types::GraphStoreDocument> {
        match self {
            Self::Sqlite(g) => g.export_document(),
            Self::Json(g) => g.export_document(),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.export_document(),
        }
    }

    fn import_document(&mut self, doc: &crate::types::GraphStoreDocument) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.import_document(doc),
            Self::Json(g) => g.import_document(doc),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.import_document(doc),
        }
    }

    fn flush(&mut self) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.flush(),
            Self::Json(g) => g.flush(),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.flush(),
        }
    }

    fn close(self) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.close(),
            Self::Json(g) => g.close(),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(g) => g.close(),
        }
    }
}

impl GraphHandle {
    /// Best-effort FTS5 rebuild after ingest (SQLite only; JSON/Neo4j no-op).
    pub fn try_rebuild_fts5(&mut self) -> CoreResult<()> {
        match self {
            Self::Sqlite(g) => g.try_rebuild_fts5(),
            Self::Json(_) => Ok(()),
            #[cfg(feature = "neo4j")]
            Self::Neo4j(_) => Ok(()),
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
        assert!(cfg.neo4j.is_none());
    }

    #[test]
    fn neo4j_rejected_without_feature() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        std::fs::write(
            dir.path().join("dare-graph.yml"),
            "backend: neo4j\nneo4j:\n  url: http://localhost:7474\n",
        )
        .unwrap();
        #[cfg(not(feature = "neo4j"))]
        {
            let err = load_graph_config(&root, None).unwrap_err();
            assert!(
                err.to_string().contains(MSG_NEO4J_FEATURE_REQUIRED),
                "err={err}"
            );
            let err = open_graph(
                &root,
                &GraphConfig {
                    backend: GraphBackend::Neo4j,
                    path: String::new(),
                    neo4j: None,
                },
            )
            .unwrap_err();
            assert!(err.to_string().contains(MSG_NEO4J_FEATURE_REQUIRED));
        }
        #[cfg(feature = "neo4j")]
        {
            let cfg = load_graph_config(&root, None).unwrap();
            assert_eq!(cfg.backend, GraphBackend::Neo4j);
            assert!(cfg.neo4j.is_some());
            assert_eq!(
                cfg.neo4j.as_ref().unwrap().url,
                "http://localhost:7474"
            );
        }
    }

    #[test]
    fn neo4j_connect_config_redacts_password() {
        let cfg = Neo4jConnectConfig {
            url: "http://localhost:7474".into(),
            user: "neo4j".into(),
            password: "hunter2".into(),
            database: "neo4j".into(),
        };
        let s = format!("{cfg:?}");
        assert!(!s.contains("hunter2"));
        assert!(s.contains("<redacted>"));
    }
}
