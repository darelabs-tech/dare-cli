//! CLI integration: `dare init` / `dare bootstrap` (mp047-006, BLUEPRINT-047 Fase F).
//!
//! Golden path lists live in `fixtures/golden/init/` (3/11 stacks). The remaining 8
//! stack ids (`go-gin`, `go-stdlib`, `mcp-go`, `mcp-python`, `mcp-rust`, `php-laravel`,
//! `python-fastapi`, `ruby-rails-8`) are deferred to closeout / full CI matrix (Fase G).

use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn golden_init_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/golden/init")
}

fn load_golden_paths(stack_id: &str) -> Vec<String> {
    let path = golden_init_dir().join(format!("{stack_id}.paths.txt"));
    let raw = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("read golden paths `{}`: {e}", path.display())
    });
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn assert_paths_sorted(paths: &[String]) {
    let mut sorted = paths.to_vec();
    sorted.sort();
    assert_eq!(paths, sorted, "golden paths must be ASC");
}

fn assert_golden_paths_exist(project_root: &Path, stack_id: &str) {
    let paths = load_golden_paths(stack_id);
    assert!(!paths.is_empty(), "golden list for `{stack_id}` is empty");
    assert_paths_sorted(&paths);
    for rel in &paths {
        let abs = project_root.join(rel);
        assert!(abs.is_file(), "missing golden path `{rel}` under {}", project_root.display());
    }
}

fn parse_json_envelope(output: &[u8]) -> Value {
    let out = String::from_utf8_lossy(output);
    serde_json::from_str(out.trim()).unwrap_or_else(|e| panic!("json envelope: {e}; out={out}"))
}

#[test]
fn golden_path_lists_sorted_and_non_empty() {
    for stack in ["rust-axum", "node-nestjs", "mcp-node-ts"] {
        let paths = load_golden_paths(stack);
        assert!(!paths.is_empty(), "golden list for `{stack}` is empty");
        assert_paths_sorted(&paths);
    }
}

#[test]
fn help_mentions_init_and_bootstrap() {
    Command::new(cargo_bin("dare"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("bootstrap"));
}

#[test]
fn init_noninteractive_json_rust_axum_golden_paths() {
    let parent = tempfile::tempdir().expect("tempdir");
    let parent_str = parent.path().to_str().expect("utf8 path");

    let assert = Command::new(cargo_bin("dare"))
        .args([
            "--json",
            "init",
            "demo-app",
            "--stack",
            "rust-axum",
            "--non-interactive",
            "-d",
            parent_str,
            "--no-color",
        ])
        .assert()
        .success();

    let v = parse_json_envelope(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    let data = &v["data"];
    assert_eq!(data["schemaVersion"], 1);
    assert_eq!(data["mode"], "init");
    assert_eq!(data["stackId"], "rust-axum");
    assert_eq!(data["projectName"], "demo-app");
    assert_eq!(data["check"], false);

    let project_root = parent.path().join("demo-app");
    assert_golden_paths_exist(&project_root, "rust-axum");
}

#[test]
fn init_noninteractive_mcp_node_ts_golden_paths() {
    let parent = tempfile::tempdir().expect("tempdir");
    let parent_str = parent.path().to_str().expect("utf8 path");

    let assert = Command::new(cargo_bin("dare"))
        .args([
            "--json",
            "init",
            "mcp-demo",
            "--mcp",
            "ts",
            "--non-interactive",
            "-d",
            parent_str,
            "--no-color",
        ])
        .assert()
        .success();

    let v = parse_json_envelope(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["schemaVersion"], 1);
    assert_eq!(v["data"]["stackId"], "mcp-node-ts");

    assert_golden_paths_exist(&parent.path().join("mcp-demo"), "mcp-node-ts");
}

#[test]
fn init_fullstack_react_creates_frontend_package_json() {
    let parent = tempfile::tempdir().expect("tempdir");
    let parent_str = parent.path().to_str().expect("utf8 path");

    Command::new(cargo_bin("dare"))
        .args([
            "init",
            "fs-demo",
            "--stack",
            "rust-axum",
            "--fullstack",
            "react",
            "--non-interactive",
            "-d",
            parent_str,
            "--no-color",
        ])
        .assert()
        .success();

    let frontend_pkg = parent.path().join("fs-demo/frontend/package.json");
    assert!(
        frontend_pkg.is_file(),
        "expected `{}` after fullstack react init",
        frontend_pkg.display()
    );
}

#[test]
fn bootstrap_idempotent_after_init() {
    let parent = tempfile::tempdir().expect("tempdir");
    let parent_str = parent.path().to_str().expect("utf8 path");
    let project = parent.path().join("boot-demo");
    let project_str = project.to_str().expect("utf8 project path");

    Command::new(cargo_bin("dare"))
        .args([
            "init",
            "boot-demo",
            "--stack",
            "node-nestjs",
            "--non-interactive",
            "-d",
            parent_str,
            "--no-color",
        ])
        .assert()
        .success();

    assert_golden_paths_exist(&project, "node-nestjs");

    for run in 1..=2 {
        let assert = Command::new(cargo_bin("dare"))
            .args(["--json", "bootstrap", "-d", project_str, "--no-color"])
            .assert()
            .success();
        let v = parse_json_envelope(&assert.get_output().stdout);
        assert_eq!(v["ok"], true, "bootstrap run {run}");
        let data = &v["data"];
        assert_eq!(data["schemaVersion"], 1);
        assert_eq!(data["mode"], "bootstrap");
        assert_eq!(data["stackId"], "node-nestjs");
        assert!(
            data["created"].as_array().expect("created").is_empty(),
            "bootstrap run {run} must not create files (idempotent): {:?}",
            data["created"]
        );
    }
}
