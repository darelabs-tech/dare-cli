//! `dare info` — diagnóstico read-only (microplano 017).

use std::path::{Path, PathBuf};

use dare_assets::verify_embedded_assets;
use dare_core::{CoreResult, ProjectRoot, SafeRelativePath};
use serde::Serialize;
use serde_json::{json, Value};

pub const INFO_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoReport {
    pub schema_version: u32,
    pub version: String,
    pub platform: PlatformInfo,
    pub project_root: Option<String>,
    pub assets_ok: bool,
    pub assets_error: Option<String>,
    pub config_present: bool,
    pub graph_path: Option<String>,
    pub graph_present: bool,
    pub backend: Option<String>,
    pub tasks: TasksProgress,
    pub dare_dir_present: bool,
    pub state_present: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub family: String,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TasksProgress {
    pub source: Option<String>,
    pub done: u32,
    pub pending: u32,
    pub total_marked: u32,
}

/// Walk upward looking for project markers (read-only).
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    loop {
        if cur.join("dare.config.json").is_file()
            || cur.join("DARE").is_dir()
            || cur.join("Cargo.toml").is_file()
        {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

fn platform() -> PlatformInfo {
    PlatformInfo {
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        family: std::env::consts::FAMILY.into(),
    }
}

fn read_backend(root: &ProjectRoot) -> Option<String> {
    let rel = SafeRelativePath::new("dare.config.json").ok()?;
    let abs = root.resolve(&rel).ok()?;
    let path = abs.as_path();
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

fn tasks_progress(root: &Path) -> TasksProgress {
    // Prefer DARE/TASKS.md; else lexicographically first DARE/TASKS-*.md
    let path = {
        let tasks_md = root.join("DARE/TASKS.md");
        if tasks_md.is_file() {
            Some(tasks_md)
        } else {
            let dare = root.join("DARE");
            let mut names: Vec<String> = Vec::new();
            if let Ok(rd) = std::fs::read_dir(&dare) {
                for e in rd.flatten() {
                    let name = e.file_name().to_string_lossy().into_owned();
                    if name.starts_with("TASKS-") && name.ends_with(".md") {
                        names.push(name);
                    }
                }
            }
            names.sort();
            names.first().map(|n| dare.join(n))
        }
    };
    let Some(p) = path else {
        return TasksProgress::default();
    };
    let text = std::fs::read_to_string(&p).unwrap_or_default();
    let done = text.matches("✅").count() as u32 + text.matches("DONE").count() as u32;
    let pending = text.matches("⏳").count() as u32 + text.matches("PENDING").count() as u32;
    TasksProgress {
        source: Some(p.display().to_string()),
        done,
        pending,
        total_marked: done + pending,
    }
}

pub fn collect_info(cwd: &Path) -> CoreResult<InfoReport> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let project_root = find_project_root(cwd);
    let assets = verify_embedded_assets();
    let (assets_ok, assets_error) = match assets {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e.to_string())),
    };

    let mut config_present = false;
    let mut graph_path = None;
    let mut graph_present = false;
    let mut backend = None;
    let mut dare_dir_present = false;
    let mut state_present = false;
    let mut tasks = TasksProgress::default();

    if let Some(ref root_path) = project_root {
        dare_dir_present = root_path.join("DARE").is_dir();
        config_present = root_path.join("dare.config.json").is_file();
        state_present = root_path.join(".dare/state.json").is_file();
        let gp = root_path.join("dare-graph.yml");
        graph_present = gp.is_file();
        if graph_present {
            graph_path = Some(gp.display().to_string());
        } else {
            let alt = root_path.join("DARE/dare-graph.yml");
            if alt.is_file() {
                graph_present = true;
                graph_path = Some(alt.display().to_string());
            }
        }
        if let Ok(pr) = ProjectRoot::new(root_path) {
            backend = read_backend(&pr);
        }
        tasks = tasks_progress(root_path);
    }

    Ok(InfoReport {
        schema_version: INFO_SCHEMA_VERSION,
        version,
        platform: platform(),
        project_root: project_root.map(|p| p.display().to_string()),
        assets_ok,
        assets_error,
        config_present,
        graph_path,
        graph_present,
        backend,
        tasks,
        dare_dir_present,
        state_present,
    })
}

pub fn format_human(r: &InfoReport) -> String {
    let mut lines = vec![
        format!("DARE info (schema {})", r.schema_version),
        format!("  version:    {}", r.version),
        format!(
            "  platform:   {}-{} ({})",
            r.platform.os, r.platform.arch, r.platform.family
        ),
        format!(
            "  project:    {}",
            r.project_root.as_deref().unwrap_or("(not detected)")
        ),
        format!(
            "  assets:     {}",
            if r.assets_ok {
                "ok".into()
            } else {
                format!("FAIL ({})", r.assets_error.as_deref().unwrap_or("?"))
            }
        ),
        format!(
            "  config:     {}",
            if r.config_present { "yes" } else { "no" }
        ),
        format!(
            "  DARE/:      {}",
            if r.dare_dir_present { "yes" } else { "no" }
        ),
        format!(
            "  .dare/state:{}",
            if r.state_present { " yes" } else { " no" }
        ),
        format!(
            "  graph:      {}",
            r.graph_path.as_deref().unwrap_or(if r.graph_present {
                "(present)"
            } else {
                "(absent)"
            })
        ),
        format!(
            "  backend/ide:{}",
            r.backend
                .as_ref()
                .map(|b| format!(" {b}"))
                .unwrap_or_else(|| " (none)".into())
        ),
        format!(
            "  tasks:      done={} pending={} ({})",
            r.tasks.done,
            r.tasks.pending,
            r.tasks.source.as_deref().unwrap_or("no TASKS.md")
        ),
        "  mode:       read-only (zero mutations)".into(),
    ];
    lines.push(String::new());
    lines.join("\n")
}

pub fn report_to_json(r: &InfoReport) -> Value {
    json!(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn find_root_walks_up() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("a/b");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(dir.path().join("dare.config.json"), b"{}").unwrap();
        let found = find_project_root(&nested).unwrap();
        assert_eq!(found, dir.path());
    }

    #[test]
    fn collect_is_read_only_and_schema_stable() {
        let dir = tempdir().unwrap();
        let before: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        let r = collect_info(dir.path()).unwrap();
        assert_eq!(r.schema_version, 1);
        assert!(r.version.contains('.'));
        let after: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(before, after, "info must not create files");
        let v = report_to_json(&r);
        assert_eq!(v["schemaVersion"], 1);
    }

    #[test]
    fn tasks_picks_lexicographic_tasks_star() {
        let dir = tempdir().unwrap();
        let dare = dir.path().join("DARE");
        std::fs::create_dir_all(&dare).unwrap();
        std::fs::write(dare.join("TASKS-b.md"), "✅ DONE\n").unwrap();
        std::fs::write(dare.join("TASKS-a.md"), "⏳ PENDING\n").unwrap();
        let r = collect_info(dir.path()).unwrap();
        let src = r.tasks.source.expect("source");
        assert!(
            src.ends_with("TASKS-a.md"),
            "expected lexicographic first TASKS-a.md, got {src}"
        );
        assert_eq!(r.tasks.pending, 2); // ⏳ + PENDING
        assert_eq!(r.tasks.done, 0);
    }
}
