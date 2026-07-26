//! ConflictPolicy::SkipExisting integration tests (mp047-002).

use dare_core::{ProjectRoot, SafeRelativePath};
use dare_scaffold::{apply_scaffold, plan_scaffold, ConflictPolicy, PlanAction, ScaffoldRequest, Toolchain};
use std::fs;
use tempfile::tempdir;

fn sample_req(stack_id: &str) -> ScaffoldRequest {
    ScaffoldRequest {
        project_name: "demo-app".to_string(),
        stack_id: stack_id.to_string(),
        toolchain: Toolchain::None,
        transport: None,
        frontend: None,
        conflict_policy: ConflictPolicy::SkipExisting,
        force: false,
        check: false,
    }
}

#[test]
fn conflict_skip_existing() {
    let dir = tempdir().expect("tempdir");
    let root = ProjectRoot::new(dir.path()).expect("project root");

    let rel = SafeRelativePath::new("dare.config.json").expect("safe path");
    dare_core::fs::atomic_write(&root, &rel, br#"{"seed":true}"#).expect("seed file");

    let plan = plan_scaffold(&root, &sample_req("go-gin")).expect("plan with skip");
    let item = plan
        .items
        .iter()
        .find(|i| i.path == "dare.config.json")
        .expect("dare.config.json in plan");
    assert_eq!(item.action, PlanAction::Skip);

    let report = apply_scaffold(&root, &plan).expect("apply with skip");
    assert!(report.skipped.contains(&"dare.config.json".to_string()));
    assert!(!report.created.contains(&"dare.config.json".to_string()));
    assert!(!report.replaced.contains(&"dare.config.json".to_string()));

    let content = fs::read_to_string(dir.path().join("dare.config.json")).expect("read seed");
    assert!(content.contains("\"seed\":true"));
}
