//! CLI integration: `dare steering list|show` (mp048-006).
//! Hooks CLI smokes land in mp048-005 — leave room; do not stub hooks here.

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

#[test]
fn help_mentions_steering() {
    Command::new(cargo_bin("dare"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("steering"));
}

#[test]
fn steering_list_json() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(dir.path(), "DARE/PROJECT-DNA.md", "# DNA\nconventions\n");
    let dir_str = dir.path().to_str().expect("utf8 path");

    let assert = Command::new(cargo_bin("dare"))
        .args([
            "steering",
            "list",
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
    let files = data["files"].as_array().expect("files array");
    assert!(!files.is_empty());
    assert_eq!(files[0]["path"], "DARE/PROJECT-DNA.md");
}

#[test]
fn steering_show_env_excluded() {
    let dir = tempfile::tempdir().expect("tempdir");
    let secret = "SUPER_SECRET_TOKEN_mp048_006_do_not_leak";
    write_file(dir.path(), ".env", &format!("SECRET={secret}\n"));
    write_file(dir.path(), "DARE/PROJECT-DNA.md", "# DNA\n");
    let dir_str = dir.path().to_str().expect("utf8 path");

    let assert = Command::new(cargo_bin("dare"))
        .args(["steering", "show", ".env", "-d", dir_str, "--no-color"])
        .assert()
        .code(4);

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(
        !stdout.contains(secret),
        "stdout must not contain secret: {stdout}"
    );
    assert!(
        !stderr.contains(secret),
        "stderr must not contain secret: {stderr}"
    );
    assert!(
        stderr.contains("excluded") || stderr.contains(".env"),
        "stderr should mention exclusion: {stderr}"
    );
}

#[test]
fn steering_show_blocks() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_file(dir.path(), "DARE/PROJECT-DNA.md", "# DNA\nbase rules\n");
    write_file(dir.path(), "src/lib.rs", "fn main() {}\n");
    write_file(
        dir.path(),
        ".dare/steering/x.md",
        "---\nscope: glob\nglob: \"src/**/*.rs\"\npriority: 10\n---\nrust steering body\n",
    );
    let dir_str = dir.path().to_str().expect("utf8 path");

    let assert = Command::new(cargo_bin("dare"))
        .args([
            "steering",
            "show",
            "src/lib.rs",
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
    assert_eq!(data["target"], "src/lib.rs");
    let blocks = data["blocks"].as_array().expect("blocks array");
    assert!(
        blocks.len() >= 2,
        "expected DNA + glob block, got {blocks:?}"
    );
    let paths: Vec<&str> = blocks
        .iter()
        .filter_map(|b| b["path"].as_str())
        .collect();
    assert!(paths.contains(&"DARE/PROJECT-DNA.md"));
    assert!(paths.contains(&".dare/steering/x.md"));
}
