//! Keyword search, BFS traverse, and RRF fusion (no semantic embeddings).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use dare_core::CoreResult;

use crate::knowledge_graph::{EdgeDirection, KnowledgeGraph};
use crate::types::GraphNode;

/// Reciprocal Rank Fusion constant (TS / Mestre: `1/(60+rank)`).
pub const RRF_K: u32 = 60;

pub const DEFAULT_MAX_HOPS: usize = 2;
pub const MAX_HOPS_CAP: usize = 5;
pub const DEFAULT_FANOUT: usize = 50;
pub const MAX_FANOUT_CAP: usize = 200;
pub const DEFAULT_LIMIT: usize = 20;
pub const MAX_LIMIT_CAP: usize = 100;

/// Options for hybrid / keyword queries.
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub limit: usize,
    pub max_hops: usize,
    pub fanout: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: DEFAULT_LIMIT,
            max_hops: DEFAULT_MAX_HOPS,
            fanout: DEFAULT_FANOUT,
        }
    }
}

impl SearchOptions {
    pub fn clamped(self) -> Self {
        Self {
            limit: self.limit.clamp(1, MAX_LIMIT_CAP),
            max_hops: self.max_hops.clamp(0, MAX_HOPS_CAP),
            fanout: self.fanout.clamp(1, MAX_FANOUT_CAP),
        }
    }
}

/// One ranked hit (deterministic: score DESC, id ASC).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedHit {
    pub id: String,
    pub score: f64,
    pub label: String,
    pub node_type: String,
}

/// Case-insensitive LIKE-style match against id / label / description.
pub fn node_matches_keyword(node: &GraphNode, query: &str) -> bool {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return false;
    }
    let id = node.id.to_ascii_lowercase();
    let label = node.label.to_ascii_lowercase();
    let desc = node
        .description
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();
    id.contains(&q) || label.contains(&q) || desc.contains(&q)
}

/// Keyword search over all nodes (LIKE parity). Ranking: id ASC as initial order (rank 1..n).
pub fn keyword_search(
    graph: &dyn KnowledgeGraph,
    query: &str,
    limit: usize,
) -> CoreResult<Vec<RankedHit>> {
    let limit = limit.clamp(1, MAX_LIMIT_CAP);
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let mut nodes = graph.query_nodes(None, None)?;
    nodes.retain(|n| node_matches_keyword(n, q));
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out = Vec::new();
    for (i, n) in nodes.into_iter().take(limit).enumerate() {
        let rank = (i + 1) as u32;
        let score = 1.0 / f64::from(RRF_K + rank);
        out.push(RankedHit {
            id: n.id,
            score,
            label: n.label,
            node_type: n.node_type,
        });
    }
    Ok(out)
}

/// BFS from seeds with hop and fanout caps. Returns visited node ids (deterministic).
pub fn bfs_expand(
    graph: &dyn KnowledgeGraph,
    seeds: &[String],
    max_hops: usize,
    fanout: usize,
) -> CoreResult<Vec<String>> {
    let max_hops = max_hops.clamp(0, MAX_HOPS_CAP);
    let fanout = fanout.clamp(1, MAX_FANOUT_CAP);
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut order: Vec<String> = Vec::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();

    let mut seed_sorted: Vec<String> = seeds.to_vec();
    seed_sorted.sort();
    seed_sorted.dedup();
    for s in seed_sorted {
        if visited.insert(s.clone()) {
            order.push(s.clone());
            queue.push_back((s, 0));
        }
    }

    while let Some((node_id, depth)) = queue.pop_front() {
        if depth >= max_hops {
            continue;
        }
        let mut edges = graph.get_edges(&node_id, EdgeDirection::Both)?;
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
        for (i, nb) in neighbors.into_iter().enumerate() {
            if i >= fanout {
                break;
            }
            if visited.insert(nb.clone()) {
                order.push(nb.clone());
                queue.push_back((nb, depth + 1));
            }
        }
    }
    Ok(order)
}

/// Reciprocal Rank Fusion over multiple rankings (each list ordered best→worst).
pub fn rrf_fuse(rankings: &[Vec<String>], k: u32) -> Vec<(String, f64)> {
    let k = if k == 0 { RRF_K } else { k };
    let mut scores: BTreeMap<String, f64> = BTreeMap::new();
    for ranking in rankings {
        for (i, id) in ranking.iter().enumerate() {
            let rank = (i + 1) as u32;
            let add = 1.0 / f64::from(k + rank);
            *scores.entry(id.clone()).or_insert(0.0) += add;
        }
    }
    let mut fused: Vec<(String, f64)> = scores.into_iter().collect();
    fused.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    fused
}

/// Hybrid query: keyword hits + BFS expansion from those seeds, fused by RRF (no semantic).
pub fn hybrid_query(
    graph: &dyn KnowledgeGraph,
    query: &str,
    opts: &SearchOptions,
) -> CoreResult<Vec<RankedHit>> {
    let opts = opts.clone().clamped();
    let kw = keyword_search(graph, query, opts.limit)?;
    let seed_ids: Vec<String> = kw.iter().map(|h| h.id.clone()).collect();
    let expanded = bfs_expand(graph, &seed_ids, opts.max_hops, opts.fanout)?;

    let kw_ranking: Vec<String> = seed_ids.clone();
    // Graph ranking: seeds first (already in expanded), then neighbors in BFS order.
    let graph_ranking = expanded;

    let fused = rrf_fuse(&[kw_ranking, graph_ranking], RRF_K);
    let mut hits = Vec::new();
    for (id, score) in fused.into_iter().take(opts.limit) {
        if let Some(n) = graph.get_node(&id)? {
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

/// Render a Mermaid flowchart of a node/edge subset (deterministic).
pub fn render_mermaid_subset(graph: &dyn KnowledgeGraph, max_nodes: usize) -> CoreResult<String> {
    let max_nodes = max_nodes.clamp(1, 500);
    let nodes = graph.query_nodes(None, Some(max_nodes))?;
    let ids: BTreeSet<String> = nodes.iter().map(|n| n.id.clone()).collect();
    let mut lines = vec![
        "flowchart LR".to_string(),
        "%% dare graph viz (microplano 041)".to_string(),
    ];
    for n in &nodes {
        let safe_id = mermaid_id(&n.id);
        let label = escape_label(&n.label);
        lines.push(format!("  {safe_id}[\"{label}\"]"));
    }
    let mut edge_lines = Vec::new();
    for n in &nodes {
        let edges = graph.get_edges(&n.id, EdgeDirection::Out)?;
        for e in edges {
            if ids.contains(&e.target_id) {
                edge_lines.push(format!(
                    "  {} --> {}",
                    mermaid_id(&e.source_id),
                    mermaid_id(&e.target_id)
                ));
            }
        }
    }
    edge_lines.sort();
    edge_lines.dedup();
    lines.extend(edge_lines);
    Ok(lines.join("\n") + "\n")
}

fn mermaid_id(id: &str) -> String {
    let mut s: String = id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    if s.is_empty() || s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        s = format!("n_{s}");
    }
    s
}

fn escape_label(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "'").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{canonical_edge_id, canonical_file_node_id};
    use crate::types::{EdgeType, GraphEdge, GraphNode, NodeType};
    use crate::{load_graph_config, open_graph};
    use dare_core::ProjectRoot;
    use tempfile::tempdir;

    fn seed_graph(root: &ProjectRoot) -> crate::GraphHandle {
        let cfg = load_graph_config(root, None).unwrap();
        let mut g = open_graph(root, &cfg).unwrap();
        g.migrate().unwrap();
        // A --contains--> B --related_to--> C ; keyword "alpha" on A and C labels
        let a = canonical_file_node_id("src/alpha.rs");
        let b = "code_symbol:src/alpha.rs::helper".to_string();
        let c = canonical_file_node_id("src/other.rs");
        g.add_node(GraphNode::new(a.clone(), NodeType::File, "alpha module"))
            .unwrap();
        g.add_node(GraphNode::new(b.clone(), NodeType::CodeSymbol, "helper"))
            .unwrap();
        let mut cnode = GraphNode::new(c.clone(), NodeType::File, "beta");
        cnode.description = Some("mentions alpha in docs".into());
        g.add_node(cnode).unwrap();
        g.add_edge(GraphEdge::new(
            canonical_edge_id(EdgeType::Contains.as_str(), &a, &b),
            a.clone(),
            b.clone(),
            EdgeType::Contains,
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
    fn rrf_k60_formula() {
        let fused = rrf_fuse(
            &[vec!["a".into(), "b".into()], vec!["b".into(), "c".into()]],
            60,
        );
        // b appears in both → highest (rank2 in list1 + rank1 in list2)
        assert_eq!(fused[0].0, "b");
        let score_b = 1.0 / 62.0 + 1.0 / 61.0;
        assert!((fused[0].1 - score_b).abs() < 1e-12);
    }

    #[test]
    fn bfs_respects_max_hops_and_fanout() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let g = seed_graph(&root);
        let a = canonical_file_node_id("src/alpha.rs");
        let hops0 = bfs_expand(&g, &[a.clone()], 0, 50).unwrap();
        assert_eq!(hops0, vec![a.clone()]);
        let hops2 = bfs_expand(&g, &[a], 2, 50).unwrap();
        assert!(hops2.len() >= 3);
    }

    #[test]
    fn golden_hybrid_ranking_order() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let g = seed_graph(&root);
        let hits = hybrid_query(&g, "alpha", &SearchOptions::default()).unwrap();
        assert!(!hits.is_empty());
        let ids: Vec<String> = hits.iter().map(|h| h.id.clone()).collect();
        let a = canonical_file_node_id("src/alpha.rs");
        let b = "code_symbol:src/alpha.rs::helper".to_string();
        let c = canonical_file_node_id("src/other.rs");
        assert!(ids.contains(&a));
        assert!(ids.contains(&b));
        assert!(ids.contains(&c));
        // Golden: deterministic fused order for this fixture (keyword + BFS RRF).
        let golden = vec![b.clone(), a.clone(), c.clone()];
        assert_eq!(ids, golden);
        let hits2 = hybrid_query(&g, "alpha", &SearchOptions::default()).unwrap();
        assert_eq!(
            hits2.iter().map(|h| h.id.clone()).collect::<Vec<_>>(),
            golden
        );
    }

    #[test]
    fn search_options_clamp() {
        let o = SearchOptions {
            limit: 999,
            max_hops: 99,
            fanout: 9999,
        }
        .clamped();
        assert_eq!(o.limit, MAX_LIMIT_CAP);
        assert_eq!(o.max_hops, MAX_HOPS_CAP);
        assert_eq!(o.fanout, MAX_FANOUT_CAP);
    }
}
