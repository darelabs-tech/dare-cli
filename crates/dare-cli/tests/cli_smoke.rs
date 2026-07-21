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
