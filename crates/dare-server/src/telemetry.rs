//! Telemetry snapshot builder for the read-only dashboard.

use dare_contracts::{load_runtime_state, RuntimeStateV1, TelemetrySnapshot};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use dare_graph::{load_graph_config, open_graph};
use serde_json::{json, Map, Value};

const STATE_REL: &str = ".dare/state.json";

/// Build a `TelemetrySnapshot` from project state and soft graph availability.
pub fn build_telemetry_snapshot(root: &ProjectRoot) -> CoreResult<TelemetrySnapshot> {
    let dag = match load_runtime_state(root, &SafeRelativePath::new(STATE_REL)?) {
        Ok(state) => dag_counts_from_state(&state),
        Err(CoreError::NotFound(_)) => empty_dag_none(),
        Err(e) => return Err(e),
    };

    let drift = Map::from_iter([(
        "available".to_string(),
        Value::Bool(graph_available(root)),
    )]);

    Ok(TelemetrySnapshot {
        dag,
        gates: Map::new(),
        cost: Map::new(),
        best_of_n: Map::new(),
        guard: Map::new(),
        drift,
        extra: Map::new(),
    })
}

fn graph_available(root: &ProjectRoot) -> bool {
    match load_graph_config(root, None) {
        Ok(cfg) => open_graph(root, &cfg).is_ok(),
        Err(_) => false,
    }
}

fn empty_dag_none() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("tasksTotal".into(), json!(0));
    m.insert("done".into(), json!(0));
    m.insert("pending".into(), json!(0));
    m.insert("running".into(), json!(0));
    m.insert("failed".into(), json!(0));
    m.insert("skipped".into(), json!(0));
    m.insert("source".into(), json!("none"));
    m
}

fn dag_counts_from_state(state: &RuntimeStateV1) -> Map<String, Value> {
    let mut done = 0u64;
    let mut pending = 0u64;
    let mut running = 0u64;
    let mut failed = 0u64;
    let mut skipped = 0u64;

    for task in state.tasks.values() {
        match task.status.as_str() {
            "DONE" => done += 1,
            "RUNNING" => running += 1,
            "FAILED" => failed += 1,
            "SKIPPED" => skipped += 1,
            // PENDING and unknown statuses count as pending (TaskStatus convention).
            _ => pending += 1,
        }
    }

    let total = state.tasks.len() as u64;
    let mut m = Map::new();
    m.insert("tasksTotal".into(), json!(total));
    m.insert("done".into(), json!(done));
    m.insert("pending".into(), json!(pending));
    m.insert("running".into(), json!(running));
    m.insert("failed".into(), json!(failed));
    m.insert("skipped".into(), json!(skipped));
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_contracts::save_runtime_state;
    use dare_contracts::TaskRuntimeState;
    use std::collections::BTreeMap;

    #[test]
    fn telemetry_maps_keys() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let snap = build_telemetry_snapshot(&root).expect("snapshot");
        assert!(snap.dag.contains_key("tasksTotal"));
        assert!(snap.dag.contains_key("source"));
        assert_eq!(snap.dag.get("source"), Some(&json!("none")));
        assert!(snap.gates.is_empty());
        assert!(snap.cost.is_empty());
        assert!(snap.best_of_n.is_empty());
        assert!(snap.guard.is_empty());
        assert!(snap.drift.contains_key("available"));
        assert!(snap.drift.get("available").and_then(|v| v.as_bool()).is_some());
    }

    #[test]
    fn telemetry_counts_from_state() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let rel = SafeRelativePath::new(STATE_REL).unwrap();
        std::fs::create_dir_all(dir.path().join(".dare")).unwrap();
        let mut tasks = BTreeMap::new();
        tasks.insert(
            "a".into(),
            TaskRuntimeState {
                status: "DONE".into(),
                output: String::new(),
                error: String::new(),
                tokens: None,
                duration: None,
                attempts: Vec::new(),
                parent_id: None,
                depends_on: Vec::new(),
                extra: Map::new(),
            },
        );
        tasks.insert(
            "b".into(),
            TaskRuntimeState {
                status: "PENDING".into(),
                output: String::new(),
                error: String::new(),
                tokens: None,
                duration: None,
                attempts: Vec::new(),
                parent_id: None,
                depends_on: Vec::new(),
                extra: Map::new(),
            },
        );
        let state = RuntimeStateV1 {
            version: 1,
            updated_at: "2026-01-01T00:00:00Z".into(),
            tasks,
            extra: Map::new(),
        };
        save_runtime_state(&root, &rel, &state).unwrap();
        let snap = build_telemetry_snapshot(&root).expect("snapshot");
        assert_eq!(snap.dag.get("tasksTotal"), Some(&json!(2)));
        assert_eq!(snap.dag.get("done"), Some(&json!(1)));
        assert_eq!(snap.dag.get("pending"), Some(&json!(1)));
        assert!(!snap.dag.contains_key("source"));
    }
}
