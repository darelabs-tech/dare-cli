//! Deterministic DAG validation rules (microplano 020).

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use dare_contracts::{load_dag, DagDocument, DagLimits, DagV21, LegacyDag};
use dare_core::{CoreResult, ProjectRoot, SafeRelativePath};

use crate::report::{
    IssueSeverity, ValidateOptions, ValidationIssue, ValidationReport, COMPLEXITY_ALLOWED, MSG_MAX,
    VALIDATION_SCHEMA_VERSION,
};

/// Filesystem context for `spec_file` existence checks.
pub struct ValidateFsContext<'a> {
    pub root: &'a ProjectRoot,
    pub dag_path_display: String,
}

#[derive(Clone)]
struct TaskView {
    id: String,
    title: String,
    depends_on: Vec<String>,
    complexity: String,
    subtask_prompt: Option<String>,
    spec_file: Option<String>,
}

/// ASCII kebab-case: `^[a-z0-9]+(-[a-z0-9]+)*$` without a regex crate.
pub fn is_kebab_id(id: &str) -> bool {
    if id.is_empty() || id.starts_with('-') || id.ends_with('-') || id.contains("--") {
        return false;
    }
    id.split('-').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    })
}

fn truncate_msg(s: &str) -> String {
    if s.chars().count() <= MSG_MAX {
        return s.to_string();
    }
    s.chars().take(MSG_MAX).collect()
}

fn issue(
    severity: IssueSeverity,
    code: &str,
    task_id: &str,
    message: impl Into<String>,
    path: Option<Vec<String>>,
) -> ValidationIssue {
    ValidationIssue {
        severity,
        code: code.to_string(),
        task_id: task_id.to_string(),
        message: truncate_msg(&message.into()),
        path,
    }
}

fn materialize_v21(dag: &DagV21) -> (Vec<TaskView>, Option<&DagLimits>) {
    let tasks = dag
        .tasks
        .iter()
        .map(|t| TaskView {
            id: t.id.clone(),
            title: t.title.clone(),
            depends_on: t.depends_on.clone(),
            complexity: t.complexity.clone(),
            subtask_prompt: Some(t.subtask_prompt.clone()),
            spec_file: Some(t.spec_file.clone()),
        })
        .collect();
    (tasks, Some(&dag.limits))
}

fn materialize_legacy(dag: &LegacyDag) -> Vec<TaskView> {
    dag.tasks
        .iter()
        .map(|(id, t)| TaskView {
            id: id.clone(),
            title: t.title.clone(),
            depends_on: t.depends_on.clone(),
            complexity: t.complexity.clone(),
            subtask_prompt: None,
            spec_file: None,
        })
        .collect()
}

fn sort_issues(issues: &mut [ValidationIssue]) {
    issues.sort_by(|a, b| {
        let sev = match (a.severity, b.severity) {
            (IssueSeverity::Error, IssueSeverity::Warning) => Ordering::Less,
            (IssueSeverity::Warning, IssueSeverity::Error) => Ordering::Greater,
            _ => Ordering::Equal,
        };
        sev.then_with(|| a.code.cmp(&b.code))
            .then_with(|| a.task_id.cmp(&b.task_id))
            .then_with(|| a.message.cmp(&b.message))
    });
}

fn collect_per_task(
    tasks: &[TaskView],
    ctx: &ValidateFsContext<'_>,
    is_v21: bool,
) -> Vec<ValidationIssue> {
    let mut out = Vec::new();
    for t in tasks {
        if !is_kebab_id(&t.id) {
            out.push(issue(
                IssueSeverity::Error,
                "invalid_id",
                &t.id,
                format!("task id must be kebab-case: {}", t.id),
                None,
            ));
        }
        if t.title.trim().is_empty() {
            out.push(issue(
                IssueSeverity::Error,
                "empty_title",
                &t.id,
                "title must not be empty",
                None,
            ));
        }
        if !COMPLEXITY_ALLOWED.contains(&t.complexity.as_str()) {
            out.push(issue(
                IssueSeverity::Error,
                "invalid_complexity",
                &t.id,
                format!(
                    "complexity must be one of LOW|MED|HIGH (case-sensitive), got: {}",
                    t.complexity
                ),
                None,
            ));
        }
        if is_v21 {
            let prompt = t.subtask_prompt.as_deref().unwrap_or("");
            let spec = t.spec_file.as_deref().unwrap_or("");
            if prompt.trim().is_empty() && spec.trim().is_empty() {
                out.push(issue(
                    IssueSeverity::Error,
                    "missing_prompt_or_spec",
                    &t.id,
                    "subtask_prompt and spec_file are both empty",
                    None,
                ));
            }
            if !spec.trim().is_empty() {
                let abs = ctx.root.as_path().as_std_path().join("DARE").join(spec);
                if !abs.is_file() {
                    out.push(issue(
                        IssueSeverity::Warning,
                        "missing_spec_file",
                        &t.id,
                        format!("spec_file not found under DARE/: {spec}"),
                        None,
                    ));
                }
            }
        }
    }
    out
}

fn collect_duplicates(tasks: &[TaskView]) -> Vec<ValidationIssue> {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut out = Vec::new();
    for t in tasks {
        if !seen.insert(t.id.as_str()) {
            out.push(issue(
                IssueSeverity::Error,
                "duplicate_id",
                &t.id,
                format!("duplicate task id: {}", t.id),
                None,
            ));
        }
    }
    out
}

fn collect_deps(tasks: &[TaskView]) -> Vec<ValidationIssue> {
    let ids: HashSet<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    let mut out = Vec::new();
    for t in tasks {
        for dep in &t.depends_on {
            if dep == &t.id {
                out.push(issue(
                    IssueSeverity::Error,
                    "self_dependency",
                    &t.id,
                    format!("task depends on itself: {}", t.id),
                    None,
                ));
            } else if !ids.contains(dep.as_str()) {
                out.push(issue(
                    IssueSeverity::Error,
                    "missing_dependency",
                    &t.id,
                    format!("depends_on references unknown id: {dep}"),
                    None,
                ));
            }
        }
    }
    out
}

fn canonicalize_cycle(cycle: &[String]) -> Vec<String> {
    let n = cycle.len();
    if n == 0 {
        return vec![];
    }
    let mut best_i = 0;
    for i in 1..n {
        if cycle[i] < cycle[best_i] {
            best_i = i;
        }
    }
    let mut out: Vec<String> = cycle[best_i..].to_vec();
    out.extend_from_slice(&cycle[..best_i]);
    let start = out[0].clone();
    out.push(start);
    out
}

/// Edge a → b means a depends_on b.
fn collect_cycles(tasks: &[TaskView]) -> Vec<ValidationIssue> {
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for t in tasks {
        adj.entry(t.id.as_str()).or_default();
        for dep in &t.depends_on {
            adj.entry(t.id.as_str()).or_default().push(dep.as_str());
        }
    }
    for v in adj.values_mut() {
        v.sort_unstable();
        v.dedup();
    }

    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        White,
        Gray,
        Black,
    }
    let mut color: HashMap<&str, Color> = HashMap::new();
    for id in adj.keys() {
        color.insert(*id, Color::White);
    }

    let mut stack: Vec<&str> = Vec::new();
    let mut cycles: Vec<Vec<String>> = Vec::new();
    let mut reported: BTreeSet<String> = BTreeSet::new();

    fn dfs<'a>(
        u: &'a str,
        adj: &BTreeMap<&'a str, Vec<&'a str>>,
        color: &mut HashMap<&'a str, Color>,
        stack: &mut Vec<&'a str>,
        cycles: &mut Vec<Vec<String>>,
        reported: &mut BTreeSet<String>,
    ) {
        color.insert(u, Color::Gray);
        stack.push(u);
        if let Some(neigh) = adj.get(u) {
            for &v in neigh {
                match color.get(v).copied().unwrap_or(Color::White) {
                    Color::Gray => {
                        let start = stack.iter().position(|x| *x == v).unwrap_or(0);
                        let raw: Vec<String> =
                            stack[start..].iter().map(|s| (*s).to_string()).collect();
                        let path = canonicalize_cycle(&raw);
                        let key = path.join("->");
                        if reported.insert(key) {
                            cycles.push(path);
                        }
                    }
                    Color::White => dfs(v, adj, color, stack, cycles, reported),
                    Color::Black => {}
                }
            }
        }
        stack.pop();
        color.insert(u, Color::Black);
    }

    let ids: Vec<&str> = adj.keys().copied().collect();
    for id in ids {
        if color.get(id) == Some(&Color::White) {
            dfs(id, &adj, &mut color, &mut stack, &mut cycles, &mut reported);
        }
    }

    let mut out = Vec::new();
    for path in cycles {
        let task_id = path.first().cloned().unwrap_or_default();
        let msg = format!("dependency cycle detected: {}", path.join(" -> "));
        out.push(issue(
            IssueSeverity::Error,
            "cycle",
            &task_id,
            msg,
            Some(path),
        ));
    }
    out
}

fn collect_limits(limits: Option<&DagLimits>) -> Vec<ValidationIssue> {
    let Some(l) = limits else {
        return vec![];
    };
    if l.parent_context_chars == 0 || l.task_output_chars == 0 || l.timeout_seconds == 0 {
        return vec![issue(
            IssueSeverity::Warning,
            "invalid_limits",
            "",
            "limits parent_context_chars, task_output_chars, and timeout_seconds must be > 0",
            None,
        )];
    }
    vec![]
}

pub fn validate_dag(
    doc: &DagDocument,
    opts: &ValidateOptions,
    ctx: &ValidateFsContext<'_>,
) -> ValidationReport {
    let (tasks, limits, format) = match doc {
        DagDocument::V21(d) => {
            let (t, l) = materialize_v21(d);
            (t, l, "v2.1")
        }
        DagDocument::Legacy(d) => (materialize_legacy(d), None, "legacy"),
    };
    let is_v21 = format == "v2.1";

    let mut issues = Vec::new();
    issues.extend(collect_per_task(&tasks, ctx, is_v21));
    issues.extend(collect_duplicates(&tasks));
    issues.extend(collect_deps(&tasks));
    issues.extend(collect_cycles(&tasks));
    issues.extend(collect_limits(limits));

    sort_issues(&mut issues);

    let error_count = issues
        .iter()
        .filter(|i| i.severity == IssueSeverity::Error)
        .count() as u32;
    let warning_count = issues
        .iter()
        .filter(|i| i.severity == IssueSeverity::Warning)
        .count() as u32;
    let ok = ValidationReport::compute_ok(error_count, warning_count, opts.strict);

    ValidationReport {
        schema_version: VALIDATION_SCHEMA_VERSION,
        mode: "validate".into(),
        ok,
        dag_path: ctx.dag_path_display.clone(),
        format: format.into(),
        task_count: tasks.len() as u32,
        error_count,
        warning_count,
        strict: opts.strict,
        issues,
    }
}

pub fn validate_path(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    opts: &ValidateOptions,
) -> CoreResult<ValidationReport> {
    let doc = load_dag(root, rel)?;
    let display = rel.as_str().replace('\\', "/");
    let ctx = ValidateFsContext {
        root,
        dag_path_display: display,
    };
    Ok(validate_dag(&doc, opts, &ctx))
}
