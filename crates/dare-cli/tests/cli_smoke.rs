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

fn fixture_dag(name: &str) -> String {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../tests/fixtures/dag");
    p.push(name);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

fn validate_project_with_dag(dag_yaml: &str, with_spec: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("dare.config.json"), "{}").unwrap();
    std::fs::create_dir_all(dir.path().join("DARE")).unwrap();
    std::fs::write(dir.path().join("DARE/dare-dag.yaml"), dag_yaml).unwrap();
    if with_spec {
        std::fs::create_dir_all(dir.path().join("DARE/EXECUTION")).unwrap();
        std::fs::write(dir.path().join("DARE/EXECUTION/task-001.md"), "#").unwrap();
    }
    dir
}

#[test]
fn validate_ok_fixture() {
    let dir = validate_project_with_dag(&fixture_dag("valid.v21.yaml"), true);
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["validate", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("validate: ok"));
}

#[test]
fn validate_cycle_exit_1() {
    let dir = validate_project_with_dag(&fixture_dag("cycle.v21.yaml"), false);
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["validate", "--no-color"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("FAILED"));
}

#[test]
fn validate_strict_warning() {
    let dir = validate_project_with_dag(&fixture_dag("warning-missing-spec.v21.yaml"), false);
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["validate", "--strict", "--no-color"])
        .assert()
        .code(1);
}

#[test]
fn validate_warning_without_strict() {
    let dir = validate_project_with_dag(&fixture_dag("warning-missing-spec.v21.yaml"), false);
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["validate", "--no-color"])
        .assert()
        .success();
}

#[test]
fn validate_missing_dag_not_found() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("dare.config.json"), "{}").unwrap();
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["validate", "--no-color"])
        .assert()
        .code(3);
}

#[test]
fn validate_json_schema() {
    let dir = validate_project_with_dag(&fixture_dag("valid.v21.yaml"), true);
    let assert = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["validate", "--json", "--no-color"])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(out.trim()).expect("json");
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["schemaVersion"], 1);
    assert_eq!(v["data"]["mode"], "validate");
}

#[test]
fn validate_json_failure_has_issues() {
    let dir = validate_project_with_dag(&fixture_dag("cycle.v21.yaml"), false);
    let assert = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["validate", "--json", "--no-color"])
        .assert()
        .code(1);
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(out.trim()).expect("json");
    assert_eq!(v["ok"], false);
    assert!(v["data"]["issues"].as_array().unwrap().len() >= 1);
}

#[test]
fn validate_zero_writes() {
    let dir = validate_project_with_dag(&fixture_dag("valid.v21.yaml"), true);
    fn listing(base: &std::path::Path) -> Vec<String> {
        let mut out = Vec::new();
        fn walk(base: &std::path::Path, cur: &std::path::Path, out: &mut Vec<String>) {
            if let Ok(rd) = std::fs::read_dir(cur) {
                for e in rd.flatten() {
                    let p = e.path();
                    let rel = p
                        .strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    out.push(rel);
                    if p.is_dir() {
                        walk(base, &p, out);
                    }
                }
            }
        }
        walk(base, base, &mut out);
        out.sort();
        out
    }
    let before = listing(dir.path());
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["validate", "--no-color"])
        .assert()
        .success();
    let after = listing(dir.path());
    assert_eq!(before, after);
}

fn update_fixture(name: &str) -> std::path::PathBuf {
    fixture(&format!("update/{name}"))
}

fn listing_tree(base: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(base: &std::path::Path, cur: &std::path::Path, out: &mut Vec<String>) {
        if let Ok(rd) = std::fs::read_dir(cur) {
            for e in rd.flatten() {
                let p = e.path();
                let rel = p
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push(rel);
                if p.is_dir() {
                    walk(base, &p, out);
                }
            }
        }
    }
    walk(base, base, &mut out);
    out.sort();
    out
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for e in std::fs::read_dir(src).unwrap() {
        let e = e.unwrap();
        let from = e.path();
        let to = dst.join(e.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

fn update_fixture_temp(name: &str) -> tempfile::TempDir {
    let src = update_fixture(name);
    let dir = tempfile::tempdir().unwrap();
    copy_dir_recursive(&src, dir.path());
    dir
}

fn claude_md_canonical() -> Vec<u8> {
    format!(
        "<!-- dare:managed claude-md -->\n# DARE Framework\n\n\
         Generated by `dare harness claude`. Follow Design → Blueprint → Tasks → Execute.\n\
         Use slash commands from `.claude/commands/`.\n"
    )
    .into_bytes()
}

#[test]
fn update_apply_without_dry_run_ok() {
    let dir = update_fixture_temp("customized-assets");
    let path = dir.path().to_str().expect("utf8");
    Command::new(cargo_bin("dare"))
        .args(["update", "-y", "-d", path, "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode: update"));
}

#[test]
fn update_dry_run_ok() {
    let mixed = update_fixture("mixed");
    let path = mixed.to_str().expect("utf8");
    Command::new(cargo_bin("dare"))
        .args(["update", "--dry-run", "-d", path, "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode: dry-run"));
}

#[test]
fn update_dry_run_json_schema() {
    let mixed = update_fixture("mixed");
    let path = mixed.to_str().expect("utf8");
    let assert = Command::new(cargo_bin("dare"))
        .args(["update", "--dry-run", "--json", "-d", path, "--no-color"])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(out.trim()).expect("json");
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["schemaVersion"], 1);
    assert_eq!(v["data"]["mode"], "dry-run");
}

#[test]
fn update_target_codex() {
    let mixed = update_fixture("mixed");
    let path = mixed.to_str().expect("utf8");
    let assert = Command::new(cargo_bin("dare"))
        .args([
            "update",
            "--dry-run",
            "--target",
            "codex",
            "--json",
            "-d",
            path,
            "--no-color",
        ])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(out.trim()).expect("json");
    let items = v["data"]["items"].as_array().expect("items");
    assert!(!items.is_empty(), "expected at least one codex|* item");
    for item in items {
        let applies = item["appliesTo"].as_array().expect("appliesTo");
        let ok = applies.iter().any(|a| {
            let s = a.as_str().unwrap_or("");
            s == "*" || s == "codex"
        });
        assert!(
            ok,
            "item {} appliesTo must include * or codex: {:?}",
            item["path"], applies
        );
    }
}

#[test]
fn update_invalid_target() {
    let mixed = update_fixture("mixed");
    let path = mixed.to_str().expect("utf8");
    Command::new(cargo_bin("dare"))
        .args([
            "update",
            "--dry-run",
            "--target",
            "3.2.0",
            "-d",
            path,
            "--no-color",
        ])
        .assert()
        .code(4);
}

#[test]
fn update_customized_detected() {
    let customized = update_fixture("customized-assets");
    let path = customized.to_str().expect("utf8");
    let assert = Command::new(cargo_bin("dare"))
        .args(["update", "--dry-run", "--json", "-d", path, "--no-color"])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(out.trim()).expect("json");
    let customized_n = v["data"]["counts"]["customized"]
        .as_u64()
        .expect("customized count");
    assert!(
        customized_n >= 1,
        "expected counts.customized >= 1, got {customized_n}"
    );
}

#[test]
fn update_zero_writes() {
    let mixed = update_fixture("mixed");
    let path = mixed.to_str().expect("utf8");
    let before = listing_tree(&mixed);
    Command::new(cargo_bin("dare"))
        .args(["update", "--dry-run", "-d", path, "--no-color"])
        .assert()
        .success();
    let after = listing_tree(&mixed);
    assert_eq!(before, after, "dry-run must not create or delete files");
}

#[test]
fn update_no_root() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().expect("utf8");
    Command::new(cargo_bin("dare"))
        .args(["update", "--dry-run", "-d", path, "--no-color"])
        .assert()
        .code(4);
}

#[test]
fn update_dry_run_zero_write() {
    let dir = update_fixture_temp("mixed");
    let path = dir.path().to_str().expect("utf8");
    let before = listing_tree(dir.path());
    Command::new(cargo_bin("dare"))
        .args(["update", "--dry-run", "--force", "-d", path, "--no-color"])
        .assert()
        .success();
    let after = listing_tree(dir.path());
    assert_eq!(
        before, after,
        "dry-run must leave listing unchanged (force ignored)"
    );
}

#[test]
fn update_yes_keeps_customized() {
    let dir = update_fixture_temp("customized-assets");
    let path = dir.path().to_str().expect("utf8");
    let claude = dir.path().join("CLAUDE.md");
    let before = std::fs::read(&claude).unwrap();
    let assert = Command::new(cargo_bin("dare"))
        .args(["update", "-y", "--json", "-d", path, "--no-color"])
        .assert()
        .success();
    let after = std::fs::read(&claude).unwrap();
    assert_eq!(before, after, "-y must keep customized CLAUDE.md bytes");
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(out.trim()).expect("json");
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["mode"], "update");
    let warnings = v["data"]["warnings"].as_array().expect("warnings");
    assert!(
        warnings.iter().any(|w| {
            w.as_str()
                .map(|s| s.contains("kept customized") && s.contains("CLAUDE.md"))
                .unwrap_or(false)
        }),
        "expected kept customized warning, got {warnings:?}"
    );
}

#[test]
fn update_force_overwrites_customized() {
    let dir = update_fixture_temp("customized-assets");
    let path = dir.path().to_str().expect("utf8");
    let claude = dir.path().join("CLAUDE.md");
    let before = std::fs::read(&claude).unwrap();
    Command::new(cargo_bin("dare"))
        .args(["update", "--force", "-y", "-d", path, "--no-color"])
        .assert()
        .success();
    let after = std::fs::read(&claude).unwrap();
    assert_ne!(before, after, "--force must change customized bytes");
    assert_eq!(
        after,
        claude_md_canonical(),
        "--force must write canonical CLAUDE.md"
    );
    let dare = dir.path().join(".dare");
    let has_backup = std::fs::read_dir(&dare)
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| e.file_name().to_string_lossy().starts_with("backup-"));
    assert!(has_backup, "expected .dare/backup-* after --force");
}

#[test]
fn update_creates_missing() {
    let dir = update_fixture_temp("missing-assets");
    let path = dir.path().to_str().expect("utf8");
    let agents = dir.path().join("AGENTS.md");
    assert!(!agents.exists(), "fixture must start without AGENTS.md");
    Command::new(cargo_bin("dare"))
        .args(["update", "-y", "-d", path, "--no-color"])
        .assert()
        .success();
    assert!(agents.is_file(), "update -y must create missing AGENTS.md");
}

#[test]
fn update_dir_missing_exit_3() {
    Command::new(cargo_bin("dare"))
        .args([
            "update",
            "-y",
            "-d",
            "__dare_missing_update_dir_9f3a2b__",
            "--no-color",
        ])
        .assert()
        .failure()
        .code(3);
}

fn design_project_temp() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("dare.config.json"), "{}").expect("dare.config.json");
    dir
}

fn design_fixture(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/design")
        .join(name)
}

#[test]
fn design_creates_file() {
    let dir = design_project_temp();
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["design", "hello world", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("design: ok"))
        .stdout(predicate::str::contains("path: DARE/DESIGN.md"));
    let design_path = dir.path().join("DARE/DESIGN.md");
    assert!(design_path.is_file(), "DARE/DESIGN.md must exist");
    let content = std::fs::read_to_string(&design_path).expect("read DESIGN.md");
    assert!(
        content.contains("<!-- AGENT:BEGIN"),
        "DESIGN.md must contain AGENT markers"
    );
}

#[test]
fn design_json_schema() {
    let dir = design_project_temp();
    let assert = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["design", "payment API", "--json", "--no-color"])
        .assert()
        .success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout);
    let v: Value = serde_json::from_str(out.trim()).expect("json envelope");
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["schemaVersion"], 1);
    assert_eq!(v["data"]["mode"], "design");
}

#[test]
fn design_empty_desc_usage_or_4() {
    let dir = design_project_temp();
    let code = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["design", "--no-color"])
        .assert()
        .get_output()
        .status
        .code()
        .expect("exit code");
    assert!(
        code == 2 || code == 4,
        "expected exit 2 or 4 for empty description, got {code}"
    );
}

#[test]
fn design_preserve_notes() {
    let dir = design_project_temp();
    std::fs::create_dir_all(dir.path().join("DARE")).expect("DARE dir");
    std::fs::copy(
        design_fixture("existing-with-notes.md"),
        dir.path().join("DARE/DESIGN.md"),
    )
    .expect("copy fixture");
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["design", "Updated payment API", "--no-color"])
        .assert()
        .success();
    let content = std::fs::read_to_string(dir.path().join("DARE/DESIGN.md")).expect("read");
    assert!(
        content.contains("User note outside any AGENT marker — must survive merge."),
        "unmanaged notes must survive merge"
    );
}

#[test]
fn design_interactive_no_tty_exits_2() {
    let dir = design_project_temp();
    let output = std::process::Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["design", "--interactive", "--no-color"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("spawn dare");
    assert_eq!(
        output.status.code(),
        Some(2),
        "piped stdin must reject --interactive without TTY"
    );
}
