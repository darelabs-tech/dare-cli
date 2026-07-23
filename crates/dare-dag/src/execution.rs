//! Execute helpers: ready-at-min-rank, prompt compose, status snapshot (microplano 028).

use std::collections::BTreeMap;

use dare_contracts::{DagDocument, RuntimeStateV1};

use crate::graph::{iter_task_views, next_executable, DagGraphError};
use crate::status::TaskStatus;

/// Canonical human message when the DAG has finished.
pub const MSG_RESOLVED: &str = "✅ All tasks resolved.";
/// Canonical human message when PENDING remain but none are executable.
pub const MSG_BLOCKED: &str = "Blocked — no executable tasks";
/// Canonical human message for a DAG with zero tasks.
pub const MSG_EMPTY: &str = "Empty DAG — no tasks.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteOutcome {
    Status,
    NextReady,
    Resolved,
    Blocked,
    Empty,
    Waiting,
}

impl ExecuteOutcome {
    /// JSON / wire form (camelCase values per Blueprint §5.7).
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecuteOutcome::Status => "status",
            ExecuteOutcome::NextReady => "ready",
            ExecuteOutcome::Resolved => "resolved",
            ExecuteOutcome::Blocked => "blocked",
            ExecuteOutcome::Empty => "empty",
            ExecuteOutcome::Waiting => "waiting",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusCounts {
    pub done: u32,
    pub running: u32,
    pub pending: u32,
    pub failed: u32,
    pub skipped: u32,
    pub total: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusTaskRow {
    pub id: String,
    pub title: String,
    pub status: String,
    pub rank: u32,
    pub complexity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSnapshot {
    pub title: String,
    pub dag_rel: String,
    pub canvas_path: String,
    pub counts: StatusCounts,
    pub tasks: Vec<StatusTaskRow>,
    pub outcome: ExecuteOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadyTask {
    pub id: String,
    pub title: String,
    pub rank: u32,
    pub complexity: String,
    pub spec_file: String,
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextReport {
    pub rank: Option<u32>,
    pub ready: Vec<ReadyTask>,
    pub outcome: ExecuteOutcome,
}

/// `parent_context_chars` from V2.1 limits, or **2000** for Legacy.
pub fn parent_context_limit(doc: &DagDocument) -> usize {
    match doc {
        DagDocument::V21(d) => d.limits.parent_context_chars as usize,
        DagDocument::Legacy(_) => 2000,
    }
}

/// `next_executable` filtered to the minimum rank among candidates; ids already lexico within rank.
pub fn ready_at_min_rank(
    doc: &DagDocument,
    state: &RuntimeStateV1,
    ranks: &BTreeMap<String, u32>,
) -> Vec<String> {
    let all = next_executable(doc, state, ranks);
    let Some(min_r) = all
        .iter()
        .map(|id| ranks.get(id).copied().unwrap_or(0))
        .min()
    else {
        return Vec::new();
    };
    all.into_iter()
        .filter(|id| ranks.get(id).copied().unwrap_or(0) == min_r)
        .collect()
}

/// Compose `subtask_prompt` + optional upstream tails from DONE parents.
pub fn compose_task_prompt(
    doc: &DagDocument,
    state: &RuntimeStateV1,
    task_id: &str,
) -> Result<String, DagGraphError> {
    let meta = task_meta(doc, task_id).ok_or_else(|| DagGraphError::InvalidDag {
        message: format!("task not found: {task_id}"),
    })?;
    let limit = parent_context_limit(doc);
    let mut deps = meta.depends_on.clone();
    deps.sort();

    let mut upstream = String::new();
    for dep_id in &deps {
        let Some(rt) = state.tasks.get(dep_id) else {
            continue;
        };
        if rt.status != TaskStatus::Done.as_str() {
            continue;
        }
        let out = rt.output.trim();
        if out.is_empty() {
            continue;
        }
        let title = task_meta(doc, dep_id)
            .map(|m| m.title)
            .unwrap_or_else(|| dep_id.clone());
        let tail = unicode_tail(out, limit);
        upstream.push_str(&format!("### From parent: {dep_id} — {title}\n{tail}\n"));
    }

    if upstream.is_empty() {
        Ok(meta.subtask_prompt)
    } else {
        Ok(format!(
            "{}\n\n## Upstream context\n\n{}",
            meta.subtask_prompt, upstream
        ))
    }
}

/// Build a status snapshot (domain leaves `dag_rel` / `canvas_path` empty for CLI to fill).
pub fn build_status_snapshot(
    doc: &DagDocument,
    state: &RuntimeStateV1,
    ranks: &BTreeMap<String, u32>,
) -> StatusSnapshot {
    let views = iter_task_views(doc);
    let title = dag_title(doc);
    if views.is_empty() {
        return StatusSnapshot {
            title,
            dag_rel: String::new(),
            canvas_path: String::new(),
            counts: StatusCounts {
                done: 0,
                running: 0,
                pending: 0,
                failed: 0,
                skipped: 0,
                total: 0,
            },
            tasks: Vec::new(),
            outcome: ExecuteOutcome::Empty,
        };
    }

    let mut counts = StatusCounts {
        done: 0,
        running: 0,
        pending: 0,
        failed: 0,
        skipped: 0,
        total: views.len() as u32,
    };
    let mut tasks: Vec<StatusTaskRow> = Vec::new();
    for t in &views {
        let status = state
            .tasks
            .get(&t.id)
            .map(|r| r.status.clone())
            .unwrap_or_else(|| TaskStatus::Pending.as_str().to_string());
        bump_count(&mut counts, &status);
        let meta = task_meta(doc, &t.id);
        tasks.push(StatusTaskRow {
            id: t.id.clone(),
            title: t.title.clone(),
            status,
            rank: ranks.get(&t.id).copied().unwrap_or(0),
            complexity: meta
                .map(|m| m.complexity)
                .unwrap_or_else(|| "UNKNOWN".into()),
        });
    }
    tasks.sort_by(|a, b| a.rank.cmp(&b.rank).then_with(|| a.id.cmp(&b.id)));

    StatusSnapshot {
        title,
        dag_rel: String::new(),
        canvas_path: String::new(),
        counts,
        tasks,
        outcome: ExecuteOutcome::Status,
    }
}

/// Classify `--next` outcome from ready set + runtime state.
pub fn classify_next_outcome(
    doc: &DagDocument,
    state: &RuntimeStateV1,
    ready: &[String],
) -> ExecuteOutcome {
    if iter_task_views(doc).is_empty() {
        return ExecuteOutcome::Empty;
    }
    if !ready.is_empty() {
        return ExecuteOutcome::NextReady;
    }
    let mut pending = 0u32;
    let mut running = 0u32;
    for t in iter_task_views(doc) {
        let status = state
            .tasks
            .get(&t.id)
            .map(|r| r.status.as_str())
            .unwrap_or(TaskStatus::Pending.as_str());
        match TaskStatus::parse(status) {
            Ok(TaskStatus::Pending) => pending += 1,
            Ok(TaskStatus::Running) => running += 1,
            _ => {}
        }
    }
    if running > 0 {
        return ExecuteOutcome::Waiting;
    }
    if pending > 0 {
        return ExecuteOutcome::Blocked;
    }
    ExecuteOutcome::Resolved
}

/// Build a full `--next` report (ready tasks with composed prompts).
pub fn build_next_report(
    doc: &DagDocument,
    state: &RuntimeStateV1,
    ranks: &BTreeMap<String, u32>,
) -> Result<NextReport, DagGraphError> {
    let ready_ids = ready_at_min_rank(doc, state, ranks);
    let outcome = classify_next_outcome(doc, state, &ready_ids);
    if ready_ids.is_empty() {
        return Ok(NextReport {
            rank: None,
            ready: Vec::new(),
            outcome,
        });
    }
    let rank = ranks.get(&ready_ids[0]).copied();
    let mut ready = Vec::new();
    for id in ready_ids {
        let meta = task_meta(doc, &id).ok_or_else(|| DagGraphError::InvalidDag {
            message: format!("task not found: {id}"),
        })?;
        let prompt = compose_task_prompt(doc, state, &id)?;
        ready.push(ReadyTask {
            id: id.clone(),
            title: meta.title,
            rank: ranks.get(&id).copied().unwrap_or(0),
            complexity: meta.complexity,
            spec_file: meta.spec_file,
            prompt,
        });
    }
    Ok(NextReport {
        rank,
        ready,
        outcome,
    })
}

fn bump_count(counts: &mut StatusCounts, status: &str) {
    match TaskStatus::parse(status) {
        Ok(TaskStatus::Done) => counts.done += 1,
        Ok(TaskStatus::Running) => counts.running += 1,
        Ok(TaskStatus::Pending) => counts.pending += 1,
        Ok(TaskStatus::Failed) => counts.failed += 1,
        Ok(TaskStatus::Skipped) => counts.skipped += 1,
        Err(_) => counts.pending += 1,
    }
}

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

struct TaskMeta {
    title: String,
    depends_on: Vec<String>,
    complexity: String,
    subtask_prompt: String,
    #[allow(dead_code)]
    spec_file: String,
}

fn task_meta(doc: &DagDocument, id: &str) -> Option<TaskMeta> {
    match doc {
        DagDocument::V21(d) => d.tasks.iter().find(|t| t.id == id).map(|t| TaskMeta {
            title: t.title.clone(),
            depends_on: t.depends_on.clone(),
            complexity: t.complexity.clone(),
            subtask_prompt: t.subtask_prompt.clone(),
            spec_file: t.spec_file.clone(),
        }),
        DagDocument::Legacy(d) => d.tasks.get(id).map(|t| TaskMeta {
            title: t.title.clone(),
            depends_on: t.depends_on.clone(),
            complexity: t.complexity.clone(),
            subtask_prompt: String::new(),
            spec_file: String::new(),
        }),
    }
}

fn unicode_tail(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    s.chars().skip(count - max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::compute_ranks;
    use crate::status::TaskStatus;
    use dare_contracts::{parse_dag_yaml, TaskRuntimeState};
    use serde_json::Map;
    use std::fs;
    use std::path::PathBuf;

    fn fixture(name: &str) -> String {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("../../tests/fixtures/dag");
        p.push(name);
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
    }

    fn task(status: TaskStatus, output: &str, depends_on: &[&str]) -> TaskRuntimeState {
        TaskRuntimeState {
            status: status.as_str().to_string(),
            output: output.to_string(),
            error: String::new(),
            tokens: None,
            duration: None,
            attempts: Vec::new(),
            parent_id: None,
            depends_on: depends_on.iter().map(|s| (*s).to_string()).collect(),
            extra: Map::new(),
        }
    }

    fn state_from(pairs: &[(&str, TaskRuntimeState)]) -> RuntimeStateV1 {
        let mut tasks = BTreeMap::new();
        for (id, t) in pairs {
            tasks.insert((*id).to_string(), t.clone());
        }
        RuntimeStateV1 {
            version: 1,
            updated_at: "2026-01-01T00:00:00Z".into(),
            tasks,
            extra: Map::new(),
        }
    }

    #[test]
    fn ready_at_min_rank_filters() {
        let doc = parse_dag_yaml(&fixture("ranks-diamond.v21.yaml")).unwrap();
        let ranks = compute_ranks(&doc).unwrap();
        // root DONE → left+right PENDING ready at rank 1; join not ready
        let st = state_from(&[
            ("task-root", task(TaskStatus::Done, "ok", &[])),
            ("task-left", task(TaskStatus::Pending, "", &["task-root"])),
            ("task-right", task(TaskStatus::Pending, "", &["task-root"])),
            (
                "task-join",
                task(TaskStatus::Pending, "", &["task-left", "task-right"]),
            ),
        ]);
        let ready = ready_at_min_rank(&doc, &st, &ranks);
        assert_eq!(
            ready,
            vec!["task-left".to_string(), "task-right".to_string()]
        );
        assert!(ready.iter().all(|id| ranks.get(id).copied() == Some(1)));
    }

    #[test]
    fn compose_tail_exact_chars() {
        let yaml = r#"
title: "T"
version: "1.0.0"
limits:
  parent_context_chars: 5
  task_output_chars: 4000
  timeout_seconds: 600
tasks:
  - id: task-a
    title: Alpha
    depends_on: []
    complexity: LOW
    subtask_prompt: do-a
  - id: task-b
    title: Beta
    depends_on: [task-a]
    complexity: MED
    subtask_prompt: do-b
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        assert_eq!(parent_context_limit(&doc), 5);
        let long = "ABCDEFGHIJ"; // 10 chars
        let st = state_from(&[
            ("task-a", task(TaskStatus::Done, long, &[])),
            ("task-b", task(TaskStatus::Pending, "", &["task-a"])),
        ]);
        let prompt = compose_task_prompt(&doc, &st, "task-b").unwrap();
        assert!(prompt.starts_with("do-b"));
        assert!(prompt.contains("## Upstream context"));
        assert!(prompt.contains("### From parent: task-a — Alpha\nFGHIJ\n"));
        assert!(!prompt.contains("ABCDE"));
    }

    #[test]
    fn compose_omits_empty_parent() {
        let yaml = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-a
    title: Alpha
    depends_on: []
    complexity: LOW
    subtask_prompt: a
  - id: task-b
    title: Beta
    depends_on: [task-a]
    complexity: LOW
    subtask_prompt: b-only
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        let st = state_from(&[
            ("task-a", task(TaskStatus::Done, "   ", &[])),
            ("task-b", task(TaskStatus::Pending, "", &["task-a"])),
        ]);
        let prompt = compose_task_prompt(&doc, &st, "task-b").unwrap();
        assert_eq!(prompt, "b-only");
        assert!(!prompt.contains("Upstream"));
    }

    #[test]
    fn classify_empty_blocked_resolved_waiting() {
        let empty = parse_dag_yaml(&fixture("exec-empty.v21.yaml")).unwrap();
        let st_empty = state_from(&[]);
        assert_eq!(
            classify_next_outcome(&empty, &st_empty, &[]),
            ExecuteOutcome::Empty
        );

        let doc = parse_dag_yaml(&fixture("exec-blocked.v21.yaml")).unwrap();
        let blocked = state_from(&[
            ("task-a", task(TaskStatus::Failed, "boom", &[])),
            ("task-b", task(TaskStatus::Pending, "", &["task-a"])),
        ]);
        assert_eq!(
            classify_next_outcome(&doc, &blocked, &[]),
            ExecuteOutcome::Blocked
        );

        let waiting = state_from(&[
            ("task-a", task(TaskStatus::Running, "", &[])),
            ("task-b", task(TaskStatus::Pending, "", &["task-a"])),
        ]);
        assert_eq!(
            classify_next_outcome(&doc, &waiting, &[]),
            ExecuteOutcome::Waiting
        );

        let resolved = state_from(&[
            ("task-a", task(TaskStatus::Done, "ok", &[])),
            ("task-b", task(TaskStatus::Done, "ok", &["task-a"])),
        ]);
        assert_eq!(
            classify_next_outcome(&doc, &resolved, &[]),
            ExecuteOutcome::Resolved
        );

        assert_eq!(
            classify_next_outcome(&doc, &blocked, &["task-b".into()]),
            ExecuteOutcome::NextReady
        );
    }

    #[test]
    fn parent_context_legacy_2000() {
        let yaml = r#"
task-a:
  title: A
  complexity: LOW
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        assert!(matches!(doc, DagDocument::Legacy(_)));
        assert_eq!(parent_context_limit(&doc), 2000);
    }

    #[test]
    fn compose_task_not_found() {
        let doc = parse_dag_yaml(&fixture("exec-blocked.v21.yaml")).unwrap();
        let st = state_from(&[]);
        let err = compose_task_prompt(&doc, &st, "missing").unwrap_err();
        assert!(matches!(err, DagGraphError::InvalidDag { .. }));
        assert!(err.to_string().contains("task not found"));
    }

    #[test]
    fn msg_constants_exact() {
        assert_eq!(MSG_RESOLVED, "✅ All tasks resolved.");
        assert_eq!(MSG_BLOCKED, "Blocked — no executable tasks");
        assert_eq!(MSG_EMPTY, "Empty DAG — no tasks.");
        assert_eq!(ExecuteOutcome::NextReady.as_str(), "ready");
    }

    #[test]
    fn build_status_snapshot_empty() {
        let doc = parse_dag_yaml(&fixture("exec-empty.v21.yaml")).unwrap();
        let ranks = BTreeMap::new();
        let snap = build_status_snapshot(&doc, &state_from(&[]), &ranks);
        assert_eq!(snap.outcome, ExecuteOutcome::Empty);
        assert_eq!(snap.counts.total, 0);
    }
}
