//! Advanced GraphRAG queries (043): locate, owners, drift.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use dare_core::{CoreError, CoreResult};
use serde::Serialize;
use serde_json::Value;

use crate::knowledge_graph::{EdgeDirection, KnowledgeGraph};
use crate::search::{
    node_matches_keyword, RankedHit, DEFAULT_FANOUT, DEFAULT_LIMIT, DEFAULT_MAX_HOPS,
    MAX_FANOUT_CAP, MAX_HOPS_CAP, MAX_LIMIT_CAP,
};
use crate::types::{EdgeType, NodeType};

/// Hop decay factor for [`locate`] (score = `1.0 * LOCATE_DECAY.powi(hop)`).
pub const LOCATE_DECAY: f64 = 0.7;

/// Options for [`locate`].
#[derive(Debug, Clone)]
pub struct LocateOptions {
    pub query: String,
    pub max_hops: usize,
    pub fanout: usize,
    pub limit: usize,
    /// Default [`LOCATE_DECAY`]; must be in `(0.0, 1.0]`.
    pub decay: f64,
}

impl Default for LocateOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            max_hops: DEFAULT_MAX_HOPS,
            fanout: DEFAULT_FANOUT,
            limit: DEFAULT_LIMIT,
            decay: LOCATE_DECAY,
        }
    }
}

impl LocateOptions {
    fn clamped(&self) -> Self {
        Self {
            query: self.query.clone(),
            max_hops: self.max_hops.clamp(0, MAX_HOPS_CAP),
            fanout: self.fanout.clamp(1, MAX_FANOUT_CAP),
            limit: self.limit.clamp(1, MAX_LIMIT_CAP),
            decay: self.decay,
        }
    }
}

fn validate_decay(decay: f64) -> CoreResult<()> {
    if decay > 0.0 && decay <= 1.0 {
        Ok(())
    } else {
        Err(CoreError::invalid_input("decay must be in (0.0, 1.0]"))
    }
}

/// Keyword seeds (hop 0, score 1.0) + BFS neighbors with `score = 1.0 * decay^hop`.
///
/// Aggregates the **max** score per id; result sorted score DESC, id ASC; capped by `limit`.
pub fn locate(g: &dyn KnowledgeGraph, opts: &LocateOptions) -> CoreResult<Vec<RankedHit>> {
    let q = opts.query.trim();
    if q.is_empty() {
        return Err(CoreError::invalid_input("query must not be empty"));
    }
    validate_decay(opts.decay)?;
    let opts = opts.clamped();

    let nodes = g.query_nodes(None, None)?;
    let mut seeds: Vec<String> = nodes
        .into_iter()
        .filter(|n| node_matches_keyword(n, q))
        .map(|n| n.id)
        .collect();
    if seeds.is_empty() {
        return Ok(Vec::new());
    }
    seeds.sort();
    seeds.dedup();

    let mut scores: BTreeMap<String, f64> = BTreeMap::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    let mut visited: BTreeSet<String> = BTreeSet::new();

    for s in &seeds {
        scores.insert(s.clone(), 1.0);
        if visited.insert(s.clone()) {
            queue.push_back((s.clone(), 0));
        }
    }

    while let Some((node_id, depth)) = queue.pop_front() {
        if depth >= opts.max_hops {
            continue;
        }
        let mut edges = g.get_edges(&node_id, EdgeDirection::Both)?;
        edges.sort_by(|a, b| a.id.cmp(&b.id));
        let mut neighbors: Vec<String> = Vec::new();
        for e in edges {
            let other = if e.source_id == node_id {
                e.target_id.clone()
            } else {
                e.source_id.clone()
            };
            if other != node_id {
                neighbors.push(other);
            }
        }
        neighbors.sort();
        neighbors.dedup();
        let next_hop = depth + 1;
        let cand = 1.0 * opts.decay.powi(next_hop as i32);
        for (i, nb) in neighbors.into_iter().enumerate() {
            if i >= opts.fanout {
                break;
            }
            let entry = scores.entry(nb.clone()).or_insert(cand);
            if cand > *entry {
                *entry = cand;
            }
            if visited.insert(nb.clone()) {
                queue.push_back((nb, next_hop));
            }
        }
    }

    let mut ranked: Vec<(String, f64)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    let mut hits = Vec::new();
    for (id, score) in ranked.into_iter().take(opts.limit) {
        if let Some(n) = g.get_node(&id)? {
            hits.push(RankedHit {
                id,
                score,
                label: n.label,
                node_type: n.node_type,
            });
        } else {
            hits.push(RankedHit {
                id: id.clone(),
                score,
                label: id,
                node_type: String::new(),
            });
        }
    }
    Ok(hits)
}

/// Owners of `seed`: metadata `"owner"` (raw trimmed string) + incoming `Contains` sources.
///
/// Unique, sorted ASC. Unknown seed → `InvalidInput("unknown node")`.
pub fn owners(g: &dyn KnowledgeGraph, seed: &str) -> CoreResult<Vec<String>> {
    let Some(node) = g.get_node(seed)? else {
        return Err(CoreError::invalid_input("unknown node"));
    };

    let mut out: BTreeSet<String> = BTreeSet::new();

    if let Some(Value::String(owner)) = node.metadata.get("owner") {
        let trimmed = owner.trim();
        if !trimmed.is_empty() {
            out.insert(trimmed.to_string());
        }
    }

    let edges = g.get_edges(seed, EdgeDirection::In)?;
    let contains = EdgeType::Contains.as_str();
    for e in edges {
        if e.edge_type == contains && e.target_id == seed {
            out.insert(e.source_id);
        }
    }

    Ok(out.into_iter().collect())
}

/// Options for [`drift`]. Default threshold is `1`.
#[derive(Debug, Clone)]
pub struct DriftOptions {
    /// Violation count threshold used by [`drift_exceeds_threshold`].
    /// `0` means any positive violation count exceeds.
    pub threshold: u32,
}

impl Default for DriftOptions {
    fn default() -> Self {
        Self { threshold: 1 }
    }
}

/// Drift classification report (JSON camelCase).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftReport {
    pub orphan_requirements: Vec<String>,
    pub orphan_code: Vec<String>,
    pub stale: Vec<String>,
    pub violations: u32,
    pub threshold: u32,
}

fn is_stale_metadata(value: &Value) -> bool {
    match value {
        Value::Bool(true) => true,
        Value::String(s) => s.eq_ignore_ascii_case("true"),
        _ => false,
    }
}

/// Classify orphan requirements, orphan code, and stale nodes.
///
/// Always returns `Ok(report)` when the graph is readable; exit code 7 is CLI-only.
pub fn drift(g: &dyn KnowledgeGraph, opts: &DriftOptions) -> CoreResult<DriftReport> {
    let doc = g.export_document()?;
    let implements = EdgeType::Implements.as_str();

    let mut implements_out: BTreeSet<&str> = BTreeSet::new();
    let mut implements_in: BTreeSet<&str> = BTreeSet::new();
    for e in &doc.edges {
        if e.edge_type == implements {
            implements_out.insert(e.source_id.as_str());
            implements_in.insert(e.target_id.as_str());
        }
    }

    let req_ty = NodeType::Requirement.as_str();
    let file_ty = NodeType::File.as_str();
    let symbol_ty = NodeType::CodeSymbol.as_str();

    let mut orphan_requirements: BTreeSet<String> = BTreeSet::new();
    let mut orphan_code: BTreeSet<String> = BTreeSet::new();
    let mut stale: BTreeSet<String> = BTreeSet::new();

    for n in &doc.nodes {
        if n.node_type == req_ty && !implements_out.contains(n.id.as_str()) {
            orphan_requirements.insert(n.id.clone());
        }
        if (n.node_type == file_ty || n.node_type == symbol_ty)
            && !implements_in.contains(n.id.as_str())
        {
            orphan_code.insert(n.id.clone());
        }
        if n.metadata.get("stale").is_some_and(is_stale_metadata) {
            stale.insert(n.id.clone());
        }
    }

    let orphan_requirements: Vec<String> = orphan_requirements.into_iter().collect();
    let orphan_code: Vec<String> = orphan_code.into_iter().collect();
    let stale: Vec<String> = stale.into_iter().collect();
    let violations = (orphan_requirements.len() + orphan_code.len() + stale.len()) as u32;

    Ok(DriftReport {
        orphan_requirements,
        orphan_code,
        stale,
        violations,
        threshold: opts.threshold,
    })
}

/// Helper for CLI strict mode: `threshold == 0` ⇒ `violations > 0`; else `violations >= threshold`.
pub fn drift_exceeds_threshold(report: &DriftReport) -> bool {
    if report.threshold == 0 {
        report.violations > 0
    } else {
        report.violations >= report.threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{canonical_edge_id, canonical_file_node_id};
    use crate::types::{GraphEdge, GraphNode};
    use crate::{load_graph_config, open_graph};
    use dare_core::ProjectRoot;
    use serde_json::json;
    use tempfile::tempdir;

    fn seed_locate_chain(root: &ProjectRoot) -> crate::GraphHandle {
        let cfg = load_graph_config(root, None).unwrap();
        let mut g = open_graph(root, &cfg).unwrap();
        g.migrate().unwrap();
        let a = canonical_file_node_id("src/seed.rs");
        let b = canonical_file_node_id("src/mid.rs");
        let c = canonical_file_node_id("src/leaf.rs");
        g.add_node(GraphNode::new(a.clone(), NodeType::File, "seed module"))
            .unwrap();
        g.add_node(GraphNode::new(b.clone(), NodeType::File, "mid"))
            .unwrap();
        g.add_node(GraphNode::new(c.clone(), NodeType::File, "leaf"))
            .unwrap();
        g.add_edge(GraphEdge::new(
            canonical_edge_id(EdgeType::RelatedTo.as_str(), &a, &b),
            a.clone(),
            b.clone(),
            EdgeType::RelatedTo,
        ))
        .unwrap();
        g.add_edge(GraphEdge::new(
            canonical_edge_id(EdgeType::RelatedTo.as_str(), &b, &c),
            b,
            c,
            EdgeType::RelatedTo,
        ))
        .unwrap();
        g.flush().unwrap();
        g
    }

    #[test]
    fn locate_decay_scores() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let g = seed_locate_chain(&root);
        let a = canonical_file_node_id("src/seed.rs");
        let b = canonical_file_node_id("src/mid.rs");
        let c = canonical_file_node_id("src/leaf.rs");

        let hits = locate(
            &g,
            &LocateOptions {
                query: "seed".into(),
                max_hops: 2,
                fanout: 50,
                limit: 20,
                decay: LOCATE_DECAY,
            },
        )
        .unwrap();

        assert_eq!(hits.len(), 3);
        let by_id: BTreeMap<_, _> = hits.iter().map(|h| (h.id.as_str(), h.score)).collect();
        assert!((by_id[a.as_str()] - 1.0).abs() < 1e-12);
        assert!((by_id[b.as_str()] - 0.7).abs() < 1e-12);
        assert!((by_id[c.as_str()] - 0.49).abs() < 1e-12);
        assert_eq!(hits[0].id, a);
        assert_eq!(hits[1].id, b);
        assert_eq!(hits[2].id, c);
    }

    #[test]
    fn locate_empty_query() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let g = seed_locate_chain(&root);
        let err = locate(
            &g,
            &LocateOptions {
                query: "   ".into(),
                ..LocateOptions::default()
            },
        )
        .unwrap_err();
        assert!(
            matches!(err, CoreError::InvalidInput(ref m) if m.contains("query must not be empty"))
        );
    }

    #[test]
    fn owners_contains_and_metadata() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let cfg = load_graph_config(&root, None).unwrap();
        let mut g = open_graph(&root, &cfg).unwrap();
        g.migrate().unwrap();

        let parent = canonical_file_node_id("src/parent.rs");
        let child = canonical_file_node_id("src/child.rs");
        g.add_node(GraphNode::new(parent.clone(), NodeType::File, "parent"))
            .unwrap();
        let mut child_node = GraphNode::new(child.clone(), NodeType::File, "child");
        child_node
            .metadata
            .insert("owner".into(), json!("  alice  "));
        g.add_node(child_node).unwrap();
        g.add_edge(GraphEdge::new(
            canonical_edge_id(EdgeType::Contains.as_str(), &parent, &child),
            parent.clone(),
            child.clone(),
            EdgeType::Contains,
        ))
        .unwrap();
        g.flush().unwrap();

        let got = owners(&g, &child).unwrap();
        let mut expected = vec!["alice".to_string(), parent];
        expected.sort();
        assert_eq!(got, expected);

        let err = owners(&g, "missing:node").unwrap_err();
        assert!(matches!(
            err,
            CoreError::InvalidInput(ref m) if m.contains("unknown node")
        ));
    }

    #[test]
    fn drift_three_buckets() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let cfg = load_graph_config(&root, None).unwrap();
        let mut g = open_graph(&root, &cfg).unwrap();
        g.migrate().unwrap();

        let req_ok = "requirement:r-ok".to_string();
        let file_ok = "file:src/ok.rs".to_string();
        g.add_node(GraphNode::new(req_ok.clone(), NodeType::Requirement, "ok"))
            .unwrap();
        g.add_node(GraphNode::new(file_ok.clone(), NodeType::File, "ok.rs"))
            .unwrap();
        g.add_edge(GraphEdge::new(
            canonical_edge_id(EdgeType::Implements.as_str(), &req_ok, &file_ok),
            req_ok.clone(),
            file_ok.clone(),
            EdgeType::Implements,
        ))
        .unwrap();

        let req_orphan = "requirement:r-orphan".to_string();
        g.add_node(GraphNode::new(
            req_orphan.clone(),
            NodeType::Requirement,
            "orphan",
        ))
        .unwrap();

        let file_orphan = "file:src/orphan.rs".to_string();
        let sym_orphan = "code_symbol:src/orphan.rs::f".to_string();
        g.add_node(GraphNode::new(
            file_orphan.clone(),
            NodeType::File,
            "orphan.rs",
        ))
        .unwrap();
        g.add_node(GraphNode::new(
            sym_orphan.clone(),
            NodeType::CodeSymbol,
            "f",
        ))
        .unwrap();

        let mut stale_bool = GraphNode::new("file:stale-bool.rs", NodeType::File, "stale-bool");
        stale_bool
            .metadata
            .insert("stale".into(), Value::Bool(true));
        g.add_node(stale_bool).unwrap();

        let mut stale_str = GraphNode::new("task:stale-str", NodeType::Task, "stale-str");
        stale_str
            .metadata
            .insert("stale".into(), json!("TRUE"));
        g.add_node(stale_str).unwrap();

        let mut not_stale = GraphNode::new("task:fresh", NodeType::Task, "fresh");
        not_stale
            .metadata
            .insert("stale".into(), json!("false"));
        g.add_node(not_stale).unwrap();

        g.flush().unwrap();

        let report = drift(&g, &DriftOptions::default()).unwrap();

        assert_eq!(report.orphan_requirements, vec![req_orphan]);
        assert_eq!(
            report.orphan_code,
            vec![
                "code_symbol:src/orphan.rs::f".to_string(),
                "file:src/orphan.rs".to_string(),
                "file:stale-bool.rs".to_string(),
            ]
        );
        assert_eq!(
            report.stale,
            vec![
                "file:stale-bool.rs".to_string(),
                "task:stale-str".to_string(),
            ]
        );
        assert_eq!(report.threshold, 1);
        assert_eq!(
            report.violations,
            (report.orphan_requirements.len()
                + report.orphan_code.len()
                + report.stale.len()) as u32
        );
        assert!(!report.orphan_requirements.contains(&req_ok));
        assert!(!report.orphan_code.contains(&file_ok));
        assert!(!report.stale.iter().any(|id| id == "task:fresh"));
    }

    #[test]
    fn drift_exceeds_threshold_cases() {
        let mk = |violations: u32, threshold: u32| DriftReport {
            orphan_requirements: vec![],
            orphan_code: vec![],
            stale: vec![],
            violations,
            threshold,
        };

        assert!(!drift_exceeds_threshold(&mk(0, 0)));
        assert!(drift_exceeds_threshold(&mk(1, 0)));
        assert!(!drift_exceeds_threshold(&mk(0, 1)));
        assert!(drift_exceeds_threshold(&mk(1, 1)));
        assert!(!drift_exceeds_threshold(&mk(1, 2)));
        assert!(drift_exceeds_threshold(&mk(2, 2)));
        assert!(drift_exceeds_threshold(&mk(5, 3)));
    }
}
