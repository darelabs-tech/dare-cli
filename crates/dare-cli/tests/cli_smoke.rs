use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

#[test]
fn version_prints_semver() {
    Command::new(cargo_bin("dare"))
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^dare 0\.1\.0-alpha\.0\s*$").unwrap());
}

#[test]
fn help_mentions_version_flag() {
    Command::new(cargo_bin("dare"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--version"))
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--no-color"));
}

#[test]
fn help_exit_zero() {
    Command::new(cargo_bin("dare"))
        .arg("--help")
        .assert()
        .code(0);
}

#[test]
fn cli_unknown_flag_exit_2_human() {
    Command::new(cargo_bin("dare"))
        .arg("--not-a-real-flag")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error:"));
}

#[test]
fn cli_unknown_flag_json_stdout_no_ansi() {
    let assert = Command::new(cargo_bin("dare"))
        .args(["--json", "--not-a-real-flag"])
        .assert()
        .code(2);
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !out.contains('\u{1b}'),
        "ANSI leaked into JSON stdout: {out}"
    );
    let v: Value = serde_json::from_str(out.trim()).expect("json");
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["kind"], "Usage");
    assert!(v.get("correlation_id").is_some());
    // lexicographic: correlation_id, error, ok
    let keys: Vec<_> = v.as_object().unwrap().keys().cloned().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
}

#[test]
fn welcome_no_banner_no_dare_new() {
    Command::new(cargo_bin("dare"))
        .args(["welcome", "--no-banner", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Quick start"))
        .stdout(predicate::str::contains("dare design"))
        .stdout(predicate::str::contains("dare new").not());
}

#[test]
fn welcome_env_no_banner() {
    Command::new(cargo_bin("dare"))
        .env("DARE_NO_BANNER", "1")
        .args(["welcome", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("____").not())
        .stdout(predicate::str::contains("Quick start"));
}

#[test]
fn assets_verify_ok() {
    Command::new(cargo_bin("dare"))
        .args(["assets", "verify"])
        .assert()
        .success()
        .stdout(predicate::str::contains("assets verify: ok"));
}

#[test]
fn capabilities_validate_ok() {
    Command::new(cargo_bin("dare"))
        .args(["capabilities", "validate"])
        .assert()
        .success()
        .stdout(predicate::str::contains("capabilities validate: ok"))
        .stdout(predicate::str::contains("49"));
}

#[test]
fn harness_claude_install_validate_detect() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_str().expect("utf8 path");

    Command::new(cargo_bin("dare"))
        .args(["harness", "claude", "install", "--force", "--root", root])
        .assert()
        .success()
        .stdout(predicate::str::contains("harness claude install: wrote 49"));

    Command::new(cargo_bin("dare"))
        .args(["harness", "claude", "validate", "--root", root])
        .assert()
        .success()
        .stdout(predicate::str::contains("harness claude validate: ok"))
        .stdout(predicate::str::contains("49"));

    Command::new(cargo_bin("dare"))
        .args(["harness", "claude", "detect", "--root", root])
        .assert()
        .success()
        .stdout(predicate::str::contains("claude_md=true"))
        .stdout(predicate::str::contains("claude_dir=true"));
}

#[test]
fn harness_cursor_install_validate_detect() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_str().expect("utf8 path");

    Command::new(cargo_bin("dare"))
        .args(["harness", "cursor", "install", "--force", "--root", root])
        .assert()
        .success()
        .stdout(predicate::str::contains("harness cursor install: wrote 49"));

    Command::new(cargo_bin("dare"))
        .args(["harness", "cursor", "validate", "--root", root])
        .assert()
        .success()
        .stdout(predicate::str::contains("harness cursor validate: ok"))
        .stdout(predicate::str::contains("49"));

    Command::new(cargo_bin("dare"))
        .args(["harness", "cursor", "detect", "--root", root])
        .assert()
        .success()
        .stdout(predicate::str::contains("cursor_dir=true"))
        .stdout(predicate::str::contains("cursorrules=true"));
}

#[test]
fn harness_codex_install_validate_detect() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_str().expect("utf8 path");

    Command::new(cargo_bin("dare"))
        .args(["harness", "codex", "install", "--force", "--root", root])
        .assert()
        .success()
        .stdout(predicate::str::contains("harness codex install: wrote 49"));

    Command::new(cargo_bin("dare"))
        .args(["harness", "codex", "validate", "--root", root])
        .assert()
        .success()
        .stdout(predicate::str::contains("harness codex validate: ok"))
        .stdout(predicate::str::contains("49"));

    Command::new(cargo_bin("dare"))
        .args(["harness", "codex", "detect", "--root", root])
        .assert()
        .success()
        .stdout(predicate::str::contains("agents_md=true"));
}

#[test]
fn harness_antigravity_install_validate_detect() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().to_str().expect("utf8 path");

    Command::new(cargo_bin("dare"))
        .args([
            "harness",
            "antigravity",
            "install",
            "--force",
            "--root",
            root,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "harness antigravity install: wrote 49",
        ));

    Command::new(cargo_bin("dare"))
        .args(["harness", "antigravity", "validate", "--root", root])
        .assert()
        .success()
        .stdout(predicate::str::contains("harness antigravity validate: ok"))
        .stdout(predicate::str::contains("49"));

    Command::new(cargo_bin("dare"))
        .args(["harness", "antigravity", "detect", "--root", root])
        .assert()
        .success()
        .stdout(predicate::str::contains("rules=true"));
}

#[test]
fn info_human_tempdir() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_str().expect("utf8");
    Command::new(cargo_bin("dare"))
        .args(["info", "--root", root, "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("read-only"))
        .stdout(predicate::str::contains("version:"));
}

#[test]
fn info_json_schema() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_str().expect("utf8");
    let assert = Command::new(cargo_bin("dare"))
        .args(["info", "--json", "--root", root, "--no-color"])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(out.trim()).expect("json envelope");
    assert_eq!(v["ok"], true);
    let data = &v["data"];
    assert_eq!(data["schemaVersion"], 1);
    assert!(data["assetsOk"].is_boolean());
    assert!(data["version"].as_str().unwrap_or("").contains('.'));
}

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

#[test]
fn discover_check_human_node() {
    let node = fixture("existing-node-project");
    let path = node.to_str().expect("utf8");
    Command::new(cargo_bin("dare"))
        .args(["discover", "--check", "-d", path, "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("check (zero mutations)"))
        .stdout(predicate::str::contains("node"));
}

#[test]
fn discover_check_json_schema() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("package.json"), "{}").unwrap();
    let path = dir.path().to_str().expect("utf8");
    let assert = Command::new(cargo_bin("dare"))
        .args(["discover", "--check", "--json", "-d", path, "--no-color"])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(out.trim()).expect("json envelope");
    assert_eq!(v["ok"], true);
    let data = &v["data"];
    assert_eq!(data["schemaVersion"], 1);
    assert_eq!(data["mode"], "check");
}

#[test]
fn discover_dir_missing_exits_3() {
    Command::new(cargo_bin("dare"))
        .args([
            "discover",
            "--check",
            "-d",
            "__dare_missing_dir_9f3a2b__",
            "--no-color",
        ])
        .assert()
        .failure()
        .code(3);
}

#[test]
fn discover_install_node_fixture() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("package.json"), "{}").unwrap();
    let path = dir.path().to_str().expect("utf8");
    Command::new(cargo_bin("dare"))
        .args(["discover", "-d", path, "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode: install"));
    assert!(dir.path().join("dare.config.json").is_file());
    assert!(dir.path().join("DARE").join("README.md").is_file());
}

#[test]
fn discover_install_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("package.json"), "{}").unwrap();
    let path = dir.path().to_str().expect("utf8");
    Command::new(cargo_bin("dare"))
        .args(["discover", "-d", path, "--no-color"])
        .assert()
        .success();
    Command::new(cargo_bin("dare"))
        .args(["discover", "-d", path, "--no-color"])
        .assert()
        .success();
}

#[test]
fn discover_check_still_read_only() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("package.json"), "{}").unwrap();
    let before: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .map(|e| e.file_name())
        .collect();
    let path = dir.path().to_str().expect("utf8");
    Command::new(cargo_bin("dare"))
        .args(["discover", "--check", "-d", path, "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("zero mutations"));
    let after: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .flatten()
        .map(|e| e.file_name())
        .collect();
    assert_eq!(before, after);
}

#[test]
fn discover_strict_conflicts_exits_4() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("package.json"), "{}").unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname=\"x\"\nversion=\"0.0.0\"\n",
    )
    .unwrap();
    let path = dir.path().to_str().expect("utf8");
    Command::new(cargo_bin("dare"))
        .args(["discover", "--strict-conflicts", "-d", path, "--no-color"])
        .assert()
        .failure()
        .code(4);
    assert!(!dir.path().join("dare.config.json").exists());
}

#[test]
fn discover_dry_run_no_writes() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("package.json"), "{}").unwrap();
    let path = dir.path().to_str().expect("utf8");
    Command::new(cargo_bin("dare"))
        .args(["discover", "--dry-run", "-d", path, "--no-color"])
        .assert()
        .success();
    assert!(!dir.path().join("dare.config.json").exists());
}

#[test]
fn discover_install_json_schema() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("package.json"), "{}").unwrap();
    let path = dir.path().to_str().expect("utf8");
    let assert = Command::new(cargo_bin("dare"))
        .args(["discover", "--json", "-d", path, "--no-color"])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(out.trim()).expect("json envelope");
    assert_eq!(v["ok"], true);
    let data = &v["data"];
    assert_eq!(data["schemaVersion"], 1);
    assert_eq!(data["mode"], "install");
}
