//! Project snapshot, blueprint read, and context query.

use dare_core::fs::read_to_string;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use dare_graph::{load_graph_config, locate, open_graph, LocateOptions, LOCATE_DECAY};
use serde::Serialize;
use serde_json::Value;

use crate::http_map::{
    BLUEPRINT_MAX_BYTES, BLUEPRINT_REL, DAG_REL, MSG_GRAPH_DISABLED, MSG_INVALID_CONTEXT_TYPE,
};
use crate::services::ServiceCtx;
use crate::tasks_md::TASKS_REL;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshot {
    pub schema_version: u32,
    pub root: String,
    pub dare_dir_present: bool,
    pub config_present: bool,
    pub backend: Option<String>,
    pub graph_present: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BlueprintDoc {
    pub path: String,
    pub content: String,
    pub bytes: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextQueryResponse {
    pub schema_version: u32,
    #[serde(rename = "type")]
    pub kind: String,
    pub query: String,
    pub hits: Vec<ContextHit>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextHit {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub snippet: String,
}

fn read_backend(root: &ProjectRoot) -> Option<String> {
    let rel = SafeRelativePath::new("dare.config.json").ok()?;
    let abs = root.resolve(&rel).ok()?;
    let path = abs.as_path().as_std_path();
    if !path.is_file() {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let v: Value = serde_json::from_slice(&bytes).ok()?;
    v.get("ide")
        .or_else(|| v.get("backend"))
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
}

pub fn project_snapshot(ctx: &ServiceCtx) -> CoreResult<ProjectSnapshot> {
    let root_path = ctx.root.as_path().as_std_path();
    let dare_dir_present = root_path.join("DARE").is_dir();
    let config_present = root_path.join("dare.config.json").is_file();
    let mut graph_present = root_path.join("dare-graph.yml").is_file();
    if !graph_present {
        graph_present = root_path.join("DARE").join("dare-graph.yml").is_file();
    }
    let backend = read_backend(&ctx.root);
    Ok(ProjectSnapshot {
        schema_version: 1,
        root: ctx.root.as_path().as_str().to_string(),
        dare_dir_present,
        config_present,
        backend,
        graph_present,
    })
}

pub fn read_blueprint(ctx: &ServiceCtx) -> CoreResult<BlueprintDoc> {
    let rel = SafeRelativePath::new(BLUEPRINT_REL)?;
    let abs = ctx.root.resolve(&rel)?;
    let path = abs.as_path().as_std_path();
    if !path.is_file() {
        return Err(CoreError::not_found(format!(
            "file not found: {BLUEPRINT_REL}"
        )));
    }
    let meta = std::fs::metadata(path).map_err(|e| CoreError::io(e.to_string()))?;
    if meta.len() > BLUEPRINT_MAX_BYTES {
        return Err(CoreError::invalid_input("blueprint exceeds size limit"));
    }
    let content =
        std::fs::read_to_string(path).map_err(|e| CoreError::io(e.to_string()))?;
    let bytes = content.len();
    Ok(BlueprintDoc {
        path: BLUEPRINT_REL.to_string(),
        content,
        bytes,
    })
}

pub fn context_query(
    ctx: &ServiceCtx,
    kind: &str,
    query: &str,
) -> CoreResult<ContextQueryResponse> {
    let kind = kind.trim();
    if !matches!(kind, "architecture" | "task" | "dependency") {
        return Err(CoreError::invalid_input(MSG_INVALID_CONTEXT_TYPE));
    }
    let query = query.trim().to_string();
    if query.is_empty() || query.chars().count() > 512 {
        return Err(CoreError::invalid_input(
            "query must be 1..=512 characters after trim",
        ));
    }

    let root = &ctx.root;
    let mut warnings = Vec::new();
    let hits = match kind {
        "architecture" => search_blueprint(root, &query)?,
        "task" => search_tasks(root, &query)?,
        "dependency" => search_dependency(root, &query, &mut warnings)?,
        _ => unreachable!(),
    };

    Ok(ContextQueryResponse {
        schema_version: 1,
        kind: kind.to_string(),
        query,
        hits,
        warnings,
    })
}

fn clip_snippet(s: &str) -> String {
    let t = s.trim();
    if t.chars().count() <= 280 {
        t.to_string()
    } else {
        t.chars().take(280).collect()
    }
}

fn search_blueprint(root: &ProjectRoot, query: &str) -> CoreResult<Vec<ContextHit>> {
    let rel = SafeRelativePath::new(BLUEPRINT_REL)?;
    let text = match read_to_string(root, &rel) {
        Ok(t) => t,
        Err(CoreError::NotFound(_)) => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let q = query.to_ascii_lowercase();
    let mut hits = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.to_ascii_lowercase().contains(&q) {
            hits.push(ContextHit {
                id: format!("blueprint#{}", i + 1),
                label: line.trim().chars().take(80).collect(),
                kind: "architecture".to_string(),
                snippet: clip_snippet(line),
            });
        }
    }
    Ok(hits)
}

fn search_tasks(root: &ProjectRoot, query: &str) -> CoreResult<Vec<ContextHit>> {
    let rel = SafeRelativePath::new(TASKS_REL)?;
    let text = match read_to_string(root, &rel) {
        Ok(t) => t,
        Err(CoreError::NotFound(_)) => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let q = query.to_ascii_lowercase();
    let mut hits = Vec::new();
    for line in text.lines() {
        if line.to_ascii_lowercase().contains(&q) {
            let id = line
                .split('|')
                .nth(1)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("task")
                .to_string();
            hits.push(ContextHit {
                id: id.clone(),
                label: id,
                kind: "task".to_string(),
                snippet: clip_snippet(line),
            });
        }
    }
    Ok(hits)
}

fn search_dependency(
    root: &ProjectRoot,
    query: &str,
    warnings: &mut Vec<String>,
) -> CoreResult<Vec<ContextHit>> {
    match try_graph_locate(root, query) {
        Ok(hits) => Ok(hits),
        Err(_) => {
            warnings.push(MSG_GRAPH_DISABLED.to_string());
            search_dag_depends(root, query)
        }
    }
}

fn try_graph_locate(root: &ProjectRoot, query: &str) -> CoreResult<Vec<ContextHit>> {
    let cfg = load_graph_config(root, None)
        .map_err(|_| CoreError::internal(MSG_GRAPH_DISABLED))?;
    let g = open_graph(root, &cfg).map_err(|_| CoreError::internal(MSG_GRAPH_DISABLED))?;
    let opts = LocateOptions {
        query: query.to_string(),
        decay: LOCATE_DECAY,
        ..LocateOptions::default()
    };
    let ranked = locate(&g, &opts)?;
    Ok(ranked
        .into_iter()
        .map(|h| ContextHit {
            id: h.id,
            label: h.label,
            kind: h.node_type,
            snippet: String::new(),
        })
        .collect())
}

fn search_dag_depends(root: &ProjectRoot, query: &str) -> CoreResult<Vec<ContextHit>> {
    let rel = SafeRelativePath::new(DAG_REL)?;
    let text = match read_to_string(root, &rel) {
        Ok(t) => t,
        Err(CoreError::NotFound(_)) => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let q = query.to_ascii_lowercase();
    let mut hits = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let lower = line.to_ascii_lowercase();
        if lower.contains(&q) {
            hits.push(ContextHit {
                id: format!("dag#line{}", i + 1),
                label: clip_snippet(line),
                kind: "dependency".to_string(),
                snippet: clip_snippet(line),
            });
        }
    }
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ProjectRoot;

    #[test]
    fn snapshot_flags() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("DARE")).unwrap();
        std::fs::write(
            dir.path().join("dare.config.json"),
            r#"{"ide":"rust-axum"}"#,
        )
        .unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let ctx = ServiceCtx::new(root);
        let snap = project_snapshot(&ctx).unwrap();
        assert!(snap.dare_dir_present);
        assert!(snap.config_present);
        assert_eq!(snap.backend.as_deref(), Some("rust-axum"));
        assert!(!snap.graph_present);
        assert_eq!(snap.schema_version, 1);
    }

    #[test]
    fn blueprint_size_limit() {
        let dir = tempfile::tempdir().unwrap();
        let dare = dir.path().join("DARE");
        std::fs::create_dir_all(&dare).unwrap();
        // Create a sparse-ish oversize file via truncate if possible; else write chunk.
        let path = dare.join("BLUEPRINT.md");
        let f = std::fs::File::create(&path).unwrap();
        f.set_len(BLUEPRINT_MAX_BYTES + 1).unwrap();
        drop(f);
        let root = ProjectRoot::new(dir.path()).unwrap();
        let ctx = ServiceCtx::new(root);
        let err = read_blueprint(&ctx).unwrap_err();
        assert_eq!(err.message(), "blueprint exceeds size limit");
    }

    #[test]
    fn context_rejects_bad_type() {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let ctx = ServiceCtx::new(root);
        let err = context_query(&ctx, "foo", "x").unwrap_err();
        assert_eq!(err.message(), MSG_INVALID_CONTEXT_TYPE);
    }

    #[test]
    fn context_query_trim_bounds() {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let ctx = ServiceCtx::new(root);
        assert!(context_query(&ctx, "task", "   ").is_err());
        let long: String = "a".repeat(513);
        assert!(context_query(&ctx, "task", &long).is_err());
    }
}
