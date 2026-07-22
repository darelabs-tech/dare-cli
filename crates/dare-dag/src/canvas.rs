//! Deterministic Markdown canvas for DAG runtime status (microplano 026).

use dare_contracts::{DagDocument, RuntimeStateV1};
use dare_core::fs::atomic_write;
use dare_core::{CoreResult, ProjectRoot, SafeRelativePath};
use std::collections::BTreeMap;

use crate::graph::iter_task_views;
use crate::state::Clock;
use crate::status::TaskStatus;

/// Relative path of the live canvas under the project root.
pub const CANVAS_REL: &str = "DARE/.canvas.md";

fn dag_title(doc: &DagDocument) -> String {
    match doc {
        DagDocument::V21(d) => {
            let t = d.title.trim();
            if t.is_empty() {
                "DARE DAG".to_string()
            } else {
                t.to_string()
            }
        }
        DagDocument::Legacy(_) => "DARE DAG".to_string(),
    }
}

fn status_emoji(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "⏳",
        TaskStatus::Running => "🔄",
        TaskStatus::Done => "✅",
        TaskStatus::Failed => "❌",
        TaskStatus::Skipped => "⏭️",
    }
}

fn format_status_cell(raw: &str) -> String {
    match TaskStatus::parse(raw) {
        Ok(st) => format!("{} {}", status_emoji(st), st.as_str()),
        Err(_) => raw.to_string(),
    }
}

fn format_duration(ms: Option<u64>) -> String {
    match ms {
        Some(n) => format!("{n}ms"),
        None => "-".to_string(),
    }
}

fn format_tokens(tokens: Option<u64>) -> String {
    match tokens {
        Some(n) => n.to_string(),
        None => "-".to_string(),
    }
}

fn progress_bar(done: usize, total: usize) -> String {
    let denom = total.max(1);
    let filled = ((20.0 * done as f64) / denom as f64).round() as usize;
    let filled = filled.min(20);
    let empty = 20usize.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn progress_pct(done: usize, total: usize) -> u32 {
    if total == 0 {
        100
    } else {
        ((100 * done) / total) as u32
    }
}

/// Render `DARE/.canvas.md` body (deterministic given `clock`).
pub fn render(
    doc: &DagDocument,
    state: &RuntimeStateV1,
    ranks: Option<&BTreeMap<String, u32>>,
    clock: &dyn Clock,
) -> String {
    let title = dag_title(doc);
    let updated = clock.now_rfc3339();

    let mut views = iter_task_views(doc);
    match ranks {
        Some(r) => {
            views.sort_by(|a, b| {
                let ra = r.get(&a.id).copied().unwrap_or(u32::MAX);
                let rb = r.get(&b.id).copied().unwrap_or(u32::MAX);
                ra.cmp(&rb).then_with(|| a.id.cmp(&b.id))
            });
        }
        None => {
            views.sort_by(|a, b| a.id.cmp(&b.id));
        }
    }

    let total = views.len();
    let done = views
        .iter()
        .filter(|v| {
            state
                .tasks
                .get(&v.id)
                .map(|t| t.status == TaskStatus::Done.as_str())
                .unwrap_or(false)
        })
        .count();
    let pct = progress_pct(done, total);
    let bar = progress_bar(done, total);

    let mut out = String::new();
    out.push_str(&format!("# DARE DAG Execution — {title}\n\n"));
    out.push_str(&format!("**Updated:** {updated}\n\n"));
    out.push_str("## Tasks\n\n");
    out.push_str("| ID | Title | Status | Duration | Tokens |\n");
    out.push_str("|----|-------|--------|----------|--------|\n");

    for v in &views {
        let (status_cell, duration, tokens) = match state.tasks.get(&v.id) {
            Some(t) => (
                format_status_cell(&t.status),
                format_duration(t.duration),
                format_tokens(t.tokens),
            ),
            None => (
                format_status_cell(TaskStatus::Pending.as_str()),
                "-".to_string(),
                "-".to_string(),
            ),
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            v.id, v.title, status_cell, duration, tokens
        ));
    }

    out.push('\n');
    out.push_str(&format!("## Progress: {done}/{total} tasks ({pct}%)\n\n"));
    out.push_str(&bar);
    out.push('\n');
    out
}

/// Atomically write `CANVAS_REL` with UTF-8 Markdown from [`render`].
pub fn write(
    root: &ProjectRoot,
    doc: &DagDocument,
    state: &RuntimeStateV1,
    ranks: Option<&BTreeMap<String, u32>>,
    clock: &dyn Clock,
) -> CoreResult<()> {
    let rel = SafeRelativePath::new(CANVAS_REL)?;
    let body = render(doc, state, ranks, clock);
    atomic_write(root, &rel, body.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::compute_ranks;
    use crate::state::{ensure_state, transition, FixedClock, RefreshCanvas, Transition};
    use dare_contracts::parse_dag_yaml;
    use dare_contracts::TaskRuntimeState;
    use serde_json::Map;
    use std::fs;
    use std::path::PathBuf;

    fn fixture(name: &str) -> String {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("../../tests/fixtures/dag");
        p.push(name);
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
    }

    fn clock() -> FixedClock {
        FixedClock("2026-07-22T12:00:00Z".into())
    }

    fn root_tmp() -> (tempfile::TempDir, ProjectRoot) {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("dare.config.json"), "{}").unwrap();
        fs::create_dir_all(dir.path().join("DARE")).unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        (dir, root)
    }

    fn task(status: TaskStatus, duration: Option<u64>, tokens: Option<u64>) -> TaskRuntimeState {
        TaskRuntimeState {
            status: status.as_str().to_string(),
            output: String::new(),
            error: String::new(),
            tokens,
            duration,
            attempts: Vec::new(),
            parent_id: None,
            depends_on: Vec::new(),
            extra: Map::new(),
        }
    }

    #[test]
    fn render_snapshot_fixed_clock() {
        let doc = parse_dag_yaml(&fixture("ranks-chain.v21.yaml")).unwrap();
        let ranks = compute_ranks(&doc).unwrap();
        let mut state = RuntimeStateV1 {
            version: 1,
            updated_at: "2026-07-22T12:00:00Z".into(),
            tasks: BTreeMap::new(),
            extra: Map::new(),
        };
        state.tasks.insert(
            "task-alpha".into(),
            task(TaskStatus::Done, Some(100), Some(42)),
        );
        state
            .tasks
            .insert("task-beta".into(), task(TaskStatus::Running, None, None));
        state
            .tasks
            .insert("task-gamma".into(), task(TaskStatus::Pending, None, None));

        let md = render(&doc, &state, Some(&ranks), &clock());
        let expected = "\
# DARE DAG Execution — Ranks chain\n\
\n\
**Updated:** 2026-07-22T12:00:00Z\n\
\n\
## Tasks\n\
\n\
| ID | Title | Status | Duration | Tokens |\n\
|----|-------|--------|----------|--------|\n\
| task-alpha | Alpha root | ✅ DONE | 100ms | 42 |\n\
| task-beta | Beta middle | 🔄 RUNNING | - | - |\n\
| task-gamma | Gamma leaf | ⏳ PENDING | - | - |\n\
\n\
## Progress: 1/3 tasks (33%)\n\
\n\
███████░░░░░░░░░░░░░\n\
";
        assert_eq!(md, expected);

        // Without ranks: still lexico (same ids), but order key is id-only.
        let md_no_ranks = render(&doc, &state, None, &clock());
        assert!(md_no_ranks.contains("| task-alpha |"));
        assert!(md_no_ranks.find("task-alpha").unwrap() < md_no_ranks.find("task-beta").unwrap());
        assert!(md_no_ranks.find("task-beta").unwrap() < md_no_ranks.find("task-gamma").unwrap());

        // SKIPPED does not count as done
        state
            .tasks
            .insert("task-beta".into(), task(TaskStatus::Skipped, None, None));
        let md_skip = render(&doc, &state, Some(&ranks), &clock());
        assert!(md_skip.contains("## Progress: 1/3 tasks (33%)"));
        assert!(md_skip.contains("⏭️ SKIPPED"));
    }

    #[test]
    fn write_creates_file() {
        let (_dir, root) = root_tmp();
        let doc = parse_dag_yaml(&fixture("valid.v21.yaml")).unwrap();
        let mut state = RuntimeStateV1 {
            version: 1,
            updated_at: clock().now_rfc3339(),
            tasks: BTreeMap::new(),
            extra: Map::new(),
        };
        state
            .tasks
            .insert("task-001".into(), task(TaskStatus::Pending, None, None));

        write(&root, &doc, &state, None, &clock()).unwrap();
        let path = root.as_path().as_std_path().join(CANVAS_REL);
        assert!(path.is_file());
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("# DARE DAG Execution — Valid fixture"));
        assert!(body.contains("**Updated:** 2026-07-22T12:00:00Z"));
        assert!(body.contains("| task-001 | One | ⏳ PENDING | - | - |"));
    }

    #[test]
    fn transition_yes_refreshes_canvas() {
        let (_dir, root) = root_tmp();
        let doc = parse_dag_yaml(&fixture("valid.v21.yaml")).unwrap();
        ensure_state(&root, &doc, &clock()).unwrap();

        let canvas_path = root.as_path().as_std_path().join(CANVAS_REL);
        assert!(!canvas_path.exists());

        transition(
            &root,
            &doc,
            "task-001",
            Transition::Start,
            &clock(),
            RefreshCanvas::Yes,
        )
        .unwrap();

        assert!(canvas_path.is_file());
        let body = fs::read_to_string(&canvas_path).unwrap();
        assert!(body.contains("🔄 RUNNING"));
        assert!(body.contains("**Updated:** 2026-07-22T12:00:00Z"));
    }

    #[test]
    fn empty_title_falls_back() {
        let yaml = r#"
title: "   "
version: "1.0.0"
tasks:
  - id: task-001
    title: One
    complexity: LOW
    subtask_prompt: x
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        let state = RuntimeStateV1 {
            version: 1,
            updated_at: String::new(),
            tasks: BTreeMap::new(),
            extra: Map::new(),
        };
        let md = render(&doc, &state, None, &clock());
        assert!(md.starts_with("# DARE DAG Execution — DARE DAG\n"));
    }
}
