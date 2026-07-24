//! Incremental GraphRAG ingest: files by contentHash + symbols via regex.

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

use dare_core::{CoreError, CoreResult, ProjectRoot};
use regex::Regex;
use serde_json::{json, Map};
use sha2::{Digest, Sha256};

use crate::ids::{canonical_code_symbol_node_id, canonical_edge_id, canonical_file_node_id};
use crate::knowledge_graph::KnowledgeGraph;
use crate::types::{EdgeType, GraphEdge, GraphNode, NodeType};

pub const DEFAULT_MAX_FILES: usize = 4_096;
pub const DEFAULT_MAX_FILE_BYTES: usize = 1_048_576;

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "vendor",
    ".dare",
    "DARE",
    "dist",
    "build",
    ".next",
    "coverage",
    ".cursor",
    ".claude",
];

const SOURCE_EXTS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "py", "go", "php", "rb", "java", "kt", "cs", "vue", "svelte",
    "c", "h", "cpp", "hpp", "md",
];

/// Options for project ingest.
#[derive(Debug, Clone)]
pub struct IngestOptions {
    pub max_files: usize,
    pub max_file_bytes: usize,
}

impl Default for IngestOptions {
    fn default() -> Self {
        Self {
            max_files: DEFAULT_MAX_FILES,
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
        }
    }
}

/// Summary of an ingest run.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestReport {
    pub scanned: u64,
    pub indexed: u64,
    pub skipped_unchanged: u64,
    pub symbols: u64,
    pub warnings: Vec<String>,
}

/// SHA-256 hex digest of bytes (contentHash).
pub fn content_hash_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Extract symbol names from source text via multi-language regex heuristics.
pub fn extract_symbols(source: &str) -> Vec<String> {
    // Lazy-ish: compile once per call is fine for ingest caps; keep deterministic order.
    let patterns: &[&str] = &[
        r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)",
        r"(?m)^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_][A-Za-z0-9_]*)",
        r"(?m)^\s*(?:export\s+)?class\s+([A-Za-z_][A-Za-z0-9_]*)",
        r"(?m)^\s*(?:export\s+)?(?:interface|type|enum)\s+([A-Za-z_][A-Za-z0-9_]*)",
        r"(?m)^\s*(?:pub\s+)?(?:struct|enum|trait|mod)\s+([A-Za-z_][A-Za-z0-9_]*)",
        r"(?m)^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)",
        r"(?m)^\s*(?:public|private|protected)?\s*(?:static\s+)?(?:final\s+)?class\s+([A-Za-z_][A-Za-z0-9_]*)",
        r"(?m)^\s*func\s+([A-Za-z_][A-Za-z0-9_]*)",
    ];
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for pat in patterns {
        let Ok(re) = Regex::new(pat) else {
            continue;
        };
        for cap in re.captures_iter(source) {
            if let Some(m) = cap.get(1) {
                let name = m.as_str().to_string();
                if seen.insert(name.clone()) {
                    out.push(name);
                }
            }
        }
    }
    out
}

/// Walk project sources and upsert file + code_symbol nodes.
pub fn ingest_project(
    root: &ProjectRoot,
    graph: &mut dyn KnowledgeGraph,
    opts: &IngestOptions,
) -> CoreResult<IngestReport> {
    let mut report = IngestReport {
        scanned: 0,
        indexed: 0,
        skipped_unchanged: 0,
        symbols: 0,
        warnings: Vec::new(),
    };

    let files = collect_source_files(root.as_path().as_std_path(), opts)?;
    for rel in files {
        report.scanned += 1;
        match ingest_one_file(root, graph, &rel, opts) {
            Ok(FileIngestOutcome::Indexed { symbols }) => {
                report.indexed += 1;
                report.symbols += symbols;
            }
            Ok(FileIngestOutcome::SkippedUnchanged) => {
                report.skipped_unchanged += 1;
            }
            Err(e) => {
                report.warnings.push(format!("skip {rel}: {}", e.message()));
            }
        }
    }
    graph.flush()?;
    Ok(report)
}

enum FileIngestOutcome {
    Indexed { symbols: u64 },
    SkippedUnchanged,
}

fn ingest_one_file(
    root: &ProjectRoot,
    graph: &mut dyn KnowledgeGraph,
    rel_posix: &str,
    opts: &IngestOptions,
) -> CoreResult<FileIngestOutcome> {
    let mut abs = root.as_path().as_std_path().to_path_buf();
    for part in rel_posix.split('/') {
        if !part.is_empty() {
            abs.push(part);
        }
    }
    let meta = fs::metadata(&abs).map_err(|e| CoreError::io(e.to_string()))?;
    if meta.len() as usize > opts.max_file_bytes {
        return Err(CoreError::invalid_input(format!(
            "file exceeds max_file_bytes: {rel_posix}"
        )));
    }
    let bytes = fs::read(&abs).map_err(|e| CoreError::io(e.to_string()))?;
    let hash = content_hash_hex(&bytes);
    let file_id = canonical_file_node_id(rel_posix);

    if let Some(existing) = graph.get_node(&file_id)? {
        if existing
            .metadata
            .get("contentHash")
            .and_then(|v| v.as_str())
            == Some(hash.as_str())
        {
            return Ok(FileIngestOutcome::SkippedUnchanged);
        }
    }

    let text = String::from_utf8_lossy(&bytes);
    let symbols = extract_symbols(&text);

    let mut node = GraphNode::new(file_id.clone(), NodeType::File, rel_posix.to_string());
    node.description = Some(format!("source file ({})", file_ext(rel_posix)));
    let mut meta_map = Map::new();
    meta_map.insert("contentHash".into(), json!(hash));
    meta_map.insert("path".into(), json!(rel_posix));
    meta_map.insert("bytes".into(), json!(bytes.len()));
    node.metadata = meta_map;
    graph.add_node(node)?;

    let mut symbol_count = 0u64;
    for sym in &symbols {
        let sid = canonical_code_symbol_node_id(rel_posix, sym);
        let mut snode = GraphNode::new(sid.clone(), NodeType::CodeSymbol, sym.clone());
        snode.description = Some(format!("{rel_posix}::{sym}"));
        let mut sm = Map::new();
        sm.insert("path".into(), json!(rel_posix));
        sm.insert("symbol".into(), json!(sym));
        snode.metadata = sm;
        graph.add_node(snode)?;

        let eid = canonical_edge_id(EdgeType::Contains.as_str(), &file_id, &sid);
        graph.add_edge(GraphEdge::new(
            eid,
            file_id.clone(),
            sid,
            EdgeType::Contains,
        ))?;
        symbol_count += 1;
    }

    Ok(FileIngestOutcome::Indexed {
        symbols: symbol_count,
    })
}

fn file_ext(path: &str) -> &str {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
}

fn collect_source_files(root: &Path, opts: &IngestOptions) -> CoreResult<Vec<String>> {
    let mut out = Vec::new();
    let mut q: VecDeque<PathBuf> = VecDeque::new();
    q.push_back(root.to_path_buf());
    let mut entries = 0usize;

    while let Some(dir) = q.pop_front() {
        let read = match fs::read_dir(&dir) {
            Ok(r) => r,
            Err(e) => return Err(CoreError::io(e.to_string())),
        };
        let mut children: Vec<PathBuf> = read.filter_map(|e| e.ok().map(|e| e.path())).collect();
        children.sort();

        for path in children {
            entries += 1;
            if entries > opts.max_files.saturating_mul(4) {
                break;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if path.is_dir() {
                if SKIP_DIRS.contains(&name.as_str()) {
                    continue;
                }
                q.push_back(path);
                continue;
            }
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !SOURCE_EXTS.contains(&ext.as_str()) {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .map_err(|_| CoreError::invalid_input("path escape during ingest walk"))?;
            let posix = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            out.push(posix);
            if out.len() >= opts.max_files {
                out.sort();
                return Ok(out);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Ensure SQLite FTS5 shadow table exists (best-effort; ignored on JSON / if FTS unavailable).
pub fn ensure_fts5_table(conn: &rusqlite::Connection) -> CoreResult<()> {
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
            id UNINDEXED,
            label,
            description
        );",
    )
    .map_err(|e| CoreError::io(format!("fts5 init: {e}")))?;
    Ok(())
}

/// Rebuild FTS5 from nodes (SQLite only). Soft-fail callers should ignore errors.
pub fn rebuild_fts5(conn: &rusqlite::Connection) -> CoreResult<()> {
    ensure_fts5_table(conn)?;
    conn.execute_batch(
        "DELETE FROM nodes_fts;
         INSERT INTO nodes_fts(id, label, description)
         SELECT id, label, COALESCE(description, '') FROM nodes;",
    )
    .map_err(|e| CoreError::io(format!("fts5 rebuild: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{load_graph_config, open_graph, KnowledgeGraph};
    use tempfile::tempdir;

    #[test]
    fn content_hash_stable() {
        assert_eq!(
            content_hash_hex(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn extract_rust_and_ts_symbols() {
        let src = "pub fn alpha() {}\nfn beta() {}\nexport function gamma() {}\nclass Delta {}\n";
        let syms = extract_symbols(src);
        assert!(syms.contains(&"alpha".to_string()));
        assert!(syms.contains(&"beta".to_string()));
        assert!(syms.contains(&"gamma".to_string()));
        assert!(syms.contains(&"Delta".to_string()));
    }

    #[test]
    fn ingest_incremental_skips_unchanged() {
        let dir = tempdir().unwrap();
        let root_path = dir.path();
        fs::create_dir_all(root_path.join("src")).unwrap();
        fs::write(
            root_path.join("src/lib.rs"),
            "pub fn greet() { println!(\"hi\"); }\n",
        )
        .unwrap();
        let root = ProjectRoot::new(root_path).unwrap();
        let cfg = load_graph_config(&root, None).unwrap();
        let mut g = open_graph(&root, &cfg).unwrap();
        g.migrate().unwrap();

        let r1 = ingest_project(&root, &mut g, &IngestOptions::default()).unwrap();
        assert!(r1.indexed >= 1);
        assert!(r1.symbols >= 1);
        assert_eq!(r1.skipped_unchanged, 0);

        let r2 = ingest_project(&root, &mut g, &IngestOptions::default()).unwrap();
        assert_eq!(r2.indexed, 0);
        assert!(r2.skipped_unchanged >= 1);

        // mutate → reindex
        fs::write(
            root_path.join("src/lib.rs"),
            "pub fn greet() { println!(\"hello\"); }\npub fn other() {}\n",
        )
        .unwrap();
        let r3 = ingest_project(&root, &mut g, &IngestOptions::default()).unwrap();
        assert!(r3.indexed >= 1);
    }
}
