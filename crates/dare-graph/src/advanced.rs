//! Advanced GraphRAG queries: locate (decay BFS) and owners (043).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use dare_core::{CoreError, CoreResult};
use serde_json::Value;

use crate::knowledge_graph::{EdgeDirection, KnowledgeGraph};
use crate::search::{
    node_matches_keyword, RankedHit, DEFAULT_FANOUT, DEFAULT_LIMIT, DEFAULT_MAX_HOPS,
    MAX_FANOUT_CAP, MAX_HOPS_CAP, MAX_LIMIT_CAP,
};
use crate::types::EdgeType;

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
        Err(CoreError::invalid_input(
            "decay must be in (0.0, 1.0]",
        ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{canonical_edge_id, canonical_file_node_id};
    use crate::types::{EdgeType, GraphEdge, GraphNode, NodeType};
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
        // score DESC: a (1.0), b (0.7), c (0.49)
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
}
