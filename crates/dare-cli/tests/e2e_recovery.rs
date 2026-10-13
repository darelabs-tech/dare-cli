//! End-to-end recovery acceptance through separate CLI processes.
use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use serde_json::Value;

fn run(root: &std::path::Path, args: &[&str], expected_code: i32) -> Value {
    let output = Command::new(cargo_bin("dare"))
        .current_dir(root)
        .args(args)
        .assert()
        .code(expected_code)
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf8 response");
    serde_json::from_str(stdout.trim()).expect("JSON CLI envelope")
}

fn state(root: &std::path::Path) -> Value {
    let raw = std::fs::read_to_string(root.join(".dare/state.json")).expect("persisted state");
    serde_json::from_str(&raw).expect("valid state")
}

#[test]
fn interrupted_execution_resumes_and_exports_safe_diagnostics() {
    let dir = tempfile::tempdir().expect("project");
    let root = dir.path();
    std::fs::create_dir_all(root.join("DARE/EXECUTION")).unwrap();
    std::fs::write(root.join("dare.config.json"), r#"{"backend":"rust-axum"}"#).unwrap();
    std::fs::write(
        root.join("DARE/dare-dag.yaml"),
        include_str!("../../../tests/fixtures/dag/valid.v21.yaml"),
    )
    .unwrap();
    std::fs::write(root.join("DARE/EXECUTION/task-001.md"), "# task").unwrap();

    let valid = run(root, &["--json", "validate"], 0);
    assert_eq!(valid["ok"], true);
    let ready = run(root, &["--json", "execute", "--next"], 0);
    assert_eq!(ready["ok"], true);
    assert_eq!(ready["data"]["ready"][0]["id"], "task-001");

    // An interrupted run exits 124 and persists the RUNNING task.
    Command::new(cargo_bin("dare"))
        .current_dir(root)
        .env("DARE_RALPH_MOCK", "timeout")
        .args(["--json", "execute", "--complete", "task-001"])
        .assert()
        .code(124);
    assert_eq!(state(root)["tasks"]["task-001"]["status"], "RUNNING");

    // Each invocation starts a fresh OS process: state must survive restarts.
    let persisted = run(root, &["--json", "execute", "--status"], 0);
    assert_eq!(persisted["data"]["counts"]["running"], 1);

    let reset = run(root, &["--json", "execute", "--reset", "task-001"], 0);
    assert_eq!(reset["ok"], true);
    assert_eq!(state(root)["tasks"]["task-001"]["status"], "PENDING");

    Command::new(cargo_bin("dare"))
        .current_dir(root)
        .env("DARE_RALPH_MOCK", "pass")
        .args(["--json", "execute", "--complete", "task-001"])
        .assert()
        .success();
    assert_eq!(state(root)["tasks"]["task-001"]["status"], "DONE");
    assert!(root.join(".dare/verification/task-001.json").is_file());

    let resolved = run(root, &["--json", "execute", "--next"], 0);
    assert_eq!(resolved["data"]["resolved"], true);
    let diagnostic = run(root, &["--json", "info"], 0);
    assert_eq!(diagnostic["ok"], true);
    let preview = run(root, &["--json", "support"], 0);
    assert_eq!(preview["data"]["included"][0], "diagnostics.json");

    // Explicitly export outside the project. No user config or state is copied.
    let out = tempfile::tempdir().unwrap();
    let exported = run(
        root,
        &[
            "--json",
            "support",
            "--out-dir",
            out.path().to_str().unwrap(),
        ],
        0,
    );
    assert_eq!(exported["data"]["written"], true);
    let report = std::fs::read_to_string(out.path().join("dare-support.json")).unwrap();
    assert!(!report.contains("rust-axum"));
    assert!(!report.contains("task-001"));
}
