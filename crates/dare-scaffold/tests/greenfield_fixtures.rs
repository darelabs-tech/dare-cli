//! Greenfield integration fixtures (mp046-005).

use dare_core::{CoreError, ProjectRoot, SafeRelativePath};
use dare_scaffold::{
    run_scaffold, validate_stack_output, ScaffoldRequest, Toolchain, MSG_UNKNOWN_STACK,
};
use tempfile::tempdir;

fn scaffold_request(stack_id: &str) -> ScaffoldRequest {
    ScaffoldRequest {
        project_name: "demo-app".to_string(),
        stack_id: stack_id.to_string(),
        toolchain: Toolchain::None,
        transport: None,
        frontend: None,
        conflict_policy: dare_scaffold::ConflictPolicy::FailFast,
        force: false,
        check: false,
    }
}

fn assert_file_exists(root: &ProjectRoot, rel: &str) {
    let path = SafeRelativePath::new(rel).expect("safe path");
    let abs = root.resolve(&path).expect("resolve");
    assert!(
        abs.as_path().as_std_path().exists(),
        "expected file `{rel}` to exist"
    );
}

#[test]
fn greenfield_node_nestjs() {
    let dir = tempdir().expect("tempdir");
    let root = ProjectRoot::new(dir.path()).expect("project root");

    let report = run_scaffold(&root, &scaffold_request("node-nestjs")).expect("scaffold");
    assert!(!report.rolled_back);
    assert!(!report.check);
    assert!(!report.created.is_empty());

    let validation = validate_stack_output(&root, "node-nestjs").expect("validate");
    assert!(validation.ok);
    assert!(validation.missing.is_empty());
    assert!(validation.secret_hits.is_empty());

    assert_file_exists(&root, "dare.config.json");
    assert_file_exists(&root, "llms.txt");
    assert_file_exists(&root, "openapi.json");
}

#[test]
fn greenfield_rust_axum() {
    let dir = tempdir().expect("tempdir");
    let root = ProjectRoot::new(dir.path()).expect("project root");

    let report = run_scaffold(&root, &scaffold_request("rust-axum")).expect("scaffold");
    assert!(!report.rolled_back);
    assert!(!report.check);
    assert!(!report.created.is_empty());

    let validation = validate_stack_output(&root, "rust-axum").expect("validate");
    assert!(validation.ok);
    assert!(validation.missing.is_empty());
    assert!(validation.secret_hits.is_empty());

    assert_file_exists(&root, "dare.config.json");
    assert_file_exists(&root, "llms.txt");
    assert_file_exists(&root, "openapi.json");
}

#[test]
fn greenfield_mcp_node_ts() {
    let dir = tempdir().expect("tempdir");
    let root = ProjectRoot::new(dir.path()).expect("project root");

    let report = run_scaffold(&root, &scaffold_request("mcp-node-ts")).expect("scaffold");
    assert!(!report.rolled_back);
    assert!(!report.check);
    assert!(!report.created.is_empty());

    let validation = validate_stack_output(&root, "mcp-node-ts").expect("validate");
    assert!(validation.ok);
    assert!(validation.missing.is_empty());
    assert!(validation.secret_hits.is_empty());

    assert_file_exists(&root, "dare.config.json");
    assert_file_exists(&root, "llms.txt");
    assert_file_exists(&root, "openapi.json");
}

#[test]
fn validate_unknown_stack_integration() {
    let dir = tempdir().expect("tempdir");
    let root = ProjectRoot::new(dir.path()).expect("project root");

    let err = validate_stack_output(&root, "not-a-real-stack").expect_err("unknown stack");
    match err {
        CoreError::InvalidInput(msg) => {
            assert!(msg.contains(MSG_UNKNOWN_STACK));
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}
