//! Keyword search, BFS traverse, and RRF fusion (semantic channel prepared in 042).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use dare_core::{CoreError, CoreResult};

use dare_core::redact;

use crate::knowledge_graph::{EdgeDirection, KnowledgeGraph};
use crate::semantic::{self, MAX_CANDIDATES};
use crate::types::GraphNode;
use crate::vector;

/// Re-export cosine for callers that historically imported it from `search`.
pub use vector::cosine_similarity;

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
/// With feature `semantic` and model OK: 3-list RRF (keyword ∪ BFS ∪ vector).
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
    let mut rankings: Vec<Vec<String>> = vec![kw_ranking.clone(), graph_ranking.clone()];

    if !opts.no_semantic {
        match resolve_vector_ranking(graph, query, &kw_ranking, &graph_ranking) {
            Ok(Some(vector_ids)) => {
                rankings.push(vector_ids);
            }
            Ok(None) => {
                // Feature off / skipped — parity 041, no warning.
            }
            Err(reason) => {
                let clean = reason
                    .strip_prefix(MSG_SEMANTIC_UNAVAILABLE)
                    .or_else(|| reason.strip_prefix(semantic::MSG_SEMANTIC_UNAVAILABLE))
                    .unwrap_or(reason.as_str());
                warnings.push(format!(
                    "{MSG_SEMANTIC_UNAVAILABLE}{}",
                    redact(clean)
                ));
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

/// Build candidate (id, passage) list: keyword ∪ BFS ids, sorted id ASC, capped at 512.
pub fn semantic_candidates(
    graph: &dyn KnowledgeGraph,
    kw_ids: &[String],
    bfs_ids: &[String],
) -> CoreResult<Vec<(String, String)>> {
    let mut id_set: BTreeSet<String> = BTreeSet::new();
    for id in kw_ids.iter().chain(bfs_ids.iter()) {
        id_set.insert(id.clone());
    }
    let mut ids: Vec<String> = id_set.into_iter().collect();
    ids.sort();
    ids.truncate(MAX_CANDIDATES);

    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        if let Some(n) = graph.get_node(&id)? {
            let passage = semantic::node_passage(&n.label, n.description.as_deref());
            out.push((id, passage));
        } else {
            out.push((id.clone(), semantic::node_passage(&id, None)));
        }
    }
    Ok(out)
}

/// Attempt to obtain a vector ranking list for RRF.
///
/// - `Ok(Some(ids))` — use as third RRF list
/// - `Ok(None)` — semantic skipped (feature off / no attempt)
/// - `Err(reason)` — soft-fail cause (no `semantic unavailable:` prefix required)
fn resolve_vector_ranking(
    graph: &dyn KnowledgeGraph,
    query: &str,
    kw_ids: &[String],
    bfs_ids: &[String],
) -> Result<Option<Vec<String>>, String> {
    #[cfg(test)]
    {
        if let Ok(guard) = TEST_VECTOR_RANKING.lock() {
            if let Some(ref injected) = *guard {
                return match injected {
                    Ok(ids) => Ok(Some(ids.clone())),
                    Err(reason) => Err(reason.clone()),
                };
            }
        }
    }

    #[cfg(feature = "semantic")]
    {
        try_real_vector_ranking(graph, query, kw_ids, bfs_ids).map(Some)
    }

    #[cfg(not(feature = "semantic"))]
    {
        let _ = (graph, query, kw_ids, bfs_ids);
        Ok(None)
    }
}

#[cfg(feature = "semantic")]
fn try_real_vector_ranking(
    graph: &dyn KnowledgeGraph,
    query: &str,
    kw_ids: &[String],
    bfs_ids: &[String],
) -> Result<Vec<String>, String> {
    if !semantic::model_is_cached() {
        return Err("model not cached (run dare graph enable)".to_string());
    }
    let opts = semantic::SemanticOptions {
        yes: true,
        max_candidates: MAX_CANDIDATES,
    };
    let handle = semantic::ensure_model(&opts).map_err(|e| e.message().to_string())?;
    let candidates = semantic_candidates(graph, kw_ids, bfs_ids).map_err(|e| e.message().to_string())?;
    semantic::vector_rank(&handle, query, &candidates).map_err(|e| e.message().to_string())
}

#[cfg(test)]
static TEST_VECTOR_RANKING: std::sync::Mutex<Option<Result<Vec<String>, String>>> =
    std::sync::Mutex::new(None);

/// Serializes tests that mutate [`TEST_VECTOR_RANKING`] (parallel rustc harness).
#[cfg(test)]
static TEST_VECTOR_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Test-only hook to inject (or fail) the vector ranking channel.
///
/// `None` = use real path (feature on) or skip (feature off).
#[cfg(test)]
pub fn set_test_vector_ranking(value: Option<Result<Vec<String>, String>>) {
    if let Ok(mut guard) = TEST_VECTOR_RANKING.lock() {
        *guard = value;
    }
}

/// Run `f` with an exclusive vector-ranking inject (cleared afterward).
#[cfg(test)]
fn with_test_vector_ranking<R>(
    value: Option<Result<Vec<String>, String>>,
    f: impl FnOnce() -> R,
) -> R {
    let _serial = TEST_VECTOR_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    set_test_vector_ranking(value);
    let out = f();
    set_test_vector_ranking(None);
    out
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
    fn rrf_three_list_fuse_order_golden() {
        let fused = rrf_fuse(
            &[
                vec!["a".into(), "b".into()],
                vec!["b".into(), "c".into()],
                vec!["c".into(), "a".into()],
            ],
            RRF_K,
        );
        // Each id appears twice → equal score 1/61+1/62; tie-break id ASC.
        let expected = 1.0 / 61.0 + 1.0 / 62.0;
        assert_eq!(
            fused.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
        for (_, s) in &fused {
            assert!((s - expected).abs() < 1e-12);
        }
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
        with_test_vector_ranking(None, || {
            let dir = tempdir().unwrap();
            let root = ProjectRoot::new(dir.path()).unwrap();
            let g = seed_graph(&root);
            // 2-list golden: force no_semantic when a real model might activate 3-list.
            #[cfg(feature = "semantic")]
            let force_no_semantic = semantic::model_is_cached();
            #[cfg(not(feature = "semantic"))]
            let force_no_semantic = false;
            let opts = SearchOptions {
                no_semantic: force_no_semantic,
                ..SearchOptions::default()
            };
            let hits = hybrid_query(&g, "alpha", &opts).unwrap();
            assert!(!hits.is_empty());
            let ids: Vec<String> = hits.iter().map(|h| h.id.clone()).collect();
            let golden = golden_041_ids();
            assert_eq!(ids, golden);
            let hits2 = hybrid_query(&g, "alpha", &opts).unwrap();
            assert_eq!(
                hits2.iter().map(|h| h.id.clone()).collect::<Vec<_>>(),
                golden
            );
        });
    }

    #[test]
    fn hybrid_no_semantic_matches_041_golden() {
        with_test_vector_ranking(None, || {
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
        });
    }

    #[test]
    fn hybrid_query_rejects_empty() {
        with_test_vector_ranking(None, || {
            let dir = tempdir().unwrap();
            let root = ProjectRoot::new(dir.path()).unwrap();
            let g = seed_graph(&root);
            let err = hybrid_query(&g, "   ", &SearchOptions::default()).unwrap_err();
            assert!(
                matches!(err, CoreError::InvalidInput(ref m) if m.contains("query must not be empty"))
            );
        });
    }

    #[test]
    fn hybrid_vector_hook_injects_third_list() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let g = seed_graph(&root);
        let c = canonical_file_node_id("src/other.rs");
        let (hits, warnings) = with_test_vector_ranking(Some(Ok(vec![c.clone()])), || {
            hybrid_query_with_warnings(&g, "alpha", &SearchOptions::default()).unwrap()
        });
        assert!(warnings.is_empty());
        assert_eq!(hits[0].id, c);
    }

    #[test]
    fn hybrid_fallback_warning_on_vector_err() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let g = seed_graph(&root);
        let (hits, warnings) = with_test_vector_ranking(
            Some(Err("model not cached (run dare graph enable)".into())),
            || hybrid_query_with_warnings(&g, "alpha", &SearchOptions::default()).unwrap(),
        );
        assert_eq!(
            hits.iter().map(|h| h.id.clone()).collect::<Vec<_>>(),
            golden_041_ids()
        );
        assert_eq!(warnings.len(), 1);
        assert!(
            warnings[0].starts_with(MSG_SEMANTIC_UNAVAILABLE),
            "got {}",
            warnings[0]
        );
        assert!(warnings[0].contains("model not cached"));
        assert!(!warnings[0].contains("token="));
    }

    #[test]
    fn hybrid_no_semantic_ignores_injected_vector() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let g = seed_graph(&root);
        let c = canonical_file_node_id("src/other.rs");
        let opts = SearchOptions {
            no_semantic: true,
            ..SearchOptions::default()
        };
        let (hits, warnings) = with_test_vector_ranking(Some(Ok(vec![c])), || {
            hybrid_query_with_warnings(&g, "alpha", &opts).unwrap()
        });
        assert!(warnings.is_empty());
        assert_eq!(
            hits.iter().map(|h| h.id.clone()).collect::<Vec<_>>(),
            golden_041_ids()
        );
    }

    #[test]
    fn semantic_candidates_union_sorted_capped() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let g = seed_graph(&root);
        let a = canonical_file_node_id("src/alpha.rs");
        let b = "code_symbol:src/alpha.rs::helper".to_string();
        let c = canonical_file_node_id("src/other.rs");
        let cands = semantic_candidates(&g, &[c.clone(), a.clone()], &[b.clone(), a.clone()])
            .unwrap();
        let ids: Vec<_> = cands.iter().map(|(id, _)| id.clone()).collect();
        let mut expected = vec![a, b, c];
        expected.sort();
        assert_eq!(ids, expected);
        for (id, passage) in &cands {
            assert!(!passage.is_empty(), "empty passage for {id}");
            assert!(passage.chars().count() <= crate::semantic::MAX_PASSAGE_CHARS);
        }
    }

    #[cfg(feature = "semantic")]
    #[test]
    fn hybrid_fallback_when_model_not_cached() {
        with_test_vector_ranking(None, || {
            if semantic::model_is_cached() {
                return;
            }
            let dir = tempdir().unwrap();
            let root = ProjectRoot::new(dir.path()).unwrap();
            let g = seed_graph(&root);
            let (hits, warnings) =
                hybrid_query_with_warnings(&g, "alpha", &SearchOptions::default()).unwrap();
            assert_eq!(
                hits.iter().map(|h| h.id.clone()).collect::<Vec<_>>(),
                golden_041_ids()
            );
            assert_eq!(warnings.len(), 1);
            assert!(warnings[0].starts_with(MSG_SEMANTIC_UNAVAILABLE));
            assert!(warnings[0].contains("model not cached"));
        });
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
