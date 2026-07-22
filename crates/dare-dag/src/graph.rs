//! Longest-path ranks over a DAG document (microplano 026).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;

use dare_contracts::{DagDocument, RuntimeStateV1};
use dare_core::CoreError;

use crate::report::{IssueSeverity, ValidateOptions};
use crate::status::TaskStatus;
use crate::validate::{find_cycle_path, validate_dag, ValidateFsContext};

/// Thin view of a task for ranking / scheduling helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskView {
    pub id: String,
    pub title: String,
    pub depends_on: Vec<String>,
}

/// Errors from rank / graph analysis (not full validate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagGraphError {
    Cycle { path: Vec<String> },
    MissingDependency { task_id: String, missing: String },
    InvalidDag { message: String },
}

impl fmt::Display for DagGraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DagGraphError::Cycle { path } => {
                write!(f, "dependency cycle detected: {}", path.join(" -> "))
            }
            DagGraphError::MissingDependency { task_id, missing } => write!(
                f,
                "depends_on references unknown id: {missing} (task {task_id})"
            ),
            DagGraphError::InvalidDag { message } => f.write_str(message),
        }
    }
}

impl std::error::Error for DagGraphError {}

impl From<DagGraphError> for CoreError {
    fn from(err: DagGraphError) -> Self {
        CoreError::invalid_input(err.to_string())
    }
}

/// Materialize id/title/depends_on for V2.1 (array order) and Legacy (BTreeMap lexico).
pub fn iter_task_views(doc: &DagDocument) -> Vec<TaskView> {
    match doc {
        DagDocument::V21(d) => d
            .tasks
            .iter()
            .map(|t| TaskView {
                id: t.id.clone(),
                title: t.title.clone(),
                depends_on: t.depends_on.clone(),
            })
            .collect(),
        DagDocument::Legacy(d) => d
            .tasks
            .iter()
            .map(|(id, t)| TaskView {
                id: id.clone(),
                title: t.title.clone(),
                depends_on: t.depends_on.clone(),
            })
            .collect(),
    }
}

/// Longest-path ranks: roots (`depends_on` empty) = 0; else `1 + max(rank(deps))`.
///
/// Pré: prefer a validated DAG (020). This does **not** run full validate.
/// Detects cycles via `find_cycle_path` first; missing dep ids → `MissingDependency`.
pub fn compute_ranks(doc: &DagDocument) -> Result<BTreeMap<String, u32>, DagGraphError> {
    if let Some(path) = find_cycle_path(doc) {
        return Err(DagGraphError::Cycle { path });
    }

    let views = iter_task_views(doc);
    if views.is_empty() {
        return Ok(BTreeMap::new());
    }

    let ids: HashSet<&str> = views.iter().map(|t| t.id.as_str()).collect();
    let by_id: HashMap<&str, &TaskView> = views.iter().map(|t| (t.id.as_str(), t)).collect();

    for t in &views {
        for dep in &t.depends_on {
            if !ids.contains(dep.as_str()) {
                return Err(DagGraphError::MissingDependency {
                    task_id: t.id.clone(),
                    missing: dep.clone(),
                });
            }
        }
    }

    let mut memo: HashMap<String, u32> = HashMap::new();
    let mut visiting: HashSet<String> = HashSet::new();

    fn rank_of(
        id: &str,
        by_id: &HashMap<&str, &TaskView>,
        memo: &mut HashMap<String, u32>,
        visiting: &mut HashSet<String>,
    ) -> Result<u32, DagGraphError> {
        if let Some(&r) = memo.get(id) {
            return Ok(r);
        }
        if !visiting.insert(id.to_string()) {
            // Back-edge while memoizing — should be rare after find_cycle_path.
            return Err(DagGraphError::Cycle {
                path: vec![id.to_string(), id.to_string()],
            });
        }
        let task = by_id.get(id).ok_or_else(|| DagGraphError::InvalidDag {
            message: format!("unknown task id during rank: {id}"),
        })?;
        let rank = if task.depends_on.is_empty() {
            0
        } else {
            let mut max_dep = 0u32;
            for dep in &task.depends_on {
                max_dep = max_dep.max(rank_of(dep, by_id, memo, visiting)?);
            }
            1 + max_dep
        };
        visiting.remove(id);
        memo.insert(id.to_string(), rank);
        Ok(rank)
    }

    let mut out = BTreeMap::new();
    for t in &views {
        let r = rank_of(&t.id, &by_id, &mut memo, &mut visiting)?;
        out.insert(t.id.clone(), r);
    }
    Ok(out)
}

/// Group task ids by rank; each bucket sorted lexicographically.
pub fn tasks_by_rank(ranks: &BTreeMap<String, u32>) -> BTreeMap<u32, Vec<String>> {
    let mut out: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    for (id, &rank) in ranks {
        out.entry(rank).or_default().push(id.clone());
    }
    for ids in out.values_mut() {
        ids.sort();
    }
    out
}

/// Validate then compute ranks. Any `Error` severity → `InvalidDag`; warnings alone are OK.
pub fn compute_ranks_validated(
    doc: &DagDocument,
    opts: &ValidateOptions,
    ctx: &ValidateFsContext<'_>,
) -> Result<BTreeMap<String, u32>, DagGraphError> {
    let report = validate_dag(doc, opts, ctx);
    if let Some(err) = report
        .issues
        .iter()
        .find(|i| i.severity == IssueSeverity::Error)
    {
        return Err(DagGraphError::InvalidDag {
            message: err.message.clone(),
        });
    }
    compute_ranks(doc)
}

/// PENDING tasks whose every dependency is DONE, ordered by rank↑ then id lexico.
pub fn next_executable(
    doc: &DagDocument,
    state: &RuntimeStateV1,
    ranks: &BTreeMap<String, u32>,
) -> Vec<String> {
    let mut candidates: Vec<(u32, String)> = Vec::new();
    for t in iter_task_views(doc) {
        let Some(rt) = state.tasks.get(&t.id) else {
            continue;
        };
        if rt.status != TaskStatus::Pending.as_str() {
            continue;
        }
        let deps_done = t.depends_on.iter().all(|dep| {
            state
                .tasks
                .get(dep)
                .is_some_and(|d| d.status == TaskStatus::Done.as_str())
        });
        if !deps_done {
            continue;
        }
        let rank = ranks.get(&t.id).copied().unwrap_or(0);
        candidates.push((rank, t.id));
    }
    candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    candidates.into_iter().map(|(_, id)| id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{ValidateOptions, DEFAULT_DAG_REL};
    use crate::state::apply_cascading_skip;
    use dare_contracts::{parse_dag_yaml, TaskRuntimeState};
    use dare_core::ProjectRoot;
    use proptest::prelude::*;
    use serde_json::Map;
    use std::fs;
    use std::path::PathBuf;

    fn fixture(name: &str) -> String {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("../../tests/fixtures/dag");
        p.push(name);
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
    }

    fn load_ranks_golden(name: &str) -> BTreeMap<String, u32> {
        let text = fixture(name);
        serde_json::from_str(&text).expect("parse ranks golden")
    }

    fn root_with_dare() -> (tempfile::TempDir, ProjectRoot) {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("DARE")).unwrap();
        fs::write(dir.path().join("dare.config.json"), "{}").unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        (dir, root)
    }

    fn ctx(root: &ProjectRoot) -> ValidateFsContext<'_> {
        ValidateFsContext {
            root,
            dag_path_display: DEFAULT_DAG_REL.into(),
        }
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

    /// Build an acyclic V2.1 DAG: edges only from lower index → higher index.
    fn acyclic_yaml(n: usize, edges: &[(usize, usize)]) -> String {
        let mut deps: Vec<Vec<String>> = vec![Vec::new(); n];
        for &(from, to) in edges {
            if from < to && from < n && to < n {
                let id = format!("task-{from:03}");
                if !deps[to].contains(&id) {
                    deps[to].push(id);
                }
            }
        }
        let mut tasks = String::new();
        for (i, task_deps) in deps.iter().enumerate() {
            let dep_list = if task_deps.is_empty() {
                String::new()
            } else {
                format!(
                    "\n    depends_on: [{}]",
                    task_deps
                        .iter()
                        .map(|d| format!("\"{d}\""))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            tasks.push_str(&format!(
                r#"
  - id: task-{i:03}
    title: Task {i}
    complexity: LOW
    subtask_prompt: p{i}{dep_list}
"#
            ));
        }
        format!(
            r#"title: "Prop"
version: "1.0.0"
tasks:{tasks}"#
        )
    }

    #[test]
    fn compute_ranks_chain() {
        let doc = parse_dag_yaml(&fixture("ranks-chain.v21.yaml")).unwrap();
        let ranks = compute_ranks(&doc).unwrap();
        assert_eq!(ranks, load_ranks_golden("ranks-chain.ranks.json"));
    }

    #[test]
    fn compute_ranks_diamond() {
        let doc = parse_dag_yaml(&fixture("ranks-diamond.v21.yaml")).unwrap();
        let ranks = compute_ranks(&doc).unwrap();
        assert_eq!(ranks, load_ranks_golden("ranks-diamond.ranks.json"));
    }

    #[test]
    fn compute_ranks_fanout() {
        let doc = parse_dag_yaml(&fixture("ranks-fanout.v21.yaml")).unwrap();
        let ranks = compute_ranks(&doc).unwrap();
        assert_eq!(ranks, load_ranks_golden("ranks-fanout.ranks.json"));
    }

    #[test]
    fn compute_ranks_cycle_errors() {
        let doc = parse_dag_yaml(&fixture("cycle.v21.yaml")).unwrap();
        match compute_ranks(&doc) {
            Err(DagGraphError::Cycle { path }) => {
                assert_eq!(path.first(), path.last());
                assert_eq!(path[0], "task-001");
            }
            other => panic!("expected Cycle, got {other:?}"),
        }

        let missing = r#"
title: "Missing"
version: "1.0.0"
tasks:
  - id: task-alpha
    title: Alpha
    depends_on: [task-ghost]
    complexity: LOW
    subtask_prompt: x
"#;
        let doc = parse_dag_yaml(missing).unwrap();
        match compute_ranks(&doc) {
            Err(DagGraphError::MissingDependency { task_id, missing }) => {
                assert_eq!(task_id, "task-alpha");
                assert_eq!(missing, "task-ghost");
            }
            other => panic!("expected MissingDependency, got {other:?}"),
        }

        let empty = r#"
title: "Empty"
version: "1.0.0"
tasks: []
"#;
        let doc = parse_dag_yaml(empty).unwrap();
        assert!(compute_ranks(&doc).unwrap().is_empty());
    }

    #[test]
    fn tasks_by_rank_sorted() {
        let mut ranks = BTreeMap::new();
        ranks.insert("task-zeta".into(), 1);
        ranks.insert("task-alpha".into(), 0);
        ranks.insert("task-beta".into(), 1);
        ranks.insert("task-gamma".into(), 2);
        let by = tasks_by_rank(&ranks);
        assert_eq!(by.get(&0).unwrap(), &vec!["task-alpha".to_string()]);
        assert_eq!(
            by.get(&1).unwrap(),
            &vec!["task-beta".to_string(), "task-zeta".to_string()]
        );
        assert_eq!(by.get(&2).unwrap(), &vec!["task-gamma".to_string()]);
    }

    #[test]
    fn iter_task_views_legacy() {
        let doc = parse_dag_yaml(&fixture("valid.legacy.yaml")).unwrap();
        let views = iter_task_views(&doc);
        assert_eq!(views.len(), 2);
        assert_eq!(views[0].id, "task-001");
        assert_eq!(views[0].title, "Legacy One");
        assert!(views[0].depends_on.is_empty());
        assert_eq!(views[1].id, "task-002");
        assert_eq!(views[1].depends_on, vec!["task-001".to_string()]);

        let ranks = compute_ranks(&doc).unwrap();
        assert_eq!(ranks.get("task-001"), Some(&0));
        assert_eq!(ranks.get("task-002"), Some(&1));
    }

    #[test]
    fn dag_graph_error_maps_to_invalid_input() {
        let err: CoreError = DagGraphError::InvalidDag {
            message: "bad".into(),
        }
        .into();
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    #[test]
    fn next_executable_order() {
        let doc = parse_dag_yaml(&fixture("ranks-diamond.v21.yaml")).unwrap();
        let ranks = compute_ranks(&doc).unwrap();
        let mut state = empty_state();
        state
            .tasks
            .insert("task-root".into(), task(TaskStatus::Done, &[]));
        state.tasks.insert(
            "task-left".into(),
            task(TaskStatus::Pending, &["task-root"]),
        );
        state.tasks.insert(
            "task-right".into(),
            task(TaskStatus::Pending, &["task-root"]),
        );
        state.tasks.insert(
            "task-join".into(),
            task(TaskStatus::Pending, &["task-left", "task-right"]),
        );

        let next = next_executable(&doc, &state, &ranks);
        assert_eq!(
            next,
            vec!["task-left".to_string(), "task-right".to_string()]
        );

        state.tasks.get_mut("task-left").unwrap().status = TaskStatus::Done.as_str().into();
        let next = next_executable(&doc, &state, &ranks);
        assert_eq!(next, vec!["task-right".to_string()]);

        state.tasks.get_mut("task-right").unwrap().status = TaskStatus::Done.as_str().into();
        let next = next_executable(&doc, &state, &ranks);
        assert_eq!(next, vec!["task-join".to_string()]);
    }

    #[test]
    fn compute_ranks_validated_rejects_cycle() {
        let (_d, root) = root_with_dare();
        let doc = parse_dag_yaml(&fixture("cycle.v21.yaml")).unwrap();
        let err = compute_ranks_validated(&doc, &ValidateOptions { strict: false }, &ctx(&root))
            .expect_err("cycle must fail validation");
        match err {
            DagGraphError::InvalidDag { message } => {
                assert!(!message.is_empty());
            }
            other => panic!("expected InvalidDag, got {other:?}"),
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn prop_ranks_gt_deps(
            n in 1usize..=8,
            edge_bits in prop::collection::vec(any::<u8>(), 0..=24),
        ) {
            let mut edges = Vec::new();
            for (i, &b) in edge_bits.iter().enumerate() {
                if n < 2 {
                    break;
                }
                let from = (b as usize) % n;
                let to = ((b as usize) / n.max(1) + i + 1) % n;
                match from.cmp(&to) {
                    std::cmp::Ordering::Less => edges.push((from, to)),
                    std::cmp::Ordering::Greater => edges.push((to, from)),
                    std::cmp::Ordering::Equal => {}
                }
            }
            let yaml = acyclic_yaml(n, &edges);
            let doc = parse_dag_yaml(&yaml).expect("parse acyclic yaml");
            let ranks = compute_ranks(&doc).expect("ranks on acyclic");
            for t in iter_task_views(&doc) {
                let r = *ranks.get(&t.id).expect("rank present");
                for dep in &t.depends_on {
                    let rd = *ranks.get(dep).expect("dep rank");
                    prop_assert!(
                        r > rd,
                        "rank({})={} must be > rank({})={}",
                        t.id,
                        r,
                        dep,
                        rd
                    );
                }
            }
        }

        #[test]
        fn prop_skip_idempotent(
            fail_idx in 0usize..5,
            extra_failed in prop::collection::vec(0usize..5, 0..=2),
        ) {
            let doc = parse_dag_yaml(&fixture("skip-cascade.v21.yaml")).unwrap();
            let ids = [
                "task-root",
                "task-child",
                "task-grand",
                "task-sibling",
                "task-leaf",
            ];
            let mut state = empty_state();
            for id in &ids {
                let deps: Vec<&str> = match *id {
                    "task-root" => vec![],
                    "task-child" | "task-sibling" => vec!["task-root"],
                    "task-grand" => vec!["task-child"],
                    "task-leaf" => vec!["task-sibling"],
                    _ => vec![],
                };
                state
                    .tasks
                    .insert((*id).into(), task(TaskStatus::Pending, &deps));
            }
            let mark = |state: &mut RuntimeStateV1, i: usize| {
                let id = ids[i % ids.len()];
                if let Some(t) = state.tasks.get_mut(id) {
                    t.status = TaskStatus::Failed.as_str().into();
                }
            };
            mark(&mut state, fail_idx);
            for i in &extra_failed {
                mark(&mut state, *i);
            }

            let _first = apply_cascading_skip(&mut state, &doc);
            let second = apply_cascading_skip(&mut state, &doc);
            prop_assert_eq!(second, 0);
        }
    }
}
