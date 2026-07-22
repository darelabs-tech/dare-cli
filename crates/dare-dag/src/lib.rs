//! DAG validation, graph ranks, and runtime status (microplanos 020 / 026).

mod canvas;
mod format;
mod graph;
mod report;
mod state;
mod status;
mod validate;
pub mod viz;

pub use canvas::{render, write, CANVAS_REL};
pub use format::{format_human, report_to_json};
pub use graph::{
    compute_ranks, compute_ranks_validated, iter_task_views, next_executable, tasks_by_rank,
    DagGraphError, TaskView,
};
pub use report::{
    IssueSeverity, ValidateOptions, ValidationIssue, ValidationReport, COMPLEXITY_ALLOWED,
    DEFAULT_DAG_REL, MSG_MAX, VALIDATION_SCHEMA_VERSION,
};
pub use state::{
    apply_cascading_skip, ensure_state, transition, Clock, FixedClock, RefreshCanvas, SystemClock,
    Transition, STATE_REL,
};
pub use status::TaskStatus;
pub use validate::{is_kebab_id, validate_dag, validate_path, ValidateFsContext};
pub use viz::{VizFormat, VizOptions, OUTPUT_CAP, TITLE_MAX_DEFAULT};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use dare_contracts::parse_dag_yaml;
    use dare_core::{ProjectRoot, SafeRelativePath};
    use std::fs;
    use std::path::PathBuf;

    fn fixture(name: &str) -> String {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("../../tests/fixtures/dag");
        p.push(name);
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
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

    #[test]
    fn valid_v21_ok() {
        let (_d, root) = root_with_dare();
        fs::write(
            root.as_path()
                .as_std_path()
                .join("DARE/EXECUTION/task-001.md"),
            "# ok",
        )
        .ok();
        fs::create_dir_all(root.as_path().as_std_path().join("DARE/EXECUTION")).unwrap();
        fs::write(
            root.as_path()
                .as_std_path()
                .join("DARE/EXECUTION/task-001.md"),
            "# ok",
        )
        .unwrap();
        let doc = parse_dag_yaml(&fixture("valid.v21.yaml")).unwrap();
        let r = validate_dag(&doc, &ValidateOptions { strict: false }, &ctx(&root));
        assert!(r.ok, "{:?}", r.issues);
        assert_eq!(r.format, "v2.1");
        assert_eq!(r.error_count, 0);
    }

    #[test]
    fn valid_legacy_ok() {
        let (_d, root) = root_with_dare();
        let doc = parse_dag_yaml(&fixture("valid.legacy.yaml")).unwrap();
        let r = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        assert!(r.ok, "{:?}", r.issues);
        assert_eq!(r.format, "legacy");
    }

    #[test]
    fn rejects_bad_id() {
        let (_d, root) = root_with_dare();
        let doc = parse_dag_yaml(&fixture("bad-id.v21.yaml")).unwrap();
        let r = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        assert!(!r.ok);
        assert!(r.issues.iter().any(|i| i.code == "invalid_id"));
    }

    #[test]
    fn rejects_duplicate_id() {
        let (_d, root) = root_with_dare();
        let yaml = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-001
    title: One
    complexity: LOW
    subtask_prompt: x
  - id: task-001
    title: Two
    complexity: LOW
    subtask_prompt: y
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        let r = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        assert!(r.issues.iter().any(|i| i.code == "duplicate_id"));
    }

    #[test]
    fn rejects_missing_dep() {
        let (_d, root) = root_with_dare();
        let doc = parse_dag_yaml(&fixture("missing-dep.v21.yaml")).unwrap();
        let r = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        assert!(r.issues.iter().any(|i| i.code == "missing_dependency"));
    }

    #[test]
    fn rejects_self_dep() {
        let (_d, root) = root_with_dare();
        let yaml = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-001
    title: One
    depends_on: [task-001]
    complexity: LOW
    subtask_prompt: x
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        let r = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        assert!(r.issues.iter().any(|i| i.code == "self_dependency"));
    }

    #[test]
    fn rejects_cycle_canonical_path() {
        let (_d, root) = root_with_dare();
        let doc = parse_dag_yaml(&fixture("cycle.v21.yaml")).unwrap();
        let r = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        let c = r.issues.iter().find(|i| i.code == "cycle").expect("cycle");
        let path = c.path.as_ref().expect("path");
        assert_eq!(path.first(), path.last());
        assert_eq!(path[0], "task-001"); // lexico min start after rotate
        assert!(r.issues.iter().filter(|i| i.code == "cycle").count() >= 1);
    }

    #[test]
    fn rejects_empty_title() {
        let (_d, root) = root_with_dare();
        let yaml = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-001
    title: "   "
    complexity: LOW
    subtask_prompt: x
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        let r = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        assert!(r.issues.iter().any(|i| i.code == "empty_title"));
    }

    #[test]
    fn rejects_bad_complexity_case() {
        let (_d, root) = root_with_dare();
        let yaml = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-001
    title: One
    complexity: low
    subtask_prompt: x
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        let r = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        assert!(r.issues.iter().any(|i| i.code == "invalid_complexity"));
    }

    #[test]
    fn rejects_missing_prompt_and_spec() {
        let (_d, root) = root_with_dare();
        let doc = parse_dag_yaml(&fixture("empty-prompt.v21.yaml")).unwrap();
        let r = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        assert!(r.issues.iter().any(|i| i.code == "missing_prompt_or_spec"));
    }

    #[test]
    fn warns_missing_spec_file() {
        let (_d, root) = root_with_dare();
        let doc = parse_dag_yaml(&fixture("warning-missing-spec.v21.yaml")).unwrap();
        let r = validate_dag(&doc, &ValidateOptions { strict: false }, &ctx(&root));
        assert!(r.ok);
        assert!(r.issues.iter().any(|i| i.code == "missing_spec_file"));
    }

    #[test]
    fn strict_fails_on_warning() {
        let (_d, root) = root_with_dare();
        let doc = parse_dag_yaml(&fixture("warning-missing-spec.v21.yaml")).unwrap();
        let r = validate_dag(&doc, &ValidateOptions { strict: true }, &ctx(&root));
        assert!(!r.ok);
        assert_eq!(r.warning_count, 1);
    }

    #[test]
    fn warns_zero_limits() {
        let (_d, root) = root_with_dare();
        let yaml = r#"
title: "T"
version: "1.0.0"
limits:
  parent_context_chars: 0
  task_output_chars: 4000
  timeout_seconds: 600
tasks:
  - id: task-001
    title: One
    complexity: LOW
    subtask_prompt: x
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        let r = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        assert!(r.issues.iter().any(|i| i.code == "invalid_limits"));
    }

    #[test]
    fn issues_sort_stable() {
        let (_d, root) = root_with_dare();
        let yaml = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-002
    title: ""
    complexity: BAD
    subtask_prompt: x
  - id: task-001
    title: ""
    complexity: BAD
    subtask_prompt: x
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        let r1 = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        let r2 = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        assert_eq!(r1.issues, r2.issues);
        // errors before warnings; codes/taskIds ordered
        for w in r1.issues.windows(2) {
            let a = &w[0];
            let b = &w[1];
            if let (IssueSeverity::Warning, IssueSeverity::Error) = (a.severity, b.severity) {
                panic!("unsorted severity");
            }
        }
    }

    #[test]
    fn legacy_skips_prompt_rules() {
        let (_d, root) = root_with_dare();
        let doc = parse_dag_yaml(&fixture("valid.legacy.yaml")).unwrap();
        let r = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        assert!(!r.issues.iter().any(|i| i.code == "missing_prompt_or_spec"));
        assert!(!r.issues.iter().any(|i| i.code == "missing_spec_file"));
    }

    #[test]
    fn validate_path_zero_writes() {
        let (dir, root) = root_with_dare();
        let rel_path = "DARE/dare-dag.yaml";
        fs::write(dir.path().join(rel_path), fixture("valid.v21.yaml")).unwrap();
        fs::create_dir_all(dir.path().join("DARE/EXECUTION")).unwrap();
        fs::write(dir.path().join("DARE/EXECUTION/task-001.md"), "#").unwrap();

        fn listing(base: &std::path::Path) -> Vec<String> {
            walkdir_simple(base, base)
        }
        fn walkdir_simple(base: &std::path::Path, cur: &std::path::Path) -> Vec<String> {
            let mut out = Vec::new();
            if let Ok(rd) = fs::read_dir(cur) {
                for e in rd.flatten() {
                    let p = e.path();
                    let rel = p
                        .strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    out.push(rel.clone());
                    if p.is_dir() {
                        out.extend(walkdir_simple(base, &p));
                    }
                }
            }
            out.sort();
            out
        }

        let before = listing(dir.path());
        let rel = SafeRelativePath::new(rel_path).unwrap();
        let _ = validate_path(&root, &rel, &ValidateOptions::default()).unwrap();
        let after = listing(dir.path());
        assert_eq!(before, after);
    }

    #[test]
    fn message_never_contains_long_prompt() {
        let (_d, root) = root_with_dare();
        let long = "P".repeat(1000);
        let yaml = format!(
            r#"
title: "T"
version: "1.0.0"
tasks:
  - id: Bad_ID
    title: One
    complexity: LOW
    subtask_prompt: "{long}"
"#
        );
        let doc = parse_dag_yaml(&yaml).unwrap();
        let r = validate_dag(&doc, &ValidateOptions::default(), &ctx(&root));
        for i in &r.issues {
            assert!(
                !i.message.contains(&long),
                "message leaked prompt: {}",
                i.message
            );
            assert!(i.message.chars().count() <= MSG_MAX);
        }
    }

    #[test]
    fn is_kebab_id_cases() {
        assert!(is_kebab_id("task-001"));
        assert!(is_kebab_id("a"));
        assert!(!is_kebab_id("Task-001"));
        assert!(!is_kebab_id("task_001"));
        assert!(!is_kebab_id("-task"));
        assert!(!is_kebab_id("task-"));
        assert!(!is_kebab_id("task--001"));
    }
}
