//! Refine scoring and sub-DAG splice (microplano 033).

use std::collections::HashSet;
use std::fmt;

use dare_contracts::{
    load_dag, save_dag, save_runtime_state, DagDocument, DagTask, RuntimeStateV1, TaskRuntimeState,
};
use dare_core::fs::FileLock;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::report::{ValidateOptions, DEFAULT_DAG_REL};
use crate::state::{ensure_state, Clock, SystemClock, STATE_REL};
use crate::validate::{find_cycle_path, is_kebab_id, validate_dag, ValidateFsContext};

/// Maximum parentId nesting for refined tasks (root depth 0).
pub const MAX_SUBDAG_DEPTH: u32 = 2;

pub const REPORT_SCHEMA: u32 = 1;
pub const STATUS_SPLIT: &str = "SPLIT";
pub const MSG_STRICT: &str = "Refine strict: level requires split (HIGH|CRITICAL).";

const HEAVY_KEYWORDS: &[&str] = &[
    "migration",
    "refactor",
    "auth",
    "security",
    "rewrite",
    "workspace",
    "graph",
    "oauth",
    "crypto",
    "distributed",
];

/// Refine complexity level (report only — YAML stays LOW|MED|HIGH).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RefineLevel {
    Low,
    Med,
    High,
    Critical,
}

impl RefineLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            RefineLevel::Low => "LOW",
            RefineLevel::Med => "MED",
            RefineLevel::High => "HIGH",
            RefineLevel::Critical => "CRITICAL",
        }
    }

    pub fn recommends_split(self) -> bool {
        matches!(self, RefineLevel::High | RefineLevel::Critical)
    }
}

impl fmt::Display for RefineLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexitySignals {
    pub file_count: u32,
    pub prompt_chars: u32,
    pub depends_count: u32,
    pub heavy_keywords: Vec<String>,
    pub dag_complexity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexityReport {
    pub score: u32,
    pub level: RefineLevel,
    pub signals: ComplexitySignals,
    pub recommends_split: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposedSubtask {
    pub id: String,
    pub title: String,
    pub depends_on: Vec<String>,
    pub complexity: String,
    pub subtask_prompt: String,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitProposal {
    pub parent_id: String,
    pub subtasks: Vec<ProposedSubtask>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefineReport {
    pub schema_version: u32,
    pub task_id: String,
    pub report: ComplexityReport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposal: Option<SplitProposal>,
    pub applied: bool,
    pub noop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubDagError {
    Cycle {
        path: Vec<String>,
    },
    MaxDepth {
        task_id: String,
        depth: u32,
        max: u32,
    },
    TaskNotFound {
        task_id: String,
    },
    Invalid {
        message: String,
    },
}

impl fmt::Display for SubDagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubDagError::Cycle { path } => {
                write!(f, "CycleError: {}", path.join(" -> "))
            }
            SubDagError::MaxDepth {
                task_id,
                depth,
                max,
            } => write!(
                f,
                "MaxDepthError: task {task_id} depth {depth} exceeds max {max}"
            ),
            SubDagError::TaskNotFound { task_id } => {
                write!(f, "task not found: {task_id}")
            }
            SubDagError::Invalid { message } => f.write_str(message),
        }
    }
}

impl std::error::Error for SubDagError {}

impl From<SubDagError> for CoreError {
    fn from(err: SubDagError) -> Self {
        match err {
            SubDagError::TaskNotFound { task_id } => {
                CoreError::not_found(format!("task not found: {task_id}"))
            }
            other => CoreError::invalid_input(other.to_string()),
        }
    }
}

/// Options for `run_refine` / apply.
#[derive(Debug, Clone)]
pub struct RefineOptions {
    pub task_id: String,
    pub split: bool,
    pub apply: bool,
    pub strict: bool,
    pub dag_rel: String,
}

impl Default for RefineOptions {
    fn default() -> Self {
        Self {
            task_id: String::new(),
            split: false,
            apply: false,
            strict: false,
            dag_rel: DEFAULT_DAG_REL.to_string(),
        }
    }
}

pub fn level_from_score(score: u32) -> RefineLevel {
    match score {
        0..=5 => RefineLevel::Low,
        6..=11 => RefineLevel::Med,
        12..=17 => RefineLevel::High,
        _ => RefineLevel::Critical,
    }
}

pub fn assess_complexity(signals: &ComplexitySignals) -> ComplexityReport {
    let mut score = 0u32;
    score += (signals.file_count.saturating_mul(2)).min(10);
    score += (signals.prompt_chars / 400).min(6);
    score += signals.depends_count.min(4);
    let kw = (signals.heavy_keywords.len() as u32)
        .saturating_mul(3)
        .min(9);
    score += kw;
    score += match signals.dag_complexity.as_str() {
        "MED" => 2,
        "HIGH" => 4,
        _ => 0,
    };
    let level = level_from_score(score);
    ComplexityReport {
        score,
        level,
        signals: signals.clone(),
        recommends_split: level.recommends_split(),
    }
}

fn find_heavy_keywords(text: &str) -> Vec<String> {
    let lower = text.to_ascii_lowercase();
    let mut out = Vec::new();
    for kw in HEAVY_KEYWORDS {
        if lower.contains(kw) && !out.iter().any(|x: &String| x == *kw) {
            out.push((*kw).to_string());
        }
    }
    out.sort();
    out
}

/// Extract backtick paths from markdown tables (EXECUTION section 3 style).
pub fn parse_spec_file_paths(markdown: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut in_section3 = false;
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            let title = trimmed.trim_start_matches('#').trim();
            in_section3 = title.starts_with('3')
                || title.to_ascii_lowercase().contains("arquivos")
                || title.to_ascii_lowercase().contains("files to");
            continue;
        }
        if !in_section3 || !trimmed.starts_with('|') {
            continue;
        }
        let mut parts = trimmed.split('`');
        let _ = parts.next();
        while let Some(inside) = parts.next() {
            let path = inside.trim();
            if !path.is_empty()
                && !path.contains(' ')
                && !path.contains("..")
                && (path.contains('/') || path.contains('.'))
            {
                let norm = path.replace('\\', "/");
                if !paths.iter().any(|p| p == &norm) {
                    paths.push(norm);
                }
            }
            let _ = parts.next();
        }
    }
    paths
}

fn task_id_is_path_safe(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

fn find_v21_task<'a>(doc: &'a DagDocument, id: &str) -> Option<&'a DagTask> {
    match doc {
        DagDocument::V21(d) => d.tasks.iter().find(|t| t.id == id),
        DagDocument::Legacy(_) => None,
    }
}

/// Depth via `parentId` chain in runtime state (missing → 0).
pub fn task_depth(state: &RuntimeStateV1, task_id: &str) -> u32 {
    let mut depth = 0u32;
    let mut seen = HashSet::new();
    let mut cur = task_id.to_string();
    while let Some(task) = state.tasks.get(&cur) {
        let Some(parent) = task.parent_id.as_ref() else {
            break;
        };
        if !seen.insert(parent.clone()) {
            break;
        }
        depth = depth.saturating_add(1);
        cur = parent.clone();
        if depth > MAX_SUBDAG_DEPTH.saturating_add(4) {
            break;
        }
    }
    depth
}

pub fn collect_signals(
    root: &ProjectRoot,
    doc: &DagDocument,
    task_id: &str,
) -> CoreResult<ComplexitySignals> {
    let task = find_v21_task(doc, task_id).ok_or_else(|| SubDagError::TaskNotFound {
        task_id: task_id.to_string(),
    })?;

    let mut file_count = 0u32;
    let spec_candidates = [
        format!("DARE/EXECUTION/{task_id}.md"),
        if task.spec_file.is_empty() {
            String::new()
        } else if task.spec_file.starts_with("DARE/") || task.spec_file.starts_with("EXECUTION") {
            if task.spec_file.starts_with("DARE/") {
                task.spec_file.clone()
            } else {
                format!("DARE/{}", task.spec_file.trim_start_matches('/'))
            }
        } else {
            format!("DARE/EXECUTION/{}.md", task_id)
        },
    ];
    for rel in &spec_candidates {
        if rel.is_empty() {
            continue;
        }
        if let Ok(safe) = SafeRelativePath::new(rel) {
            if let Ok(bytes) = dare_contracts::read_limited(root, &safe) {
                if let Ok(text) = String::from_utf8(bytes) {
                    file_count = parse_spec_file_paths(&text).len() as u32;
                    break;
                }
            }
        }
    }

    let blob = format!("{} {}", task.title, task.subtask_prompt);
    Ok(ComplexitySignals {
        file_count,
        prompt_chars: task.subtask_prompt.chars().count() as u32,
        depends_count: task.depends_on.len() as u32,
        heavy_keywords: find_heavy_keywords(&blob),
        dag_complexity: task.complexity.clone(),
    })
}

pub fn propose_split(task: &DagTask, report: &ComplexityReport) -> Option<SplitProposal> {
    if !report.recommends_split {
        return None;
    }
    let n = ((1 + report.score / 6).clamp(2, 4)) as usize;
    let suffixes = ['a', 'b', 'c', 'd'];
    let mut subtasks = Vec::with_capacity(n);
    for i in 0..n {
        let id = format!("{}-{}", task.id, suffixes[i]);
        if !is_kebab_id(&id) {
            continue;
        }
        let depends_on = if i == 0 {
            task.depends_on.clone()
        } else {
            vec![format!("{}-{}", task.id, suffixes[i - 1])]
        };
        let child_complexity = if report.level == RefineLevel::Critical && i == 0 {
            "MED"
        } else {
            "LOW"
        };
        let slice = prompt_slice(&task.subtask_prompt, i, n);
        subtasks.push(ProposedSubtask {
            id,
            title: format!("{} ({})", task.title, suffixes[i]),
            depends_on,
            complexity: child_complexity.to_string(),
            subtask_prompt: format!(
                "Part {}/{} of {}: {}\n{}",
                i + 1,
                n,
                task.id,
                task.title,
                slice
            ),
            rationale: format!("Deterministic split axis {} of {}", i + 1, n),
        });
    }
    if subtasks.len() < 2 {
        return None;
    }
    Some(SplitProposal {
        parent_id: task.id.clone(),
        subtasks,
    })
}

fn prompt_slice(prompt: &str, index: usize, n: usize) -> String {
    if prompt.is_empty() {
        return format!("Implement slice {index} of {n}.");
    }
    let chars: Vec<char> = prompt.chars().collect();
    let len = chars.len();
    let chunk = len.div_ceil(n);
    let start = index.saturating_mul(chunk).min(len);
    let end = start.saturating_add(chunk).min(len);
    chars[start..end].iter().collect()
}

/// Replace parent task with proposed children; rewire dependents to last child.
pub fn splice_sub_dag(
    doc: &DagDocument,
    proposal: &SplitProposal,
) -> Result<DagDocument, SubDagError> {
    let DagDocument::V21(mut dag) = doc.clone() else {
        return Err(SubDagError::Invalid {
            message: "refine requires v2.1 DAG (tasks array)".into(),
        });
    };

    let parent_idx = dag
        .tasks
        .iter()
        .position(|t| t.id == proposal.parent_id)
        .ok_or_else(|| SubDagError::TaskNotFound {
            task_id: proposal.parent_id.clone(),
        })?;

    if proposal.subtasks.len() < 2 {
        return Err(SubDagError::Invalid {
            message: "proposal must contain at least 2 subtasks".into(),
        });
    }

    let last_id = proposal.subtasks.last().unwrap().id.clone();
    let parent_id = proposal.parent_id.clone();

    // Rewire dependents that listed the parent.
    for t in &mut dag.tasks {
        if t.id == parent_id {
            continue;
        }
        for dep in &mut t.depends_on {
            if dep == &parent_id {
                *dep = last_id.clone();
            }
        }
    }

    let new_tasks: Vec<DagTask> = proposal
        .subtasks
        .iter()
        .map(|s| DagTask {
            id: s.id.clone(),
            title: s.title.clone(),
            depends_on: s.depends_on.clone(),
            complexity: s.complexity.clone(),
            subtask_prompt: s.subtask_prompt.clone(),
            spec_file: format!("EXECUTION/{}.md", s.id),
            extra: Map::new(),
        })
        .collect();

    dag.tasks.remove(parent_idx);
    for (i, t) in new_tasks.into_iter().enumerate() {
        dag.tasks.insert(parent_idx + i, t);
    }

    let out = DagDocument::V21(dag);
    if let Some(path) = find_cycle_path(&out) {
        return Err(SubDagError::Cycle { path });
    }
    Ok(out)
}

fn pending_with_parent(depends_on: Vec<String>, parent_id: Option<String>) -> TaskRuntimeState {
    TaskRuntimeState {
        status: "PENDING".into(),
        output: String::new(),
        error: String::new(),
        tokens: None,
        duration: None,
        attempts: Vec::new(),
        parent_id,
        depends_on,
        extra: Map::new(),
    }
}

/// Merge spliced children into state; mark parent SPLIT; preserve parentId/dependsOn.
pub fn merge_state_after_splice(
    state: &mut RuntimeStateV1,
    proposal: &SplitProposal,
    clock: &dyn Clock,
) {
    if let Some(parent) = state.tasks.get_mut(&proposal.parent_id) {
        parent.status = STATUS_SPLIT.to_string();
    } else {
        state.tasks.insert(
            proposal.parent_id.clone(),
            TaskRuntimeState {
                status: STATUS_SPLIT.into(),
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
    }

    for sub in &proposal.subtasks {
        let entry = state.tasks.entry(sub.id.clone()).or_insert_with(|| {
            pending_with_parent(sub.depends_on.clone(), Some(proposal.parent_id.clone()))
        });
        entry.parent_id = Some(proposal.parent_id.clone());
        entry.depends_on = sub.depends_on.clone();
        if entry.status == STATUS_SPLIT {
            entry.status = "PENDING".into();
        }
    }
    state.updated_at = clock.now_rfc3339();
}

/// Full refine pipeline (read-only unless `opts.apply`).
pub fn run_refine(
    root: &ProjectRoot,
    opts: &RefineOptions,
    clock: &dyn Clock,
) -> CoreResult<RefineReport> {
    if !task_id_is_path_safe(&opts.task_id) {
        return Err(CoreError::invalid_input("task id is not path-safe"));
    }

    let dag_rel = SafeRelativePath::new(&opts.dag_rel)?;
    let doc = load_dag(root, &dag_rel)?;
    if matches!(doc, DagDocument::Legacy(_)) {
        return Err(CoreError::invalid_input(
            "refine requires v2.1 DAG (tasks array)",
        ));
    }

    let mut state = ensure_state(root, &doc, clock)?;

    let task = find_v21_task(&doc, &opts.task_id)
        .ok_or_else(|| CoreError::not_found(format!("task not found: {}", opts.task_id)))?;

    let depth = task_depth(&state, &opts.task_id);
    let child_depth = depth.saturating_add(1);
    if opts.apply && child_depth > MAX_SUBDAG_DEPTH {
        return Err(SubDagError::MaxDepth {
            task_id: opts.task_id.clone(),
            depth: child_depth,
            max: MAX_SUBDAG_DEPTH,
        }
        .into());
    }

    let signals = collect_signals(root, &doc, &opts.task_id)?;
    let complexity = assess_complexity(&signals);

    let want_proposal = opts.split || opts.apply || complexity.recommends_split;
    let proposal = if want_proposal {
        propose_split(task, &complexity)
    } else {
        None
    };

    let noop = !complexity.recommends_split && !opts.apply;
    let mut applied = false;

    if opts.apply {
        let proposal = proposal.clone().ok_or_else(|| {
            CoreError::invalid_input("cannot apply: no split proposal (level LOW|MED)")
        })?;

        if child_depth > MAX_SUBDAG_DEPTH {
            return Err(SubDagError::MaxDepth {
                task_id: opts.task_id.clone(),
                depth: child_depth,
                max: MAX_SUBDAG_DEPTH,
            }
            .into());
        }

        let new_doc = splice_sub_dag(&doc, &proposal)?;
        let ctx = ValidateFsContext {
            root,
            dag_path_display: opts.dag_rel.clone(),
        };
        let validation = validate_dag(&new_doc, &ValidateOptions { strict: false }, &ctx);
        if validation.error_count > 0 {
            let msg = validation
                .issues
                .iter()
                .find(|i| i.severity == crate::report::IssueSeverity::Error)
                .map(|i| i.message.clone())
                .unwrap_or_else(|| "validate failed after splice".into());
            return Err(CoreError::invalid_input(msg));
        }

        let state_rel = SafeRelativePath::new(STATE_REL)?;
        let _lock = FileLock::try_acquire(root, &state_rel)?;
        // Mutate the already-loaded state under a single lock (do not call ensure_state).
        merge_state_after_splice(&mut state, &proposal, clock);
        for view in crate::graph::iter_task_views(&new_doc) {
            state.tasks.entry(view.id.clone()).or_insert_with(|| {
                pending_with_parent(view.depends_on.clone(), Some(proposal.parent_id.clone()))
            });
            if let Some(t) = state.tasks.get_mut(&view.id) {
                if proposal.subtasks.iter().any(|s| s.id == view.id) {
                    t.parent_id = Some(proposal.parent_id.clone());
                    t.depends_on = view.depends_on.clone();
                }
            }
        }
        state.updated_at = clock.now_rfc3339();
        save_runtime_state(root, &state_rel, &state)?;
        drop(_lock);
        save_dag(root, &dag_rel, &new_doc)?;
        applied = true;
    }

    Ok(RefineReport {
        schema_version: REPORT_SCHEMA,
        task_id: opts.task_id.clone(),
        report: complexity,
        proposal,
        applied,
        noop: noop && !applied,
    })
}

pub fn run_refine_default(root: &ProjectRoot, opts: &RefineOptions) -> CoreResult<RefineReport> {
    run_refine(root, opts, &SystemClock)
}

pub fn format_refine_human(report: &RefineReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Refine {}: score {} level {}\n",
        report.task_id, report.report.score, report.report.level
    ));
    out.push_str(&format!(
        "recommendsSplit={} applied={} noop={}\n",
        report.report.recommends_split, report.applied, report.noop
    ));
    out.push_str(&format!(
        "signals: files={} promptChars={} deps={} dagComplexity={} keywords={:?}\n",
        report.report.signals.file_count,
        report.report.signals.prompt_chars,
        report.report.signals.depends_count,
        report.report.signals.dag_complexity,
        report.report.signals.heavy_keywords
    ));
    if let Some(p) = &report.proposal {
        out.push_str(&format!("proposal ({} subtasks):\n", p.subtasks.len()));
        for s in &p.subtasks {
            out.push_str(&format!(
                "  - {} [{}] deps={:?}\n",
                s.id, s.complexity, s.depends_on
            ));
        }
    } else {
        out.push_str("proposal: (none)\n");
    }
    if report.noop {
        out.push_str("No-op: task is manageable (LOW|MED).\n");
    }
    out
}

pub fn refine_report_to_json(report: &RefineReport) -> CoreResult<Value> {
    serde_json::to_value(report).map_err(|e| CoreError::internal(e.to_string()))
}

pub fn strict_should_exit_2(report: &RefineReport, strict: bool) -> bool {
    strict && report.report.level.recommends_split()
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_contracts::parse_dag_yaml;
    use std::collections::BTreeMap;
    use std::fs;

    fn signals(files: u32, prompt: u32, deps: u32, kws: &[&str], dag: &str) -> ComplexitySignals {
        ComplexitySignals {
            file_count: files,
            prompt_chars: prompt,
            depends_count: deps,
            heavy_keywords: kws.iter().map(|s| (*s).to_string()).collect(),
            dag_complexity: dag.into(),
        }
    }

    #[test]
    fn score_thresholds_boundaries() {
        assert_eq!(level_from_score(0), RefineLevel::Low);
        assert_eq!(level_from_score(5), RefineLevel::Low);
        assert_eq!(level_from_score(6), RefineLevel::Med);
        assert_eq!(level_from_score(11), RefineLevel::Med);
        assert_eq!(level_from_score(12), RefineLevel::High);
        assert_eq!(level_from_score(17), RefineLevel::High);
        assert_eq!(level_from_score(18), RefineLevel::Critical);
    }

    #[test]
    fn propose_split_high_only() {
        let task = DagTask {
            id: "task-010".into(),
            title: "Big feature".into(),
            depends_on: vec!["task-001".into()],
            complexity: "HIGH".into(),
            subtask_prompt: "x".repeat(500),
            spec_file: String::new(),
            extra: Map::new(),
        };
        let low = assess_complexity(&signals(0, 10, 0, &[], "LOW"));
        assert!(propose_split(&task, &low).is_none());

        let high = assess_complexity(&signals(
            5,
            2000,
            3,
            &["refactor", "auth", "migration"],
            "HIGH",
        ));
        assert!(high.recommends_split);
        let prop = propose_split(&task, &high).expect("proposal");
        assert!(prop.subtasks.len() >= 2);
        assert_eq!(prop.subtasks[0].depends_on, vec!["task-001".to_string()]);
        assert_eq!(
            prop.subtasks[1].depends_on,
            vec![prop.subtasks[0].id.clone()]
        );
    }

    #[test]
    fn splice_rewires_dependents() {
        let yaml = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-001
    title: A
    depends_on: []
    complexity: LOW
    subtask_prompt: a
  - id: task-010
    title: Big
    depends_on: [task-001]
    complexity: HIGH
    subtask_prompt: refactor auth migration workspace security rewrite
  - id: task-020
    title: After
    depends_on: [task-010]
    complexity: LOW
    subtask_prompt: b
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        let task = find_v21_task(&doc, "task-010").unwrap();
        let report = assess_complexity(&ComplexitySignals {
            file_count: 5,
            prompt_chars: 2000,
            depends_count: 1,
            heavy_keywords: vec!["refactor".into(), "auth".into(), "migration".into()],
            dag_complexity: "HIGH".into(),
        });
        let prop = propose_split(task, &report).unwrap();
        let new_doc = splice_sub_dag(&doc, &prop).unwrap();
        let DagDocument::V21(d) = new_doc else {
            panic!("v21");
        };
        assert!(!d.tasks.iter().any(|t| t.id == "task-010"));
        let last = prop.subtasks.last().unwrap().id.clone();
        let after = d.tasks.iter().find(|t| t.id == "task-020").unwrap();
        assert_eq!(after.depends_on, vec![last]);
    }

    #[test]
    fn max_depth_blocks() {
        let mut state = RuntimeStateV1 {
            version: 1,
            updated_at: "t".into(),
            tasks: BTreeMap::new(),
            extra: Map::new(),
        };
        state
            .tasks
            .insert("root".into(), pending_with_parent(vec![], None));
        state.tasks.insert(
            "root-a".into(),
            pending_with_parent(vec![], Some("root".into())),
        );
        state.tasks.insert(
            "root-a-a".into(),
            pending_with_parent(vec![], Some("root-a".into())),
        );
        assert_eq!(task_depth(&state, "root"), 0);
        assert_eq!(task_depth(&state, "root-a"), 1);
        assert_eq!(task_depth(&state, "root-a-a"), 2);
        // child of root-a-a would be depth 3 > MAX 2
        assert!(task_depth(&state, "root-a-a") + 1 > MAX_SUBDAG_DEPTH);
    }

    #[test]
    fn cycle_blocks() {
        let yaml = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-a
    title: A
    depends_on: []
    complexity: HIGH
    subtask_prompt: refactor auth migration security
  - id: task-b
    title: B
    depends_on: [task-a]
    complexity: LOW
    subtask_prompt: b
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        // Craft a broken proposal that depends on itself via rewire nonsense —
        // force cycle by making subtask depend on task-b which depends on last child.
        let bad = SplitProposal {
            parent_id: "task-a".into(),
            subtasks: vec![
                ProposedSubtask {
                    id: "task-a-a".into(),
                    title: "a".into(),
                    depends_on: vec!["task-b".into()],
                    complexity: "LOW".into(),
                    subtask_prompt: "x".into(),
                    rationale: "t".into(),
                },
                ProposedSubtask {
                    id: "task-a-b".into(),
                    title: "b".into(),
                    depends_on: vec!["task-a-a".into()],
                    complexity: "LOW".into(),
                    subtask_prompt: "y".into(),
                    rationale: "t".into(),
                },
            ],
        };
        // After splice: task-b depends on task-a-b; task-a-a depends on task-b → cycle
        let err = splice_sub_dag(&doc, &bad).unwrap_err();
        assert!(matches!(err, SubDagError::Cycle { .. }), "{err:?}");
    }

    #[test]
    fn preserves_parent_id_in_state() {
        let mut state = RuntimeStateV1 {
            version: 1,
            updated_at: String::new(),
            tasks: BTreeMap::new(),
            extra: Map::new(),
        };
        state.tasks.insert(
            "task-010".into(),
            pending_with_parent(vec!["task-001".into()], None),
        );
        let prop = SplitProposal {
            parent_id: "task-010".into(),
            subtasks: vec![
                ProposedSubtask {
                    id: "task-010-a".into(),
                    title: "a".into(),
                    depends_on: vec!["task-001".into()],
                    complexity: "LOW".into(),
                    subtask_prompt: "x".into(),
                    rationale: "t".into(),
                },
                ProposedSubtask {
                    id: "task-010-b".into(),
                    title: "b".into(),
                    depends_on: vec!["task-010-a".into()],
                    complexity: "LOW".into(),
                    subtask_prompt: "y".into(),
                    rationale: "t".into(),
                },
            ],
        };
        merge_state_after_splice(&mut state, &prop, &crate::state::FixedClock("t".into()));
        assert_eq!(state.tasks.get("task-010").unwrap().status, STATUS_SPLIT);
        assert_eq!(
            state.tasks.get("task-010-a").unwrap().parent_id.as_deref(),
            Some("task-010")
        );
        assert_eq!(
            state.tasks.get("task-010-b").unwrap().depends_on,
            vec!["task-010-a".to_string()]
        );
    }

    #[test]
    fn apply_refine_writes_dag() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("DARE/EXECUTION")).unwrap();
        fs::write(dir.path().join("dare.config.json"), "{}").unwrap();
        let yaml = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-big
    title: Big refactor auth migration security rewrite workspace
    depends_on: []
    complexity: HIGH
    subtask_prompt: |
      refactor auth migration security rewrite workspace graph oauth crypto distributed
      Implement a large feature with many moving parts across the codebase carefully.
      Include migration scripts and auth security hardening and workspace layout changes.
    spec_file: EXECUTION/task-big.md
"#;
        fs::write(dir.path().join("DARE/dare-dag.yaml"), yaml).unwrap();
        fs::write(
            dir.path().join("DARE/EXECUTION/task-big.md"),
            r#"# TASK
## 3. ARQUIVOS A CRIAR / MODIFICAR
| Ação | Caminho | Descrição |
|------|---------|-----------|
| CRIAR | `src/a.rs` | a |
| CRIAR | `src/b.rs` | b |
| CRIAR | `src/c.rs` | c |
| CRIAR | `src/d.rs` | d |
| CRIAR | `src/e.rs` | e |
"#,
        )
        .unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let opts = RefineOptions {
            task_id: "task-big".into(),
            split: true,
            apply: true,
            strict: false,
            dag_rel: DEFAULT_DAG_REL.into(),
        };
        let report = run_refine(&root, &opts, &crate::state::FixedClock("t".into())).unwrap();
        assert!(report.applied, "{report:?}");
        assert!(report.proposal.is_some());
        let saved = fs::read_to_string(dir.path().join("DARE/dare-dag.yaml")).unwrap();
        assert!(!saved.contains("id: task-big\n") || !saved.contains("title: Big"));
        assert!(saved.contains("task-big-a"));
    }
}
