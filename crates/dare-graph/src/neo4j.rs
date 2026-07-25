//! Experimental Neo4j HTTP backend (feature `neo4j`) — read-only subset for microplano 043.

use std::thread;
use std::time::Duration;

use dare_core::{CoreError, CoreResult};
use serde_json::{json, Map, Value};

use crate::config::Neo4jConnectConfig;
use crate::knowledge_graph::{EdgeDirection, KnowledgeGraph};
use crate::migrations::CURRENT_SCHEMA_VERSION;
use crate::types::{
    empty_edges_by_type, empty_nodes_by_type, GraphEdge, GraphNode, GraphStatistics,
    GraphStoreDocument, NodeType, VectorRow,
};

/// HTTP request timeout for Neo4j transactional endpoint.
pub const NEO4J_HTTP_TIMEOUT_MS: u64 = 5_000;
/// Extra attempts after the first failure on 5xx / transport timeout.
pub const NEO4J_HTTP_RETRIES: u32 = 2;
/// Backoff base: `NEO4J_BACKOFF_MS * attempt` between retries.
pub const NEO4J_BACKOFF_MS: u64 = 100;
/// Default Neo4j database name.
pub const NEO4J_DEFAULT_DB: &str = "neo4j";

pub const MSG_NEO4J_WRITES: &str = "neo4j writes not supported in 043";

enum HttpTransport {
    Ureq(ureq::Agent),
    #[cfg(test)]
    Scripted(std::sync::Mutex<ScriptedHttp>),
}

#[cfg(test)]
struct ScriptedHttp {
    hits: std::sync::atomic::AtomicU32,
    outcomes: std::collections::VecDeque<ScriptedOutcome>,
}

#[cfg(test)]
enum ScriptedOutcome {
    /// Simulate HTTP 5xx (retryable).
    Status(u16),
    /// Simulate transport timeout (retryable).
    Timeout,
    /// Successful JSON body.
    OkBody(String),
}

/// Read-only Neo4j graph via HTTP `POST /db/{database}/tx/commit`.
pub struct Neo4jGraph {
    base_url: String,
    user: String,
    password: String,
    database: String,
    transport: HttpTransport,
}

impl std::fmt::Debug for Neo4jGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Neo4jGraph")
            .field("base_url", &self.base_url)
            .field("user", &self.user)
            .field("password", &"<redacted>")
            .field("database", &self.database)
            .finish_non_exhaustive()
    }
}

impl Neo4jGraph {
    /// Build a client from validated connect config (scheme already checked).
    pub fn connect(cfg: &Neo4jConnectConfig) -> CoreResult<Self> {
        Self::connect_with_timeout(cfg, NEO4J_HTTP_TIMEOUT_MS)
    }

    fn connect_with_timeout(cfg: &Neo4jConnectConfig, timeout_ms: u64) -> CoreResult<Self> {
        validate_neo4j_url(&cfg.url)?;
        let base_url = cfg.url.trim().trim_end_matches('/').to_string();
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_millis(timeout_ms))
            .build();
        Ok(Self {
            base_url,
            user: cfg.user.clone(),
            password: cfg.password.clone(),
            database: if cfg.database.trim().is_empty() {
                NEO4J_DEFAULT_DB.to_string()
            } else {
                cfg.database.clone()
            },
            transport: HttpTransport::Ureq(agent),
        })
    }

    #[cfg(test)]
    fn connect_scripted(
        cfg: &Neo4jConnectConfig,
        outcomes: Vec<ScriptedOutcome>,
    ) -> CoreResult<Self> {
        validate_neo4j_url(&cfg.url)?;
        Ok(Self {
            base_url: cfg.url.trim().trim_end_matches('/').to_string(),
            user: cfg.user.clone(),
            password: cfg.password.clone(),
            database: if cfg.database.trim().is_empty() {
                NEO4J_DEFAULT_DB.to_string()
            } else {
                cfg.database.clone()
            },
            transport: HttpTransport::Scripted(std::sync::Mutex::new(ScriptedHttp {
                hits: std::sync::atomic::AtomicU32::new(0),
                outcomes: outcomes.into(),
            })),
        })
    }

    #[cfg(test)]
    fn scripted_hits(&self) -> u32 {
        match &self.transport {
            HttpTransport::Scripted(m) => m
                .lock()
                .expect("scripted lock")
                .hits
                .load(std::sync::atomic::Ordering::SeqCst),
            HttpTransport::Ureq(_) => 0,
        }
    }

    fn commit_url(&self) -> String {
        format!("{}/db/{}/tx/commit", self.base_url, self.database)
    }

    fn auth_header(&self) -> String {
        let token = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", self.user, self.password),
        );
        format!("Basic {token}")
    }

    /// Run fixed Cypher templates only (no arbitrary user Cypher).
    fn tx_commit(&self, statement: &str, parameters: Value) -> CoreResult<Value> {
        let body = json!({
            "statements": [{
                "statement": statement,
                "parameters": parameters,
            }]
        });
        let body_str = body.to_string();
        let url = self.commit_url();
        let auth = self.auth_header();

        let mut attempt = 0u32;
        loop {
            match self.post_once(&url, &auth, &body_str) {
                Ok(v) => return Ok(v),
                Err(err) if err.retryable && attempt < NEO4J_HTTP_RETRIES => {
                    attempt += 1;
                    thread::sleep(Duration::from_millis(NEO4J_BACKOFF_MS * u64::from(attempt)));
                }
                Err(err) => {
                    return Err(CoreError::io(err.message));
                }
            }
        }
    }

    fn post_once(&self, url: &str, auth: &str, body: &str) -> Result<Value, HttpAttemptError> {
        match &self.transport {
            HttpTransport::Ureq(agent) => post_ureq(agent, url, auth, body),
            #[cfg(test)]
            HttpTransport::Scripted(scripted) => {
                let mut guard = scripted
                    .lock()
                    .map_err(|_| HttpAttemptError::fatal("scripted lock poisoned"))?;
                guard
                    .hits
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let outcome = guard.outcomes.pop_front().ok_or_else(|| {
                    HttpAttemptError::fatal("scripted HTTP exhausted")
                })?;
                match outcome {
                    ScriptedOutcome::Status(code) if code >= 500 => {
                        Err(HttpAttemptError::retryable(format!("neo4j HTTP {code}")))
                    }
                    ScriptedOutcome::Status(code) => {
                        Err(HttpAttemptError::fatal(format!("neo4j HTTP {code}")))
                    }
                    ScriptedOutcome::Timeout => Err(HttpAttemptError::retryable(
                        "neo4j transport: timeout".to_string(),
                    )),
                    ScriptedOutcome::OkBody(text) => parse_tx_response(&text),
                }
            }
        }
    }
}

fn post_ureq(
    agent: &ureq::Agent,
    url: &str,
    auth: &str,
    body: &str,
) -> Result<Value, HttpAttemptError> {
    let result = agent
        .post(url)
        .set("Authorization", auth)
        .set("Content-Type", "application/json")
        .set("Accept", "application/json")
        .send_string(body);

    match result {
        Ok(resp) => {
            let text = resp
                .into_string()
                .map_err(|e| HttpAttemptError::fatal(format!("neo4j response read: {e}")))?;
            parse_tx_response(&text)
        }
        Err(ureq::Error::Status(code, resp)) => {
            let _ = resp.into_string();
            if code >= 500 {
                Err(HttpAttemptError::retryable(format!("neo4j HTTP {code}")))
            } else {
                Err(HttpAttemptError::fatal(format!("neo4j HTTP {code}")))
            }
        }
        Err(ureq::Error::Transport(t)) => {
            Err(HttpAttemptError::retryable(format!("neo4j transport: {t}")))
        }
    }
}

struct HttpAttemptError {
    message: String,
    retryable: bool,
}

impl HttpAttemptError {
    fn retryable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: true,
        }
    }

    fn fatal(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retryable: false,
        }
    }
}

fn parse_tx_response(text: &str) -> Result<Value, HttpAttemptError> {
    let v: Value = serde_json::from_str(text)
        .map_err(|e| HttpAttemptError::fatal(format!("neo4j JSON: {e}")))?;
    if let Some(errors) = v.get("errors").and_then(|e| e.as_array()) {
        if let Some(first) = errors.first() {
            let msg = first
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("neo4j query error");
            return Err(HttpAttemptError::fatal(msg.to_string()));
        }
    }
    Ok(v)
}

/// Validate URL scheme allowlist (`http` | `https`) and non-empty host.
pub fn validate_neo4j_url(url: &str) -> CoreResult<()> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(CoreError::invalid_input("neo4j url must not be empty"));
    }
    let lower = trimmed.to_ascii_lowercase();
    let scheme_len = if lower.starts_with("https://") {
        "https://".len()
    } else if lower.starts_with("http://") {
        "http://".len()
    } else {
        return Err(CoreError::invalid_input(
            "neo4j url scheme must be http or https",
        ));
    };
    let rest = &trimmed[scheme_len..];
    let authority = rest.split('/').next().unwrap_or("");
    let hostport = authority.rsplit('@').next().unwrap_or("");
    let host = if hostport.starts_with('[') {
        hostport
            .split(']')
            .next()
            .unwrap_or("")
            .trim_start_matches('[')
    } else {
        hostport.split(':').next().unwrap_or("")
    };
    if host.is_empty() {
        return Err(CoreError::invalid_input(
            "neo4j url host must not be empty",
        ));
    }
    Ok(())
}

fn writes_unsupported<T>() -> CoreResult<T> {
    Err(CoreError::invalid_input(MSG_NEO4J_WRITES))
}

fn row_to_node(row: &Value) -> Option<GraphNode> {
    if let Some(obj) = row.as_object() {
        if obj.contains_key("id") || obj.contains_key("n") {
            if let Some(n) = obj.get("n") {
                return props_to_node(n);
            }
            return props_to_node(row);
        }
    }
    if let Some(arr) = row.as_array() {
        if let Some(first) = arr.first() {
            return props_to_node(first);
        }
    }
    props_to_node(row)
}

fn props_to_node(v: &Value) -> Option<GraphNode> {
    let obj = v.as_object()?;
    let id = obj.get("id")?.as_str()?.to_string();
    let node_type = obj
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("concept")
        .to_string();
    let label = obj
        .get("label")
        .and_then(|t| t.as_str())
        .unwrap_or(id.as_str())
        .to_string();
    let description = obj
        .get("description")
        .and_then(|t| t.as_str())
        .map(str::to_string);
    let metadata = match obj.get("metadata") {
        Some(Value::Object(m)) => m.clone(),
        Some(Value::String(s)) => serde_json::from_str(s).unwrap_or_default(),
        _ => Map::new(),
    };
    let created_at = obj
        .get("createdAt")
        .or_else(|| obj.get("created_at"))
        .and_then(|t| t.as_str())
        .map(str::to_string);
    let updated_at = obj
        .get("updatedAt")
        .or_else(|| obj.get("updated_at"))
        .and_then(|t| t.as_str())
        .map(str::to_string);
    Some(GraphNode {
        id,
        node_type,
        label,
        description,
        vector: None,
        metadata,
        created_at,
        updated_at,
    })
}

fn row_to_edge(row: &Value) -> Option<GraphEdge> {
    let obj = match row {
        Value::Object(m) => m,
        Value::Array(a) => {
            if a.len() < 4 {
                return None;
            }
            return Some(GraphEdge {
                id: a[0].as_str()?.to_string(),
                source_id: a[1].as_str()?.to_string(),
                target_id: a[2].as_str()?.to_string(),
                edge_type: a[3].as_str()?.to_string(),
                weight: a.get(4).and_then(|w| w.as_f64()).unwrap_or(1.0),
                metadata: match a.get(5) {
                    Some(Value::Object(m)) => m.clone(),
                    Some(Value::String(s)) => serde_json::from_str(s).unwrap_or_default(),
                    _ => Map::new(),
                },
            });
        }
        _ => return None,
    };
    let id = obj.get("id")?.as_str()?.to_string();
    let source_id = obj
        .get("sourceId")
        .or_else(|| obj.get("source_id"))
        .and_then(|t| t.as_str())?
        .to_string();
    let target_id = obj
        .get("targetId")
        .or_else(|| obj.get("target_id"))
        .and_then(|t| t.as_str())?
        .to_string();
    let edge_type = obj.get("type").and_then(|t| t.as_str())?.to_string();
    let weight = obj.get("weight").and_then(|w| w.as_f64()).unwrap_or(1.0);
    let metadata = match obj.get("metadata") {
        Some(Value::Object(m)) => m.clone(),
        Some(Value::String(s)) => serde_json::from_str(s).unwrap_or_default(),
        _ => Map::new(),
    };
    Some(GraphEdge {
        id,
        source_id,
        target_id,
        edge_type,
        weight,
        metadata,
    })
}

fn extract_rows(response: &Value) -> Vec<Value> {
    let mut out = Vec::new();
    let Some(results) = response.get("results").and_then(|r| r.as_array()) else {
        return out;
    };
    for result in results {
        let columns: Vec<String> = result
            .get("columns")
            .and_then(|c| c.as_array())
            .map(|cols| {
                cols.iter()
                    .filter_map(|c| c.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let Some(data) = result.get("data").and_then(|d| d.as_array()) else {
            continue;
        };
        for item in data {
            let Some(row) = item.get("row") else {
                continue;
            };
            if columns.len() == 1 {
                if let Some(arr) = row.as_array() {
                    if let Some(first) = arr.first() {
                        out.push(first.clone());
                        continue;
                    }
                }
                out.push(row.clone());
                continue;
            }
            if let Some(arr) = row.as_array() {
                let mut map = Map::new();
                for (i, col) in columns.iter().enumerate() {
                    if let Some(val) = arr.get(i) {
                        map.insert(col.clone(), val.clone());
                    }
                }
                out.push(Value::Object(map));
            } else {
                out.push(row.clone());
            }
        }
    }
    out
}

impl KnowledgeGraph for Neo4jGraph {
    fn schema_version(&self) -> u32 {
        CURRENT_SCHEMA_VERSION
    }

    fn migrate(&mut self) -> CoreResult<()> {
        writes_unsupported()
    }

    fn add_node(&mut self, _node: GraphNode) -> CoreResult<()> {
        writes_unsupported()
    }

    fn get_node(&self, id: &str) -> CoreResult<Option<GraphNode>> {
        const STMT: &str = "MATCH (n:DareNode {id: $id}) \
            RETURN n.id AS id, n.type AS type, n.label AS label, n.description AS description, \
            n.metadata AS metadata, n.createdAt AS createdAt, n.updatedAt AS updatedAt \
            LIMIT 1";
        let resp = self.tx_commit(STMT, json!({ "id": id }))?;
        let rows = extract_rows(&resp);
        Ok(rows.first().and_then(row_to_node))
    }

    fn query_nodes(
        &self,
        ty: Option<NodeType>,
        limit: Option<usize>,
    ) -> CoreResult<Vec<GraphNode>> {
        let lim = limit.unwrap_or(10_000);
        const STMT: &str = "MATCH (n:DareNode) \
            WHERE $type IS NULL OR n.type = $type \
            RETURN n.id AS id, n.type AS type, n.label AS label, n.description AS description, \
            n.metadata AS metadata, n.createdAt AS createdAt, n.updatedAt AS updatedAt \
            ORDER BY n.id \
            LIMIT $limit";
        let resp = self.tx_commit(
            STMT,
            json!({
                "type": ty.map(|t| t.as_str()),
                "limit": lim as i64,
            }),
        )?;
        Ok(extract_rows(&resp)
            .iter()
            .filter_map(row_to_node)
            .collect())
    }

    fn delete_node(&mut self, _id: &str) -> CoreResult<()> {
        writes_unsupported()
    }

    fn add_edge(&mut self, _edge: GraphEdge) -> CoreResult<()> {
        writes_unsupported()
    }

    fn get_edges(&self, node_id: &str, direction: EdgeDirection) -> CoreResult<Vec<GraphEdge>> {
        let stmt = match direction {
            EdgeDirection::Out => {
                "MATCH (a:DareNode {id: $id})-[r:DareRel]->(b:DareNode) \
                 RETURN r.id AS id, a.id AS sourceId, b.id AS targetId, r.type AS type, \
                 r.weight AS weight, r.metadata AS metadata \
                 ORDER BY r.id"
            }
            EdgeDirection::In => {
                "MATCH (a:DareNode)-[r:DareRel]->(b:DareNode {id: $id}) \
                 RETURN r.id AS id, a.id AS sourceId, b.id AS targetId, r.type AS type, \
                 r.weight AS weight, r.metadata AS metadata \
                 ORDER BY r.id"
            }
            EdgeDirection::Both => {
                "MATCH (a:DareNode)-[r:DareRel]->(b:DareNode) \
                 WHERE a.id = $id OR b.id = $id \
                 RETURN r.id AS id, a.id AS sourceId, b.id AS targetId, r.type AS type, \
                 r.weight AS weight, r.metadata AS metadata \
                 ORDER BY r.id"
            }
        };
        let resp = self.tx_commit(stmt, json!({ "id": node_id }))?;
        Ok(extract_rows(&resp)
            .iter()
            .filter_map(row_to_edge)
            .collect())
    }

    fn load_vectors(&self) -> CoreResult<Vec<VectorRow>> {
        Ok(Vec::new())
    }

    fn get_statistics(&self) -> CoreResult<GraphStatistics> {
        const STMT: &str = "MATCH (n:DareNode) WITH count(n) AS nodes \
            OPTIONAL MATCH ()-[r:DareRel]->() \
            RETURN nodes, count(r) AS edges";
        let resp = self.tx_commit(STMT, json!({}))?;
        let rows = extract_rows(&resp);
        let (total_nodes, total_edges) = rows
            .first()
            .map(|row| {
                let nodes = row
                    .get("nodes")
                    .or_else(|| row.as_array().and_then(|a| a.first()))
                    .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
                    .unwrap_or(0);
                let edges = row
                    .get("edges")
                    .or_else(|| row.as_array().and_then(|a| a.get(1)))
                    .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
                    .unwrap_or(0);
                (nodes, edges)
            })
            .unwrap_or((0, 0));
        Ok(GraphStatistics {
            total_nodes,
            total_edges,
            nodes_by_type: empty_nodes_by_type(),
            edges_by_type: empty_edges_by_type(),
        })
    }

    fn export_document(&self) -> CoreResult<GraphStoreDocument> {
        let nodes = self.query_nodes(None, None)?;
        const STMT: &str = "MATCH (a:DareNode)-[r:DareRel]->(b:DareNode) \
            RETURN r.id AS id, a.id AS sourceId, b.id AS targetId, r.type AS type, \
            r.weight AS weight, r.metadata AS metadata \
            ORDER BY r.id";
        let resp = self.tx_commit(STMT, json!({}))?;
        let edges = extract_rows(&resp)
            .iter()
            .filter_map(row_to_edge)
            .collect();
        Ok(GraphStoreDocument { nodes, edges })
    }

    fn import_document(&mut self, _doc: &GraphStoreDocument) -> CoreResult<()> {
        writes_unsupported()
    }

    fn flush(&mut self) -> CoreResult<()> {
        writes_unsupported()
    }

    fn close(self) -> CoreResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Neo4jConnectConfig;

    fn test_cfg() -> Neo4jConnectConfig {
        Neo4jConnectConfig {
            url: "http://127.0.0.1:7474".into(),
            user: "neo4j".into(),
            password: "p".into(),
            database: NEO4J_DEFAULT_DB.into(),
        }
    }

    #[test]
    fn rejects_bad_scheme() {
        let err = validate_neo4j_url("ftp://localhost:7474").unwrap_err();
        assert!(err.to_string().contains("http or https"));
        let err = validate_neo4j_url("file:///tmp/x").unwrap_err();
        assert!(err.to_string().contains("http or https"));
    }

    #[test]
    fn rejects_empty_host() {
        let err = validate_neo4j_url("http:///db").unwrap_err();
        assert!(err.to_string().contains("host"));
    }

    #[test]
    fn accepts_http_https() {
        validate_neo4j_url("http://localhost:7474").unwrap();
        validate_neo4j_url("https://neo4j.example.com").unwrap();
    }

    #[test]
    fn password_never_in_debug() {
        let cfg = Neo4jConnectConfig {
            url: "http://localhost:7474".into(),
            user: "neo4j".into(),
            password: "super-secret-password".into(),
            database: NEO4J_DEFAULT_DB.into(),
        };
        let dbg = format!("{cfg:?}");
        assert!(!dbg.contains("super-secret-password"));
        assert!(dbg.contains("redacted"));

        let g = Neo4jGraph::connect(&cfg).unwrap();
        let dbg = format!("{g:?}");
        assert!(!dbg.contains("super-secret-password"));
        assert!(dbg.contains("redacted"));
    }

    #[test]
    fn writes_rejected() {
        let mut g = Neo4jGraph::connect(&test_cfg()).unwrap();
        let err = g
            .add_node(GraphNode::new("x", NodeType::Task, "x"))
            .unwrap_err();
        assert!(err.to_string().contains(MSG_NEO4J_WRITES));
        let err = g.migrate().unwrap_err();
        assert!(err.to_string().contains(MSG_NEO4J_WRITES));
    }

    #[test]
    fn retries_on_5xx_then_succeeds() {
        let ok_body =
            r#"{"results":[{"columns":["nodes","edges"],"data":[{"row":[0,0]}]}],"errors":[]}"#;
        let g = Neo4jGraph::connect_scripted(
            &test_cfg(),
            vec![
                ScriptedOutcome::Status(503),
                ScriptedOutcome::Status(503),
                ScriptedOutcome::OkBody(ok_body.into()),
            ],
        )
        .unwrap();
        let stats = g.get_statistics().unwrap();
        assert_eq!(stats.total_nodes, 0);
        assert_eq!(g.scripted_hits(), 3);
    }

    #[test]
    fn retries_exhausted_on_persistent_5xx() {
        let g = Neo4jGraph::connect_scripted(
            &test_cfg(),
            vec![
                ScriptedOutcome::Status(503),
                ScriptedOutcome::Status(503),
                ScriptedOutcome::Status(503),
            ],
        )
        .unwrap();
        let err = g
            .tx_commit("RETURN 1", json!({}))
            .expect_err("should fail after retries");
        assert!(err.to_string().contains("neo4j"));
        assert_eq!(g.scripted_hits(), 1 + NEO4J_HTTP_RETRIES);
    }

    #[test]
    fn timeout_is_retryable_then_fails() {
        let g = Neo4jGraph::connect_scripted(
            &test_cfg(),
            vec![
                ScriptedOutcome::Timeout,
                ScriptedOutcome::Timeout,
                ScriptedOutcome::Timeout,
            ],
        )
        .unwrap();
        let err = g
            .tx_commit("RETURN 1", json!({}))
            .expect_err("timeout should fail");
        assert!(err.to_string().contains("neo4j"));
        assert!(err.to_string().contains("timeout") || err.to_string().contains("transport"));
        assert_eq!(g.scripted_hits(), 1 + NEO4J_HTTP_RETRIES);
    }

    #[test]
    #[ignore = "requires a live Neo4j instance; set NEO4J_URL / NEO4J_USER / NEO4J_PASSWORD"]
    fn neo4j_integration_live() {
        let url = std::env::var("NEO4J_URL").expect("NEO4J_URL");
        let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".into());
        let password = std::env::var("NEO4J_PASSWORD").unwrap_or_default();
        let database =
            std::env::var("NEO4J_DATABASE").unwrap_or_else(|_| NEO4J_DEFAULT_DB.to_string());
        let cfg = Neo4jConnectConfig {
            url,
            user,
            password,
            database,
        };
        let g = Neo4jGraph::connect(&cfg).unwrap();
        let _ = g.get_statistics().unwrap();
        let _ = g.query_nodes(None, Some(1)).unwrap();
    }
}
