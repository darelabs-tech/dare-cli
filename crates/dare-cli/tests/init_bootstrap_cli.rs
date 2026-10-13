//! CLI integration: `dare init` / `dare bootstrap` (cycle rust-init-npm-parity-011).
//!
//! Byte-level comparison against npm 3.18.1 lives in `init_npm_parity.rs`; this
//! suite covers the CLI contract (flags, JSON envelope, errors, bootstrap).

use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::path::Path;

fn parse_json_envelope(output: &[u8]) -> Value {
    let out = String::from_utf8_lossy(output);
    serde_json::from_str(out.trim()).unwrap_or_else(|e| panic!("json envelope: {e}; out={out}"))
}

fn dare(parent: &Path) -> Command {
    let mut cmd = Command::new(cargo_bin("dare"));
    cmd.current_dir(parent).env("DARE_RAILS_OFFLINE", "1");
    cmd
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
fn init_stack_json_envelope_and_npm_layout() {
    let parent = tempfile::tempdir().expect("tempdir");
    let assert = dare(parent.path())
        .args([
            "--json",
            "--no-color",
            "init",
            "demo-app",
            "--stack",
            "rust-axum",
        ])
        .assert()
        .success();
    let v = parse_json_envelope(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    let data = &v["data"];
    assert_eq!(data["schemaVersion"], 1);
    assert_eq!(data["mode"], "init");
    assert_eq!(data["structure"], "backend");
    assert_eq!(data["backend"], "rust-axum");
    assert_eq!(data["ide"], "cursor");
    assert_eq!(data["toolchain"], "auto");

    let root = parent.path().join("demo-app");
    for rel in [
        "Cargo.toml",
        "src/handlers/auth.rs",
        "migrations/0001_create_users.sql",
        ".cursorrules",
        ".cursor/rules/skill-rust-axum.mdc",
        "DARE/README.md",
        "templates/TASKS-template.md",
        "dare-graph.yml",
        "dare.config.json",
    ] {
        assert!(root.join(rel).is_file(), "missing {rel}");
    }
    assert!(
        !root.join("CLAUDE.md").exists(),
        "--stack defaults to the cursor IDE only"
    );
    assert!(!root.join("Dockerfile").exists());
    let cfg: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("dare.config.json")).unwrap())
            .unwrap();
    assert_eq!(cfg["structure"], "backend");
    assert_eq!(cfg["backend"], "rust-axum");
    assert!(cfg.get("stack").is_none());
}

#[test]
fn init_mcp_and_mvc_flags() {
    let parent = tempfile::tempdir().expect("tempdir");
    dare(parent.path())
        .args(["init", "mcp-demo", "--mcp", "node-ts", "--transport", "sse"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Stack: mcp-node-ts"));
    let cfg: Value = serde_json::from_str(
        &std::fs::read_to_string(parent.path().join("mcp-demo/dare.config.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(cfg["structure"], "mcp-server");
    assert_eq!(cfg["mcpTransport"], "sse");
    assert_eq!(cfg["mcpLanguage"], "node-ts");

    dare(parent.path())
        .args(["init", "shop", "--stack", "ruby-rails-8", "--fullstack"])
        .assert()
        .success();
    assert!(parent
        .path()
        .join("shop/app/views/home/index.html.erb")
        .is_file());
}

#[test]
fn init_errors_match_npm() {
    let parent = tempfile::tempdir().expect("tempdir");
    dare(parent.path())
        .args(["init", "x", "--non-interactive"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--non-interactive requires --stack <id> or --mcp <language>",
        ));
    dare(parent.path())
        .args(["init", "x", "--stack", "node-nestjs", "--fullstack"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--fullstack only applies to MVC frameworks: php-laravel, ruby-rails-8",
        ));
    dare(parent.path())
        .args(["init", "x", "--stack", "rust-axum", "--fullstack", "react"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Monorepo"));
    dare(parent.path())
        .args(["init", "Bad_Name!", "--stack", "go-gin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "project name may only contain lowercase letters",
        ));
}

#[test]
fn init_refuses_non_empty_target() {
    let parent = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(parent.path().join("busy")).unwrap();
    std::fs::write(parent.path().join("busy/keep.txt"), "x").unwrap();
    dare(parent.path())
        .args(["init", "busy", "--stack", "go-gin"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Target directory is not empty"));
    assert!(
        parent.path().join("busy/keep.txt").is_file(),
        "pre-existing files must survive a failed init (B-09)"
    );
}

#[test]
fn bootstrap_reruns_scaffold_after_init() {
    let parent = tempfile::tempdir().expect("tempdir");
    dare(parent.path())
        .args(["init", "boot-demo", "--stack", "node-nestjs"])
        .assert()
        .success();
    let project = parent.path().join("boot-demo");
    std::fs::remove_file(project.join("src/app.module.ts")).unwrap();

    let assert = dare(parent.path())
        .args(["--json", "bootstrap", "-d", project.to_str().unwrap()])
        .assert()
        .success();
    let v = parse_json_envelope(&assert.get_output().stdout);
    assert_eq!(v["data"]["mode"], "bootstrap");
    assert_eq!(v["data"]["backend"], "node-nestjs");
    assert!(project.join("src/app.module.ts").is_file());

    std::fs::create_dir_all(project.join("node_modules")).unwrap();
    dare(parent.path())
        .args(["bootstrap", "-d", project.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Refusing to run"));
}
