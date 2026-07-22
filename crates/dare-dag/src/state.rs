//! Runtime state store: ensure, transition, cascading skip (microplano 026).

use dare_contracts::{
    load_runtime_state, save_runtime_state, AttemptRecord, DagDocument, RuntimeStateV1,
    TaskRuntimeState,
};
use dare_core::fs::FileLock;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde_json::Map;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::canvas;
use crate::graph::{compute_ranks, iter_task_views};
use crate::status::TaskStatus;

/// Relative path of the runtime state file under the project root.
pub const STATE_REL: &str = ".dare/state.json";

/// Injectable clock for `updated_at` / attempt timestamps (RFC3339).
pub trait Clock {
    fn now_rfc3339(&self) -> String;
}

/// Wall-clock UTC via `SystemTime` (no chrono dependency).
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_rfc3339(&self) -> String {
        utc_rfc3339_now()
    }
}

/// Deterministic clock for tests / goldens.
pub struct FixedClock(pub String);

impl Clock for FixedClock {
    fn now_rfc3339(&self) -> String {
        self.0.clone()
    }
}

/// Allowed status mutations (see BLUEPRINT-026 §5.6).
pub enum Transition {
    Start,
    Complete { output: String },
    Fail { error: String },
    Reset,
    Skip,
}

/// Whether `transition` should refresh `DARE/.canvas.md` after save (T-20).
pub enum RefreshCanvas {
    Yes,
    No,
}

fn state_rel() -> CoreResult<SafeRelativePath> {
    SafeRelativePath::new(STATE_REL)
}

fn empty_runtime_state() -> RuntimeStateV1 {
    RuntimeStateV1 {
        version: 1,
        updated_at: String::new(),
        tasks: BTreeMap::new(),
        extra: Map::new(),
    }
}

fn load_or_empty(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<RuntimeStateV1> {
    match load_runtime_state(root, rel) {
        Ok(state) => Ok(state),
        Err(CoreError::NotFound(_)) => Ok(empty_runtime_state()),
        Err(e) => Err(e),
    }
}

fn pending_task(depends_on: Vec<String>) -> TaskRuntimeState {
    TaskRuntimeState {
        status: TaskStatus::Pending.as_str().to_string(),
        output: String::new(),
        error: String::new(),
        tokens: None,
        duration: None,
        attempts: Vec::new(),
        parent_id: None,
        depends_on,
        extra: Map::new(),
    }
}

/// Insert missing DAG tasks as `PENDING` with YAML `depends_on`. Orphans outside the DAG are kept.
fn merge_dag_tasks(state: &mut RuntimeStateV1, doc: &DagDocument) {
    for view in iter_task_views(doc) {
        state
            .tasks
            .entry(view.id)
            .or_insert_with(|| pending_task(view.depends_on));
    }
}

fn dag_contains(doc: &DagDocument, task_id: &str) -> bool {
    iter_task_views(doc).iter().any(|v| v.id == task_id)
}

fn append_attempt(task: &mut TaskRuntimeState, clock: &dyn Clock, passed: bool) {
    let n = task
        .attempts
        .last()
        .map(|a| a.n.saturating_add(1))
        .unwrap_or(1);
    task.attempts.push(AttemptRecord {
        n,
        at: clock.now_rfc3339(),
        passed,
        failure_signature: None,
        failed_aspect: None,
        extra: Map::new(),
    });
}

fn apply_transition(
    task: &mut TaskRuntimeState,
    tr: Transition,
    clock: &dyn Clock,
) -> CoreResult<()> {
    let status = TaskStatus::parse(&task.status)?;
    match tr {
        Transition::Start => match status {
            TaskStatus::Pending => {
                task.status = TaskStatus::Running.as_str().to_string();
                Ok(())
            }
            other => Err(CoreError::invalid_input(format!(
                "invalid transition Start from {}",
                other.as_str()
            ))),
        },
        Transition::Complete { output } => match status {
            TaskStatus::Running => {
                task.status = TaskStatus::Done.as_str().to_string();
                task.output = output;
                append_attempt(task, clock, true);
                Ok(())
            }
            other => Err(CoreError::invalid_input(format!(
                "invalid transition Complete from {}",
                other.as_str()
            ))),
        },
        Transition::Fail { error } => match status {
            TaskStatus::Running => {
                task.status = TaskStatus::Failed.as_str().to_string();
                task.error = error;
                append_attempt(task, clock, false);
                Ok(())
            }
            other => Err(CoreError::invalid_input(format!(
                "invalid transition Fail from {}",
                other.as_str()
            ))),
        },
        Transition::Reset => match status {
            TaskStatus::Pending => Ok(()), // no-op
            TaskStatus::Running | TaskStatus::Done | TaskStatus::Failed | TaskStatus::Skipped => {
                task.status = TaskStatus::Pending.as_str().to_string();
                task.output.clear();
                task.error.clear();
                Ok(())
            }
        },
        Transition::Skip => match status {
            TaskStatus::Pending => {
                task.status = TaskStatus::Skipped.as_str().to_string();
                Ok(())
            }
            other => Err(CoreError::invalid_input(format!(
                "invalid transition Skip from {}",
                other.as_str()
            ))),
        },
    }
}

fn is_failed_or_skipped(status: &str) -> bool {
    status == TaskStatus::Failed.as_str() || status == TaskStatus::Skipped.as_str()
}

/// Fixpoint: mark `PENDING` tasks `SKIPPED` when any dependency is `FAILED` or `SKIPPED`.
///
/// - Only `PENDING` may become `SKIPPED` (`RUNNING` never auto-skipped).
/// - `DONE` / `FAILED` / `SKIPPED` are untouched.
/// - A dependency missing from `state` is treated as **not** FAILED/SKIPPED.
/// - Scans task ids lexicographically each round; returns how many tasks changed.
/// - Idempotent: a second call returns `0`.
pub fn apply_cascading_skip(state: &mut RuntimeStateV1, doc: &DagDocument) -> usize {
    let views = iter_task_views(doc);
    let mut by_id: Vec<_> = views.into_iter().map(|v| (v.id, v.depends_on)).collect();
    by_id.sort_by(|a, b| a.0.cmp(&b.0));

    let mut total_changed = 0usize;
    loop {
        let mut round_ids: Vec<String> = Vec::new();
        for (id, deps) in &by_id {
            let Some(task) = state.tasks.get(id) else {
                continue;
            };
            if task.status != TaskStatus::Pending.as_str() {
                continue;
            }
            let blocked = deps.iter().any(|dep| {
                state
                    .tasks
                    .get(dep)
                    .map(|t| is_failed_or_skipped(&t.status))
                    .unwrap_or(false)
            });
            if blocked {
                round_ids.push(id.clone());
            }
        }
        if round_ids.is_empty() {
            break;
        }
        for id in round_ids {
            if let Some(task) = state.tasks.get_mut(&id) {
                if task.status == TaskStatus::Pending.as_str() {
                    task.status = TaskStatus::Skipped.as_str().to_string();
                    total_changed += 1;
                }
            }
        }
    }
    total_changed
}

/// Load or create `.dare/state.json`, merge DAG tasks as PENDING, cascade skip, save.
/// Does **not** write the canvas (T-20).
pub fn ensure_state(
    root: &ProjectRoot,
    doc: &DagDocument,
    clock: &dyn Clock,
) -> CoreResult<RuntimeStateV1> {
    let rel = state_rel()?;
    let _lock = FileLock::try_acquire(root, &rel)?;
    let mut state = load_or_empty(root, &rel)?;
    merge_dag_tasks(&mut state, doc);
    apply_cascading_skip(&mut state, doc);
    state.updated_at = clock.now_rfc3339();
    save_runtime_state(root, &rel, &state)?;
    Ok(state)
}

/// Atomically apply a status transition under the state file lock.
///
/// When `refresh` is [`RefreshCanvas::Yes`], writes `DARE/.canvas.md` after save
/// (ranks from `compute_ranks` when available, else `None`).
pub fn transition(
    root: &ProjectRoot,
    doc: &DagDocument,
    task_id: &str,
    tr: Transition,
    clock: &dyn Clock,
    refresh: RefreshCanvas,
) -> CoreResult<RuntimeStateV1> {
    let rel = state_rel()?;
    let _lock = FileLock::try_acquire(root, &rel)?;
    let mut state = load_or_empty(root, &rel)?;
    merge_dag_tasks(&mut state, doc);

    if !dag_contains(doc, task_id) {
        return Err(CoreError::not_found(format!(
            "task not found in DAG: {task_id}"
        )));
    }

    let task = state
        .tasks
        .get_mut(task_id)
        .ok_or_else(|| CoreError::not_found(format!("task not found in DAG: {task_id}")))?;
    apply_transition(task, tr, clock)?;

    apply_cascading_skip(&mut state, doc);
    state.updated_at = clock.now_rfc3339();
    save_runtime_state(root, &rel, &state)?;

    if matches!(refresh, RefreshCanvas::Yes) {
        let ranks = compute_ranks(doc).ok();
        canvas::write(root, doc, &state, ranks.as_ref(), clock)?;
    }

    Ok(state)
}

fn utc_rfc3339_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    const SECS_PER_DAY: u64 = 86400;
    let days = secs / SECS_PER_DAY;
    let rem = secs % SECS_PER_DAY;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;
    let (y, m, d) = civil_from_days(days as i64);
    format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z")
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::TaskStatus;
    use dare_contracts::parse_dag_yaml;
    use dare_core::fs::FileLock;
    use dare_core::{CoreError, ProjectRoot};
    use std::fs;
    use std::path::PathBuf;

    fn fixture(name: &str) -> String {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("../../tests/fixtures/dag");
        p.push(name);
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
    }

    fn task(status: TaskStatus, depends_on: &[&str]) -> TaskRuntimeState {
        TaskRuntimeState {
            status: status.as_str().to_string(),
            output: String::new(),
            error: String::new(),
            tokens: None,
            duration: None,
            attempts: Vec::new(),
            parent_id: None,
            depends_on: depends_on.iter().map(|s| (*s).to_string()).collect(),
            extra: Map::new(),
        }
    }

    fn empty_state() -> RuntimeStateV1 {
        RuntimeStateV1 {
            version: 1,
            updated_at: "2026-01-01T00:00:00Z".into(),
            tasks: BTreeMap::new(),
            extra: Map::new(),
        }
    }

    fn root_tmp() -> (tempfile::TempDir, ProjectRoot) {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("dare.config.json"), "{}").unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        (dir, root)
    }

    fn clock() -> FixedClock {
        FixedClock("2026-07-22T12:00:00Z".into())
    }

    #[test]
    fn skip_marks_pending_children() {
        let doc = parse_dag_yaml(&fixture("skip-cascade.v21.yaml")).unwrap();
        let mut state = empty_state();
        state
            .tasks
            .insert("task-root".into(), task(TaskStatus::Failed, &[]));
        state.tasks.insert(
            "task-child".into(),
            task(TaskStatus::Pending, &["task-root"]),
        );
        state.tasks.insert(
            "task-grand".into(),
            task(TaskStatus::Pending, &["task-child"]),
        );
        state.tasks.insert(
            "task-sibling".into(),
            task(TaskStatus::Pending, &["task-root"]),
        );
        state.tasks.insert(
            "task-leaf".into(),
            task(TaskStatus::Pending, &["task-sibling"]),
        );

        let n = apply_cascading_skip(&mut state, &doc);
        assert_eq!(n, 4);
        assert_eq!(
            state.tasks.get("task-root").unwrap().status,
            TaskStatus::Failed.as_str()
        );
        assert_eq!(
            state.tasks.get("task-child").unwrap().status,
            TaskStatus::Skipped.as_str()
        );
        assert_eq!(
            state.tasks.get("task-grand").unwrap().status,
            TaskStatus::Skipped.as_str()
        );
        assert_eq!(
            state.tasks.get("task-sibling").unwrap().status,
            TaskStatus::Skipped.as_str()
        );
        assert_eq!(
            state.tasks.get("task-leaf").unwrap().status,
            TaskStatus::Skipped.as_str()
        );
    }

    #[test]
    fn skip_idempotent() {
        let doc = parse_dag_yaml(&fixture("skip-cascade.v21.yaml")).unwrap();
        let mut state = empty_state();
        state
            .tasks
            .insert("task-root".into(), task(TaskStatus::Failed, &[]));
        state.tasks.insert(
            "task-child".into(),
            task(TaskStatus::Pending, &["task-root"]),
        );
        state.tasks.insert(
            "task-grand".into(),
            task(TaskStatus::Pending, &["task-child"]),
        );
        state.tasks.insert(
            "task-sibling".into(),
            task(TaskStatus::Pending, &["task-root"]),
        );
        state.tasks.insert(
            "task-leaf".into(),
            task(TaskStatus::Pending, &["task-sibling"]),
        );

        let first = apply_cascading_skip(&mut state, &doc);
        assert!(first > 0);
        let second = apply_cascading_skip(&mut state, &doc);
        assert_eq!(second, 0);
    }

    #[test]
    fn skip_ignores_running() {
        let doc = parse_dag_yaml(&fixture("skip-cascade.v21.yaml")).unwrap();
        let mut state = empty_state();
        state
            .tasks
            .insert("task-root".into(), task(TaskStatus::Failed, &[]));
        state.tasks.insert(
            "task-child".into(),
            task(TaskStatus::Running, &["task-root"]),
        );
        state.tasks.insert(
            "task-grand".into(),
            task(TaskStatus::Pending, &["task-child"]),
        );
        state.tasks.insert(
            "task-sibling".into(),
            task(TaskStatus::Pending, &["task-root"]),
        );
        state.tasks.insert(
            "task-leaf".into(),
            task(TaskStatus::Pending, &["task-sibling"]),
        );

        let n = apply_cascading_skip(&mut state, &doc);
        assert_eq!(
            state.tasks.get("task-child").unwrap().status,
            TaskStatus::Running.as_str()
        );
        // sibling + leaf skipped; grand stays PENDING (dep is RUNNING, not FAILED/SKIPPED)
        assert_eq!(n, 2);
        assert_eq!(
            state.tasks.get("task-grand").unwrap().status,
            TaskStatus::Pending.as_str()
        );
        assert_eq!(
            state.tasks.get("task-sibling").unwrap().status,
            TaskStatus::Skipped.as_str()
        );
        assert_eq!(
            state.tasks.get("task-leaf").unwrap().status,
            TaskStatus::Skipped.as_str()
        );
    }

    #[test]
    fn skip_missing_dep_in_state_does_not_trigger() {
        let doc = parse_dag_yaml(&fixture("skip-cascade.v21.yaml")).unwrap();
        let mut state = empty_state();
        // task-root absent from state
        state.tasks.insert(
            "task-child".into(),
            task(TaskStatus::Pending, &["task-root"]),
        );
        let n = apply_cascading_skip(&mut state, &doc);
        assert_eq!(n, 0);
        assert_eq!(
            state.tasks.get("task-child").unwrap().status,
            TaskStatus::Pending.as_str()
        );
    }

    #[test]
    fn ensure_creates_pending() {
        let (_dir, root) = root_tmp();
        let doc = parse_dag_yaml(&fixture("skip-cascade.v21.yaml")).unwrap();
        let state = ensure_state(&root, &doc, &clock()).unwrap();
        assert_eq!(state.version, 1);
        assert_eq!(state.updated_at, "2026-07-22T12:00:00Z");
        assert_eq!(state.tasks.len(), 5);
        for id in [
            "task-root",
            "task-child",
            "task-grand",
            "task-sibling",
            "task-leaf",
        ] {
            let t = state.tasks.get(id).expect(id);
            assert_eq!(t.status, TaskStatus::Pending.as_str());
        }
        assert_eq!(
            state.tasks.get("task-child").unwrap().depends_on,
            vec!["task-root".to_string()]
        );
        let on_disk = root.as_path().as_std_path().join(STATE_REL);
        assert!(on_disk.is_file());
    }

    #[test]
    fn transition_start_complete() {
        let (_dir, root) = root_tmp();
        let doc = parse_dag_yaml(&fixture("valid.v21.yaml")).unwrap();
        ensure_state(&root, &doc, &clock()).unwrap();

        let mid = transition(
            &root,
            &doc,
            "task-001",
            Transition::Start,
            &clock(),
            RefreshCanvas::No,
        )
        .unwrap();
        assert_eq!(
            mid.tasks.get("task-001").unwrap().status,
            TaskStatus::Running.as_str()
        );

        let done = transition(
            &root,
            &doc,
            "task-001",
            Transition::Complete {
                output: "ok".into(),
            },
            &clock(),
            RefreshCanvas::No,
        )
        .unwrap();
        let t = done.tasks.get("task-001").unwrap();
        assert_eq!(t.status, TaskStatus::Done.as_str());
        assert_eq!(t.output, "ok");
        assert_eq!(t.attempts.len(), 1);
        assert!(t.attempts[0].passed);
        assert_eq!(t.attempts[0].n, 1);
        assert_eq!(t.attempts[0].at, "2026-07-22T12:00:00Z");
    }

    #[test]
    fn transition_fail_skip_cascade() {
        let (_dir, root) = root_tmp();
        let doc = parse_dag_yaml(&fixture("skip-cascade.v21.yaml")).unwrap();
        ensure_state(&root, &doc, &clock()).unwrap();

        transition(
            &root,
            &doc,
            "task-root",
            Transition::Start,
            &clock(),
            RefreshCanvas::No,
        )
        .unwrap();
        let state = transition(
            &root,
            &doc,
            "task-root",
            Transition::Fail {
                error: "boom".into(),
            },
            &clock(),
            RefreshCanvas::No,
        )
        .unwrap();

        assert_eq!(
            state.tasks.get("task-root").unwrap().status,
            TaskStatus::Failed.as_str()
        );
        assert_eq!(state.tasks.get("task-root").unwrap().error, "boom");
        assert_eq!(
            state.tasks.get("task-child").unwrap().status,
            TaskStatus::Skipped.as_str()
        );
        assert_eq!(
            state.tasks.get("task-grand").unwrap().status,
            TaskStatus::Skipped.as_str()
        );
        assert_eq!(
            state.tasks.get("task-sibling").unwrap().status,
            TaskStatus::Skipped.as_str()
        );
        assert_eq!(
            state.tasks.get("task-leaf").unwrap().status,
            TaskStatus::Skipped.as_str()
        );
    }

    #[test]
    fn transition_invalid() {
        let (_dir, root) = root_tmp();
        let doc = parse_dag_yaml(&fixture("valid.v21.yaml")).unwrap();
        ensure_state(&root, &doc, &clock()).unwrap();

        let err = transition(
            &root,
            &doc,
            "task-001",
            Transition::Complete { output: "x".into() },
            &clock(),
            RefreshCanvas::No,
        )
        .unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)));
        assert!(err.to_string().contains("invalid transition"));

        let missing = transition(
            &root,
            &doc,
            "no-such-task",
            Transition::Start,
            &clock(),
            RefreshCanvas::No,
        )
        .unwrap_err();
        assert!(matches!(missing, CoreError::NotFound(_)));

        // Start → DONE path, then Start again is invalid
        transition(
            &root,
            &doc,
            "task-001",
            Transition::Start,
            &clock(),
            RefreshCanvas::No,
        )
        .unwrap();
        transition(
            &root,
            &doc,
            "task-001",
            Transition::Complete {
                output: "done".into(),
            },
            &clock(),
            RefreshCanvas::No,
        )
        .unwrap();
        let err = transition(
            &root,
            &doc,
            "task-001",
            Transition::Start,
            &clock(),
            RefreshCanvas::No,
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("invalid transition Start from DONE"));
    }

    #[test]
    fn lock_contention() {
        let (_dir, root) = root_tmp();
        let doc = parse_dag_yaml(&fixture("valid.v21.yaml")).unwrap();
        ensure_state(&root, &doc, &clock()).unwrap();

        let rel = SafeRelativePath::new(STATE_REL).unwrap();
        let held = FileLock::try_acquire(&root, &rel).expect("hold lock");
        let err = transition(
            &root,
            &doc,
            "task-001",
            Transition::Start,
            &clock(),
            RefreshCanvas::No,
        )
        .unwrap_err();
        assert!(matches!(err, CoreError::Io(_)));
        assert!(err.to_string().contains("file lock held"));
        drop(held);

        // After drop, transition succeeds
        let ok = transition(
            &root,
            &doc,
            "task-001",
            Transition::Start,
            &clock(),
            RefreshCanvas::No,
        );
        assert!(ok.is_ok());
    }

    #[test]
    fn version_2_rejected() {
        let (_dir, root) = root_tmp();
        let doc = parse_dag_yaml(&fixture("valid.v21.yaml")).unwrap();
        let state_path = root.as_path().as_std_path().join(STATE_REL);
        fs::create_dir_all(state_path.parent().unwrap()).unwrap();
        fs::write(
            &state_path,
            r#"{"version":2,"updatedAt":"2026-01-01T00:00:00Z","tasks":{}}"#,
        )
        .unwrap();

        let err = ensure_state(&root, &doc, &clock()).unwrap_err();
        assert!(matches!(err, CoreError::Config(_)));
        assert!(err.to_string().contains("unsupported state version"));
    }
}
