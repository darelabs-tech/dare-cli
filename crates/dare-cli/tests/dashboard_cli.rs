//! CLI integration: `dare dashboard` / `dare server` (mp051-006).

use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_lists_dashboard_server() {
    Command::new(cargo_bin("dare"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("dashboard"))
        .stdout(predicate::str::contains("server"));
}

#[test]
fn server_bad_protocol_exit_2() {
    Command::new(cargo_bin("dare"))
        .args(["server", "--protocol", "mcp", "--no-color"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unknown protocol"));
}
