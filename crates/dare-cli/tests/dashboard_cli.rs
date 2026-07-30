//! CLI integration: `dare dashboard` / `dare server` (mp051-006) + alias (mp052-006).

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

#[test]
fn mcp_server_alias_prints_deprecation_on_help() {
    Command::new(cargo_bin("dare-mcp-server"))
        .arg("--help")
        .assert()
        .success()
        .stderr(predicate::str::contains("deprecated"))
        .stderr(predicate::str::contains("legacy REST"));
}

#[test]
fn mcp_server_alias_prints_deprecation_before_bind_failure() {
    Command::new(cargo_bin("dare-mcp-server"))
        .args(["--bind", "not-a-valid-ip"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("deprecated"))
        .stderr(predicate::str::contains("legacy REST"));
}
