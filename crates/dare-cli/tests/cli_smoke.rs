use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;

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
        .stdout(predicate::str::contains("--version"));
}

#[test]
fn help_exit_zero() {
    Command::new(cargo_bin("dare"))
        .arg("--help")
        .assert()
        .code(0);
}

#[test]
fn unknown_flag_fails() {
    Command::new(cargo_bin("dare"))
        .arg("--not-a-real-flag")
        .assert()
        .failure();
}
