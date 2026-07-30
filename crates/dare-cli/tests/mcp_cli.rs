//! CLI integration: `dare mcp serve` (mp052-004).

use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_lists_mcp() {
    Command::new(cargo_bin("dare"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("mcp"));
}

#[test]
fn bad_transport_exit_2() {
    Command::new(cargo_bin("dare"))
        .args(["mcp", "serve", "--transport", "sse", "--no-color"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unknown transport"));
}
