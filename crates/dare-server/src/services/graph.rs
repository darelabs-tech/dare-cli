//! Graph locate / traverse / map-requirement.

use dare_core::{CoreError, CoreResult};
use dare_graph::{
    bfs_expand, load_graph_config, locate, open_graph, GraphHandle, LocateOptions, NodeType,
    RankedHit, DEFAULT_FANOUT, DEFAULT_MAX_HOPS,
};

use crate::http_map::MSG_GRAPH_DISABLED;
use crate::services::ServiceCtx;

fn open_project_graph(root: &dare_core::ProjectRoot) -> CoreResult<GraphHandle> {
    let cfg = load_graph_config(root, None)
        .map_err(|_| CoreError::internal(MSG_GRAPH_DISABLED))?;
    open_graph(root, &cfg).map_err(|_| CoreError::internal(MSG_GRAPH_DISABLED))
}

fn validate_locate_query(opts: &LocateOptions) -> CoreResult<()> {
    if opts.query.trim().is_empty() {
        return Err(CoreError::invalid_input("query must not be empty"));
    }
    Ok(())
}

pub fn graph_locate(ctx: &ServiceCtx, opts: LocateOptions) -> CoreResult<Vec<RankedHit>> {
    validate_locate_query(&opts)?;
    let g = open_project_graph(&ctx.root)?;
    locate(&g, &opts)
}

pub fn graph_traverse(
    ctx: &ServiceCtx,
    seeds: &[String],
    max_hops: usize,
    fanout: usize,
) -> CoreResult<Vec<String>> {
    if seeds.is_empty() || seeds.len() > 32 {
        return Err(CoreError::invalid_input(
            "seeds must contain 1..=32 entries",
        ));
    }
    for s in seeds {
        let t = s.trim();
        if t.is_empty() || t.len() > 256 {
            return Err(CoreError::invalid_input(
                "each seed must be non-empty and <= 256 chars",
            ));
        }
    }
    let g = open_project_graph(&ctx.root)?;
    bfs_expand(&g, seeds, max_hops, fanout)
}

pub fn graph_map_requirement(
    ctx: &ServiceCtx,
    opts: LocateOptions,
) -> CoreResult<Vec<RankedHit>> {
    validate_locate_query(&opts)?;
    let g = open_project_graph(&ctx.root)?;
    let all = locate(&g, &opts)?;
    let req = NodeType::Requirement.as_str();
    let filtered: Vec<RankedHit> = all
        .iter()
        .filter(|h| h.node_type == req)
        .cloned()
        .collect();
    Ok(if filtered.is_empty() { all } else { filtered })
}

/// Defaults used by REST when optional body fields are omitted.
pub fn locate_defaults(
    query: String,
    max_hops: Option<usize>,
    fanout: Option<usize>,
    limit: Option<usize>,
    decay: Option<f64>,
) -> LocateOptions {
    use dare_graph::{DEFAULT_LIMIT, LOCATE_DECAY};
    LocateOptions {
        query,
        max_hops: max_hops.unwrap_or(DEFAULT_MAX_HOPS),
        fanout: fanout.unwrap_or(DEFAULT_FANOUT),
        limit: limit.unwrap_or(DEFAULT_LIMIT),
        decay: decay.unwrap_or(LOCATE_DECAY),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ProjectRoot;

    #[test]
    fn empty_query_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let ctx = ServiceCtx::new(root);
        let opts = LocateOptions {
            query: "  ".to_string(),
            ..LocateOptions::default()
        };
        let err = graph_locate(&ctx, opts).unwrap_err();
        assert_eq!(err.message(), "query must not be empty");
    }

    #[test]
    fn empty_project_returns_empty_hits() {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let ctx = ServiceCtx::new(root);
        let opts = LocateOptions {
            query: "auth".to_string(),
            ..LocateOptions::default()
        };
        let hits = graph_locate(&ctx, opts).unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn traverse_seed_bounds() {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let ctx = ServiceCtx::new(root);
        let err = graph_traverse(&ctx, &[], DEFAULT_MAX_HOPS, DEFAULT_FANOUT).unwrap_err();
        assert!(err.message().contains("1..=32"));
    }
}
