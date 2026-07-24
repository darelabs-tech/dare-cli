//! Keyword search, BFS traverse, and RRF fusion (semantic channel prepared in 042).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use dare_core::{CoreError, CoreResult};

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

/// Prefix for soft-fail warnings when the semantic channel is skipped.
pub const MSG_SEMANTIC_UNAVAILABLE: &str = "semantic unavailable: ";

/// Options for hybrid / keyword queries.
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub limit: usize,
    pub max_hops: usize,
    pub fanout: usize,
    /// When true, skip the vector channel even if semantic is available.
    pub no_semantic: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: DEFAULT_LIMIT,
            max_hops: DEFAULT_MAX_HOPS,
            fanout: DEFAULT_FANOUT,
            no_semantic: false,
        }
    }
}

impl SearchOptions {
    pub fn clamped(self) -> Self {
        Self {
            limit: self.limit.clamp(1, MAX_LIMIT_CAP),
            max_hops: self.max_hops.clamp(0, MAX_HOPS_CAP),
            fanout: self.fanout.clamp(1, MAX_FANOUT_CAP),
            no_semantic: self.no_semantic,
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

/// Cosine similarity over equal-length finite vectors.
///
/// Returns `0.0` on length mismatch, zero-norm, or non-finite inputs/results (never NaN).
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0_f64;
    let mut norm_a = 0.0_f64;
    let mut norm_b = 0.0_f64;
    for (&x, &y) in a.iter().zip(b.iter()) {
        let xf = f64::from(x);
        let yf = f64::from(y);
        if !xf.is_finite() || !yf.is_finite() {
            return 0.0;
        }
        dot += xf * yf;
        norm_a += xf * xf;
        norm_b += yf * yf;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 || !denom.is_finite() {
        return 0.0;
    }
    let score = dot / denom;
    if score.is_finite() {
        score
    } else {
        0.0
    }
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

/// Hybrid query: keyword + BFS RRF (and optional vector list when available).
///
/// Thin wrapper over [`hybrid_query_with_warnings`] that drops warnings.
pub fn hybrid_query(
    graph: &dyn KnowledgeGraph,
    query: &str,
    opts: &SearchOptions,
) -> CoreResult<Vec<RankedHit>> {
    let (hits, _warnings) = hybrid_query_with_warnings(graph, query, opts)?;
    Ok(hits)
}

/// Same as [`hybrid_query`], returning soft-fail warnings for the semantic channel.
///
/// When `no_semantic` or semantic is unavailable: 2-list RRF identical to microplano 041.
pub fn hybrid_query_with_warnings(
    graph: &dyn KnowledgeGraph,
    query: &str,
    opts: &SearchOptions,
) -> CoreResult<(Vec<RankedHit>, Vec<String>)> {
    if query.trim().is_empty() {
        return Err(CoreError::invalid_input("query must not be empty"));
    }
    let opts = opts.clone().clamped();
    let kw = keyword_search(graph, query, opts.limit)?;
    let seed_ids: Vec<String> = kw.iter().map(|h| h.id.clone()).collect();
    let expanded = bfs_expand(graph, &seed_ids, opts.max_hops, opts.fanout)?;

    let kw_ranking: Vec<String> = seed_ids;
    let graph_ranking = expanded;

    let mut warnings: Vec<String> = Vec::new();
    let mut rankings: Vec<Vec<String>> = vec![kw_ranking, graph_ranking];

    if !opts.no_semantic {
        match try_semantic_vector_ranking() {
            Ok(Some(vector_ids)) => {
                rankings.push(vector_ids);
            }
            Ok(None) => {
                warnings.push(format!(
                    "{MSG_SEMANTIC_UNAVAILABLE}semantic feature not enabled"
                ));
            }
            Err(reason) => {
                warnings.push(format!("{MSG_SEMANTIC_UNAVAILABLE}{reason}"));
            }
        }
    }

    let fused = rrf_fuse(&rankings, RRF_K);
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
    Ok((hits, warnings))
}

/// Attempt to obtain a vector ranking list for RRF.
///
/// - `Ok(Some(ids))` — use as third RRF list
/// - `Ok(None)` — semantic unavailable (no attempt / feature off)
/// - `Err(reason)` — attempt skipped/failed with a cause string (no `semantic unavailable:` prefix)
#[cfg(test)]
fn try_semantic_vector_ranking() -> Result<Option<Vec<String>>, String> {
    match TEST_VECTOR_RANKING.lock().map(|g| g.clone()) {
        Ok(None) => Ok(None),
        Ok(Some(Ok(ids))) => Ok(Some(ids)),
        Ok(Some(Err(reason))) => Err(reason),
        Err(_) => Ok(None),
    }
}

#[cfg(not(test))]
fn try_semantic_vector_ranking() -> Result<Option<Vec<String>>, String> {
    // Semantic runtime lands in mp042-002/003; until then always unavailable.
    Ok(None)
}

#[cfg(test)]
static TEST_VECTOR_RANKING: std::sync::Mutex<Option<Result<Vec<String>, String>>> =
    std::sync::Mutex::new(None);

/// Test-only hook to inject (or fail) the vector ranking channel.
#[cfg(test)]
pub fn set_test_vector_ranking(value: Option<Result<Vec<String>, String>>) {
    if let Ok(mut guard) = TEST_VECTOR_RANKING.lock() {
        *guard = value;
    }
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

    fn golden_041_ids() -> Vec<String> {
        let a = canonical_file_node_id("src/alpha.rs");
        let b = "code_symbol:src/alpha.rs::helper".to_string();
        let c = canonical_file_node_id("src/other.rs");
        vec![b, a, c]
    }

    #[test]
    fn cosine_zero_norm() {
        let a = [0.0_f32, 0.0, 0.0];
        let b = [1.0_f32, 2.0, 3.0];
        let s = cosine_similarity(&a, &b);
        assert_eq!(s, 0.0);
        assert!(s.is_finite());
        let s2 = cosine_similarity(&b, &a);
        assert_eq!(s2, 0.0);
    }

    #[test]
    fn cosine_len_mismatch() {
        let a = [1.0_f32, 0.0];
        let b = [1.0_f32, 0.0, 0.0];
        let s = cosine_similarity(&a, &b);
        assert_eq!(s, 0.0);
        assert!(!s.is_nan());
    }

    #[test]
    fn cosine_orthogonal_ish() {
        let a = [1.0_f32, 0.0];
        let b = [0.0_f32, 1.0];
        let s = cosine_similarity(&a, &b);
        assert!((s - 0.0).abs() < 1e-12);
        assert!(s.is_finite());
        let parallel = cosine_similarity(&[1.0, 2.0], &[2.0, 4.0]);
        assert!((parallel - 1.0).abs() < 1e-6);
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
        set_test_vector_ranking(None);
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let g = seed_graph(&root);
        let hits = hybrid_query(&g, "alpha", &SearchOptions::default()).unwrap();
        assert!(!hits.is_empty());
        let ids: Vec<String> = hits.iter().map(|h| h.id.clone()).collect();
        let golden = golden_041_ids();
        assert_eq!(ids, golden);
        let hits2 = hybrid_query(&g, "alpha", &SearchOptions::default()).unwrap();
        assert_eq!(
            hits2.iter().map(|h| h.id.clone()).collect::<Vec<_>>(),
            golden
        );
    }

    #[test]
    fn hybrid_no_semantic_matches_041_golden() {
        set_test_vector_ranking(None);
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let g = seed_graph(&root);
        let opts = SearchOptions {
            no_semantic: true,
            ..SearchOptions::default()
        };
        let (hits, warnings) = hybrid_query_with_warnings(&g, "alpha", &opts).unwrap();
        assert!(warnings.is_empty(), "no_semantic must not emit skip warnings");
        let ids: Vec<String> = hits.iter().map(|h| h.id.clone()).collect();
        assert_eq!(ids, golden_041_ids());

        let baseline = hybrid_query(&g, "alpha", &SearchOptions::default()).unwrap();
        assert_eq!(
            hits.iter().map(|h| h.id.clone()).collect::<Vec<_>>(),
            baseline.iter().map(|h| h.id.clone()).collect::<Vec<_>>()
        );
        assert_eq!(
            hits.iter().map(|h| h.score).collect::<Vec<_>>(),
            baseline.iter().map(|h| h.score).collect::<Vec<_>>()
        );
    }

    #[test]
    fn hybrid_query_rejects_empty() {
        set_test_vector_ranking(None);
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let g = seed_graph(&root);
        let err = hybrid_query(&g, "   ", &SearchOptions::default()).unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(ref m) if m.contains("query must not be empty")));
    }

    #[test]
    fn hybrid_vector_hook_injects_third_list() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let g = seed_graph(&root);
        let c = canonical_file_node_id("src/other.rs");
        set_test_vector_ranking(Some(Ok(vec![c.clone()])));
        let result = hybrid_query_with_warnings(&g, "alpha", &SearchOptions::default());
        set_test_vector_ranking(None);
        let (hits, warnings) = result.unwrap();
        assert!(warnings.is_empty());
        assert_eq!(hits[0].id, c);
    }

    #[test]
    fn search_options_clamp() {
        let o = SearchOptions {
            limit: 999,
            max_hops: 99,
            fanout: 9999,
            no_semantic: true,
        }
        .clamped();
        assert_eq!(o.limit, MAX_LIMIT_CAP);
        assert_eq!(o.max_hops, MAX_HOPS_CAP);
        assert_eq!(o.fanout, MAX_FANOUT_CAP);
        assert!(o.no_semantic);
    }
}
