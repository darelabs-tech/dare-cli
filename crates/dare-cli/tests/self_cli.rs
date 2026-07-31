//! CLI integration: `dare self` (mp053-005).

use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

#[test]
fn help_lists_self() {
    Command::new(cargo_bin("dare"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("self"));

    Command::new(cargo_bin("dare"))
        .args(["self", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("project assets"));
}

#[test]
fn bad_channel_exit_2() {
    Command::new(cargo_bin("dare"))
        .args([
            "self",
            "update",
            "--channel",
            "nightly",
            "--dry-run",
            "--no-color",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unknown channel"));
}

#[test]
fn dry_run_ok() {
    // `--version` plans offline (no GitHub); channel resolve uses Releases API (see dare-self resolve).
    let assert = Command::new(cargo_bin("dare"))
        .args([
            "self",
            "update",
            "--version",
            "v0.1.0-alpha.2",
            "--dry-run",
            "--json",
            "--no-color",
        ])
        .assert()
        .success();

    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(out.trim()).expect("json envelope");
    assert_eq!(v["ok"], true);
    let data = &v["data"];
    assert_eq!(data["schemaVersion"], 1);
    assert_eq!(data["mode"], "update");
    assert_eq!(data["targetTag"], "v0.1.0-alpha.2");
    assert!(data["assetName"].as_str().unwrap().contains("dare-v0.1.0-alpha.2"));
    assert!(data["actions"].as_array().unwrap().len() >= 5);
}

#[test]
fn channel_and_version_exit_2() {
    Command::new(cargo_bin("dare"))
        .args([
            "self",
            "update",
            "--channel",
            "beta",
            "--version",
            "v0.1.0",
            "--dry-run",
            "--no-color",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("not both"));
}
