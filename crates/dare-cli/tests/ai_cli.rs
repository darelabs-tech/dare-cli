//! CLI integration: `dare ai` (mp050-005).

use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;

fn write_file(root: &Path, rel: &str, contents: &str) {
    let path = root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(contents.as_bytes()).unwrap();
}

fn parse_json_envelope(output: &[u8]) -> Value {
    let out = String::from_utf8_lossy(output);
    serde_json::from_str(out.trim()).unwrap_or_else(|e| panic!("json envelope: {e}; out={out}"))
}

fn design_markdown() -> String {
    let begin = |id: &str| format!("<!-- AGENT:BEGIN section=\"{id}\" -->");
    let end = |id: &str| format!("<!-- AGENT:END section=\"{id}\" -->");
    format!(
        "# Design\n\n\
         Unmanaged paragraph must survive.\n\n\
         {d0}\nold description\n{d1}\n\n\
         {o0}\nold objectives\n{o1}\n\n\
         {f0}\nold fr\n{f1}\n\n\
         {s0}\nold stack\n{s1}\n",
        d0 = begin("description"),
        d1 = end("description"),
        o0 = begin("objectives"),
        o1 = end("objectives"),
        f0 = begin("functional-requirements"),
        f1 = end("functional-requirements"),
        s0 = begin("stack"),
        s1 = end("stack"),
    )
}

#[test]
fn help_mentions_ai() {
    Command::new(cargo_bin("dare"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("ai"));
}

#[test]
fn doctor_mock_ready() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dir_str = dir.path().to_str().expect("utf8 path");

    let assert = Command::new(cargo_bin("dare"))
        .args([
            "ai",
            "doctor",
            "--provider",
            "mock",
            "--json",
            "-d",
            dir_str,
            "--no-color",
        ])
        .assert()
        .success();

    let v = parse_json_envelope(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    let data = &v["data"];
    assert_eq!(data["schemaVersion"], 1);
    let providers = data["providers"].as_array().expect("providers");
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0]["id"], "mock");
    assert_eq!(providers[0]["status"], "ready");
    assert_eq!(providers[0]["implemented"], true);
}

#[test]
fn providers_json_schema() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dir_str = dir.path().to_str().expect("utf8 path");

    let assert = Command::new(cargo_bin("dare"))
        .args(["ai", "providers", "--json", "-d", dir_str, "--no-color"])
        .assert()
        .success();

    let v = parse_json_envelope(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    let data = &v["data"];
    assert_eq!(data["schemaVersion"], 1);
    let providers = data["providers"].as_array().expect("providers");
    assert_eq!(providers.len(), 5);
    let ids: Vec<&str> = providers
        .iter()
        .filter_map(|p| p["id"].as_str())
        .collect();
    assert_eq!(
        ids,
        vec![
            "mock",
            "codex",
            "claude-code",
            "cursor-cli",
            "antigravity-cli"
        ]
    );
}

#[test]
fn prompt_no_env_leak() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    write_file(root, "DARE/DESIGN.md", &design_markdown());
    let dir_str = root.to_str().expect("utf8 path");
    let secret = "codex-exec-SECRET-mp050-005-do-not-leak";

    let assert = Command::new(cargo_bin("dare"))
        .env("DARE_CODEX_COMMAND", secret)
        .args([
            "ai",
            "prompt",
            "--command",
            "design",
            "--provider",
            "mock",
            "--markdown",
            "DARE/DESIGN.md",
            "--json",
            "-d",
            dir_str,
            "--no-color",
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        !stdout.contains(secret),
        "stdout must not leak env override: {stdout}"
    );
    assert!(
        !stderr.contains(secret),
        "stderr must not leak env override: {stderr}"
    );

    let v = parse_json_envelope(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["envLeaked"], false);
    assert_eq!(v["data"]["schemaVersion"], 1);
}

#[test]
fn run_mock_ok() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    write_file(root, "DARE/DESIGN.md", &design_markdown());
    let dir_str = root.to_str().expect("utf8 path");

    let assert = Command::new(cargo_bin("dare"))
        .env_remove("DARE_AI_MOCK_MODE")
        .args([
            "ai",
            "run",
            "--command",
            "design",
            "--provider",
            "mock",
            "--markdown",
            "DARE/DESIGN.md",
            "--json",
            "-d",
            dir_str,
            "--no-color",
        ])
        .assert()
        .success();

    let v = parse_json_envelope(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    let data = &v["data"];
    assert_eq!(data["schemaVersion"], 1);
    assert_eq!(data["ok"], true);
    assert_eq!(data["enriched"], true);
    assert_eq!(data["provider"], "mock");
    assert_eq!(data["command"], "design");
    assert_eq!(data["written"], false);
}

#[test]
fn unknown_provider() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dir_str = dir.path().to_str().expect("utf8 path");

    Command::new(cargo_bin("dare"))
        .args([
            "ai",
            "doctor",
            "--provider",
            "no-such-provider",
            "-d",
            dir_str,
            "--no-color",
        ])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("unknown provider"));
}

#[test]
fn missing_facts_exit_3() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dir_str = dir.path().to_str().expect("utf8 path");

    Command::new(cargo_bin("dare"))
        .args([
            "ai",
            "run",
            "--command",
            "design",
            "--provider",
            "mock",
            "--facts",
            "missing/facts.json",
            "-d",
            dir_str,
            "--no-color",
        ])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("file not found"));
}

#[test]
fn malformed_exit_4() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    write_file(root, "DARE/DESIGN.md", &design_markdown());
    let dir_str = root.to_str().expect("utf8 path");

    Command::new(cargo_bin("dare"))
        .env("DARE_AI_MOCK_MODE", "invalid-json")
        .args([
            "ai",
            "run",
            "--command",
            "design",
            "--provider",
            "mock",
            "--markdown",
            "DARE/DESIGN.md",
            "-d",
            dir_str,
            "--no-color",
        ])
        .assert()
        .code(4);
}
