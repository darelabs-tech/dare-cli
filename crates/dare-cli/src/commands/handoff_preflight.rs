//! Strict-mode preflight over the contract directory (D-06.1).
//!
//! Without this, enabling `handoff.strict` on a project that has no contracts
//! is discovered task by task, mid-run, as a repeated
//! `HANDOFF_CONTRACT_MISSING`. The preflight separates "I configured this
//! wrong" — one failure, at the start, with its own refusal code — from "this
//! task is missing evidence", which is a consumption failure. DESIGN §7 line
//! A05 requires exactly that separation.
//!
//! It runs only when the policy is strict, so the default path pays nothing.

use std::collections::BTreeSet;

use dare_contracts::{
    contract_dir_for_dag, load_contract, validate_contract, ContractViolation, DagDocument,
    HandoffPolicy, RefusalCode, ViolationSeverity,
};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use dare_dag::iter_task_views;

/// Prefix of every preflight refusal, so the cause is greppable in a log.
pub const MSG_PREFLIGHT_FAIL: &str = "handoff preflight failed";

/// How many violations a refusal message names before summarising the rest.
const MAX_LISTED_VIOLATIONS: usize = 10;

/// What the preflight found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightReport {
    pub schema_version: u32,
    pub strict: bool,
    pub tasks_total: u32,
    pub tasks_with_contract: u32,
    /// `(task_id, violation)`, in task order.
    pub violations: Vec<(String, ContractViolation)>,
}

/// Validates the contract directory before any prompt is composed.
///
/// Call only when `policy.strict`. Refuses with [`CoreError::GuardFail`],
/// which the renderer turns into exit code 6, when strict mode is on but no
/// task contract exists — naming the directory it looked in, because the usual
/// cause is a contract directory in the wrong place — or when any contract
/// carries a blocking violation.
///
/// A non-blocking violation is reported in the returned
/// [`PreflightReport::violations`] without refusing, so an over-broad scope
/// informs rather than stops the run.
pub fn run_handoff_preflight(
    root: &ProjectRoot,
    dag_rel: &SafeRelativePath,
    doc: &DagDocument,
    policy: &HandoffPolicy,
) -> CoreResult<PreflightReport> {
    let dir = contract_dir_for_dag(dag_rel)?;
    let views = iter_task_views(doc);
    let dag_task_ids: BTreeSet<String> = views.iter().map(|v| v.id.clone()).collect();

    let mut tasks_with_contract: u32 = 0;
    let mut violations: Vec<(String, ContractViolation)> = Vec::new();

    for view in &views {
        let Some(contract) = load_contract(root, dag_rel, &view.id)? else {
            continue;
        };
        tasks_with_contract = tasks_with_contract.saturating_add(1);
        for violation in validate_contract(&contract, &view.id, &dag_task_ids) {
            violations.push((view.id.clone(), violation));
        }
    }

    let report = PreflightReport {
        schema_version: 1,
        strict: policy.strict,
        tasks_total: u32::try_from(views.len()).unwrap_or(u32::MAX),
        tasks_with_contract,
        violations,
    };

    if report.tasks_with_contract == 0 {
        return Err(CoreError::guard_fail(format!(
            "{MSG_PREFLIGHT_FAIL}: strict mode enabled but no task contract found under {} ({})",
            dir.as_str(),
            RefusalCode::StrictNoContracts
        )));
    }

    let blocking: Vec<&(String, ContractViolation)> = report
        .violations
        .iter()
        .filter(|(_, v)| v.severity() == ViolationSeverity::Error)
        .collect();
    if !blocking.is_empty() {
        let listed: Vec<String> = blocking
            .iter()
            .take(MAX_LISTED_VIOLATIONS)
            .map(|(id, v)| format!("{id}: {v}"))
            .collect();
        let mut message = format!(
            "{MSG_PREFLIGHT_FAIL}: {} blocking contract violation(s); {}",
            blocking.len(),
            listed.join("; ")
        );
        if blocking.len() > MAX_LISTED_VIOLATIONS {
            message.push_str(&format!(
                "; and {} more",
                blocking.len() - MAX_LISTED_VIOLATIONS
            ));
        }
        return Err(CoreError::guard_fail(message));
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_contracts::{contract_rel, parse_dag_yaml, HandoffKind};
    use tempfile::tempdir;

    const DAG_REL: &str = "DARE/dare-dag.yaml";

    const V21_DAG: &str = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-001
    title: One
    depends_on: []
    complexity: LOW
    subtask_prompt: one
  - id: task-002
    title: Two
    depends_on: [task-001]
    complexity: LOW
    subtask_prompt: two
  - id: task-003
    title: Three
    depends_on: [task-002]
    complexity: LOW
    subtask_prompt: three
"#;

    const LEGACY_DAG: &str = r#"
task-001:
  title: One
  complexity: LOW
"#;

    fn strict() -> HandoffPolicy {
        HandoffPolicy {
            strict: true,
            require_attested_revision: false,
            require_contracts: true,
            required_kind: HandoffKind::VerifiedDeliverable,
        }
    }

    fn dag_rel() -> SafeRelativePath {
        SafeRelativePath::new(DAG_REL).expect("dag rel")
    }

    fn temp_root() -> (tempfile::TempDir, ProjectRoot) {
        let dir = tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("project root");
        (dir, root)
    }

    fn write_at(root: &ProjectRoot, rel: &str, body: &str) {
        let path = root.as_path().as_std_path().join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(&path, body).expect("write fixture");
    }

    fn contract_yaml(task_id: &str, goal: &str) -> String {
        format!(
            r#"schemaVersion: 1
taskId: {task_id}
goal: "{goal}"
scope:
  allowedPaths:
    - "crates/dare-cli/src/**"
acceptance:
  - id: A-01
    assertion: "It holds"
verification:
  requiredGates: ["build", "test"]
exitCriteria:
  kind: verified-deliverable
startingContext:
  source: project-revision
  repositories:
    - alias: dare-cli
"#
        )
    }

    fn write_contract(root: &ProjectRoot, task_id: &str, body: &str) {
        let rel = contract_rel(&dag_rel(), task_id).expect("contract rel");
        write_at(root, rel.as_str(), body);
    }

    #[test]
    fn preflight_zero_contracts_v21() {
        let (_dir, root) = temp_root();
        let doc = parse_dag_yaml(V21_DAG).expect("dag parses");
        let err = run_handoff_preflight(&root, &dag_rel(), &doc, &strict())
            .expect_err("no contracts refuses");
        assert_eq!(err.exit_code(), 6, "guard_fail is exit 6: {err:?}");
        assert!(matches!(err, CoreError::GuardFail(_)), "got {err:?}");
        assert!(
            err.message().contains("no task contract found"),
            "got {}",
            err.message()
        );
    }

    #[test]
    fn preflight_zero_contracts_legacy() {
        let (_dir, root) = temp_root();
        let doc = parse_dag_yaml(LEGACY_DAG).expect("legacy dag parses");
        assert!(matches!(doc, DagDocument::Legacy(_)));
        let err = run_handoff_preflight(&root, &dag_rel(), &doc, &strict())
            .expect_err("no contracts refuses");
        assert_eq!(err.exit_code(), 6);
        assert!(err.message().contains("no task contract found"));
    }

    #[test]
    fn preflight_missing_dir_names_it() {
        let (_dir, root) = temp_root();
        let doc = parse_dag_yaml(V21_DAG).expect("dag parses");
        let expected = contract_dir_for_dag(&dag_rel()).expect("dir");
        let err = run_handoff_preflight(&root, &dag_rel(), &doc, &strict())
            .expect_err("missing dir refuses");
        assert!(
            err.message().contains(expected.as_str()),
            "the message must name {}, got {}",
            expected.as_str(),
            err.message()
        );
        assert!(
            err.message().contains("HANDOFF_STRICT_NO_CONTRACTS"),
            "the refusal code belongs in the message: {}",
            err.message()
        );
    }

    #[test]
    fn preflight_passes_with_one_of_three() {
        let (_dir, root) = temp_root();
        let doc = parse_dag_yaml(V21_DAG).expect("dag parses");
        write_contract(&root, "task-001", &contract_yaml("task-001", "Do one"));

        let report = run_handoff_preflight(&root, &dag_rel(), &doc, &strict())
            .expect("one contract is enough");
        assert_eq!(report.schema_version, 1);
        assert!(report.strict);
        assert_eq!(report.tasks_total, 3);
        assert_eq!(report.tasks_with_contract, 1);
        assert!(report.violations.is_empty(), "{:?}", report.violations);
    }

    #[test]
    fn preflight_refuses_blocking_violation() {
        let (_dir, root) = temp_root();
        let doc = parse_dag_yaml(V21_DAG).expect("dag parses");
        // taskId does not match the file stem, which is a blocking violation.
        write_contract(&root, "task-001", &contract_yaml("task-999", "Do one"));

        let err = run_handoff_preflight(&root, &dag_rel(), &doc, &strict())
            .expect_err("a blocking violation refuses");
        assert_eq!(err.exit_code(), 6);
        assert!(
            err.message().contains("blocking contract violation"),
            "got {}",
            err.message()
        );
        assert!(err.message().contains("task-001"), "got {}", err.message());
    }

    #[test]
    fn preflight_reports_violations_capped() {
        let (_dir, root) = temp_root();
        // 12 tasks, each with a contract whose taskId is not in the DAG and
        // does not match its file stem: 2 blocking violations per task.
        let mut yaml = String::from("title: \"T\"\nversion: \"1.0.0\"\ntasks:\n");
        for i in 1..=12 {
            yaml.push_str(&format!(
                "  - id: task-{i:03}\n    title: T{i}\n    depends_on: []\n    complexity: LOW\n    subtask_prompt: p{i}\n"
            ));
        }
        let doc = parse_dag_yaml(&yaml).expect("dag parses");
        for i in 1..=12 {
            write_contract(
                &root,
                &format!("task-{i:03}"),
                &contract_yaml(&format!("other-{i:03}"), "Mismatched"),
            );
        }

        let err = run_handoff_preflight(&root, &dag_rel(), &doc, &strict())
            .expect_err("blocking violations refuse");
        let msg = err.message();
        assert!(msg.contains("and "), "the tail must be summarised: {msg}");
        assert!(msg.contains("more"), "the tail must be summarised: {msg}");
        let listed = msg.matches("task-").count();
        assert!(
            listed <= MAX_LISTED_VIOLATIONS + 1,
            "at most {MAX_LISTED_VIOLATIONS} are named, got {listed}: {msg}"
        );
    }

    #[test]
    fn preflight_tolerates_advisory_violation() {
        let (_dir, root) = temp_root();
        let doc = parse_dag_yaml(V21_DAG).expect("dag parses");
        // `**` alone is an over-broad scope: advisory, not blocking.
        let body = contract_yaml("task-001", "Do one")
            .replace("    - \"crates/dare-cli/src/**\"", "    - \"**\"");
        write_contract(&root, "task-001", &body);

        let report = run_handoff_preflight(&root, &dag_rel(), &doc, &strict())
            .expect("an advisory violation does not refuse");
        assert_eq!(report.tasks_with_contract, 1);
        assert!(
            report
                .violations
                .iter()
                .any(|(_, v)| v.severity() == ViolationSeverity::Warning),
            "the advisory violation is still reported: {:?}",
            report.violations
        );
    }
}
