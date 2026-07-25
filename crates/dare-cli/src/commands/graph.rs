//! `dare graph ingest|query|stats|viz|doctor|enable` (microplanos 041–042).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use dare_core::fs::atomic_write;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use dare_graph::{
    hybrid_query_with_warnings, ingest_project, load_graph_config, open_graph, render_mermaid_subset,
    semantic_doctor, IngestOptions, KnowledgeGraph, SearchOptions,
};
#[cfg(feature = "semantic")]
use dare_graph::MSG_DOWNLOAD_CANCELLED;
use serde_json::{json, Value};

use crate::output::OutputRenderer;

#[derive(Debug, Clone)]
pub enum GraphAction {
    Ingest {
        dir: Option<PathBuf>,
    },
    Query {
        dir: Option<PathBuf>,
        query: String,
        limit: usize,
        max_hops: usize,
        fanout: usize,
        no_semantic: bool,
    },
    Stats {
        dir: Option<PathBuf>,
    },
    Viz {
        dir: Option<PathBuf>,
        output: Option<PathBuf>,
        max_nodes: usize,
    },
    Doctor {
        dir: Option<PathBuf>,
    },
    Enable {
        dir: Option<PathBuf>,
        yes: bool,
    },
}

pub fn run_graph(action: GraphAction, renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_graph_inner(action) {
        Ok((msg, data)) => {
            let _ = renderer.write_success(&msg, data);
            ExitCode::SUCCESS
        }
        Err(e) => ExitCode::from(renderer.write_error(&e) as u8),
    }
}

fn run_graph_inner(action: GraphAction) -> CoreResult<(String, Value)> {
    match action {
        GraphAction::Ingest { dir } => {
            let root = resolve_root(dir)?;
            let cfg = load_graph_config(&root, None)?;
            let mut g = open_graph(&root, &cfg)?;
            g.migrate()?;
            let report = ingest_project(&root, &mut g, &IngestOptions::default())?;
            let _ = g.try_rebuild_fts5();
            g.flush()?;
            let human = format!(
                "graph ingest: scanned={} indexed={} skippedUnchanged={} symbols={} warnings={}",
                report.scanned,
                report.indexed,
                report.skipped_unchanged,
                report.symbols,
                report.warnings.len()
            );
            let data = json!({
                "action": "graph.ingest",
                "report": report,
                "backend": format!("{:?}", cfg.backend).to_ascii_lowercase(),
                "path": cfg.path,
            });
            Ok((human, data))
        }
        GraphAction::Query {
            dir,
            query,
            limit,
            max_hops,
            fanout,
            no_semantic,
        } => {
            if query.trim().is_empty() {
                return Err(CoreError::invalid_input("query must not be empty"));
            }
            let root = resolve_root(dir)?;
            let cfg = load_graph_config(&root, None)?;
            let mut g = open_graph(&root, &cfg)?;
            g.migrate()?;
            let opts = SearchOptions {
                limit,
                max_hops,
                fanout,
                no_semantic,
            };
            let (hits, warnings) = hybrid_query_with_warnings(&g, &query, &opts)?;
            let mut human = format_query_human(&query, &hits);
            for w in &warnings {
                human.push_str(&format!("\nwarning: {w}"));
            }
            let data = json!({
                "action": "graph.query",
                "query": query,
                "count": hits.len(),
                "hits": hits,
                "warnings": warnings,
                "noSemantic": no_semantic,
            });
            Ok((human, data))
        }
        GraphAction::Stats { dir } => {
            let root = resolve_root(dir)?;
            let cfg = load_graph_config(&root, None)?;
            let mut g = open_graph(&root, &cfg)?;
            g.migrate()?;
            let stats = g.get_statistics()?;
            let human = format!(
                "graph stats: nodes={} edges={}",
                stats.total_nodes, stats.total_edges
            );
            let data = json!({
                "action": "graph.stats",
                "statistics": stats,
            });
            Ok((human, data))
        }
        GraphAction::Viz {
            dir,
            output,
            max_nodes,
        } => {
            let root = resolve_root(dir)?;
            let cfg = load_graph_config(&root, None)?;
            let mut g = open_graph(&root, &cfg)?;
            g.migrate()?;
            let mermaid = render_mermaid_subset(&g, max_nodes)?;
            if let Some(out) = output {
                let rel = out_to_rel(&root, &out)?;
                atomic_write(&root, &rel, mermaid.as_bytes())?;
                let human = format!("graph viz: wrote {}", rel.as_str());
                let data = json!({
                    "action": "graph.viz",
                    "format": "mermaid",
                    "output": rel.as_str(),
                    "bytes": mermaid.len(),
                });
                Ok((human, data))
            } else {
                let data = json!({
                    "action": "graph.viz",
                    "format": "mermaid",
                    "content": mermaid,
                });
                Ok((mermaid, data))
            }
        }
        GraphAction::Doctor { dir } => {
            let _root = resolve_root(dir)?;
            let report = semantic_doctor();
            let human = format!(
                "semanticCompiled: {}\nmodelPresent: {}\ncacheDir: {}\nembedDim: {}",
                report.semantic_compiled,
                report.model_present,
                report.cache_dir,
                report.embed_dim
            );
            let data = json!({
                "action": "graph.doctor",
                "report": report,
            });
            Ok((human, data))
        }
        GraphAction::Enable { dir, yes } => {
            let _root = resolve_root(dir)?;
            run_enable(yes)
        }
    }
}

fn run_enable(yes: bool) -> CoreResult<(String, Value)> {
    #[cfg(not(feature = "semantic"))]
    {
        let _ = yes;
        Err(CoreError::invalid_input(
            "semantic feature not compiled into this binary",
        ))
    }

    #[cfg(feature = "semantic")]
    {
        use dare_graph::{ensure_model, model_is_cached, SemanticOptions, MAX_CANDIDATES};

        let already = model_is_cached();
        let opts = SemanticOptions {
            yes,
            max_candidates: MAX_CANDIDATES,
        };
        match ensure_model(&opts) {
            Ok(_) => {
                let human = if already {
                    "model already present".to_string()
                } else {
                    "graph enable: model ready".to_string()
                };
                let data = json!({
                    "action": "graph.enable",
                    "modelPresent": true,
                    "alreadyCached": already,
                });
                Ok((human, data))
            }
            Err(e) if e.message() == MSG_DOWNLOAD_CANCELLED => {
                let data = json!({
                    "action": "graph.enable",
                    "cancelled": true,
                });
                Ok((MSG_DOWNLOAD_CANCELLED.to_string(), data))
            }
            Err(e) => Err(e),
        }
    }
}

fn resolve_root(dir: Option<PathBuf>) -> CoreResult<ProjectRoot> {
    let path =
        dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    ProjectRoot::new(&path)
}

fn out_to_rel(root: &ProjectRoot, out: &Path) -> CoreResult<SafeRelativePath> {
    if out.is_absolute() {
        let root_std = root.as_path().as_std_path();
        let root_canon = std::fs::canonicalize(root_std).unwrap_or_else(|_| root_std.to_path_buf());
        let out_canon = if out.exists() {
            std::fs::canonicalize(out).map_err(|e| CoreError::io(e.to_string()))?
        } else if let Some(parent) = out.parent() {
            let parent_canon = if parent.as_os_str().is_empty() {
                root_canon.clone()
            } else if parent.exists() {
                std::fs::canonicalize(parent).map_err(|e| CoreError::io(e.to_string()))?
            } else {
                parent.to_path_buf()
            };
            let name = out
                .file_name()
                .ok_or_else(|| CoreError::invalid_input("invalid output path"))?;
            parent_canon.join(name)
        } else {
            out.to_path_buf()
        };
        let rel = out_canon
            .strip_prefix(&root_canon)
            .or_else(|_| out_canon.strip_prefix(root_std))
            .map_err(|_| CoreError::invalid_input("output path must stay within project root"))?;
        let posix = rel.to_string_lossy().replace('\\', "/");
        SafeRelativePath::new(&posix)
    } else {
        let posix = out.to_string_lossy().replace('\\', "/");
        SafeRelativePath::new(&posix)
    }
}

fn format_query_human(query: &str, hits: &[dare_graph::RankedHit]) -> String {
    let mut lines = vec![format!("graph query: {:?} → {} hit(s)", query, hits.len())];
    for (i, h) in hits.iter().enumerate() {
        lines.push(format!(
            "  {}. {:.6}  {}  [{}] {}",
            i + 1,
            h.score,
            h.id,
            h.node_type,
            h.label
        ));
    }
    lines.join("\n")
}
