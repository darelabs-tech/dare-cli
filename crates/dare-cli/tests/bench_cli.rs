//! CLI integration: `dare bench` (mp049-007).

use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

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

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

#[test]
fn help_mentions_bench() {
    Command::new(cargo_bin("dare"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("bench"));
}

#[test]
fn suite_invalid_exit_2() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dir_str = dir.path().to_str().expect("utf8 path");

    Command::new(cargo_bin("dare"))
        .args([
            "bench",
            "--suite",
            "no-such-suite",
            "-d",
            dir_str,
            "--no-color",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("invalid bench suite"));
}

#[test]
fn fail_on_regression_needs_baseline() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dir_str = dir.path().to_str().expect("utf8 path");

    Command::new(cargo_bin("dare"))
        .args([
            "bench",
            "--fail-on-regression",
            "3",
            "-d",
            dir_str,
            "--no-color",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("baseline required"));
}

#[test]
fn json_schema_version() {
    let root = workspace_root();
    let root_str = root.to_str().expect("utf8 path");

    let assert = Command::new(cargo_bin("dare"))
        .args([
            "bench",
            "--suite",
            "fixtures/bench",
            "--json",
            "-d",
            root_str,
            "--no-color",
        ])
        .assert()
        .success();

    let v = parse_json_envelope(&assert.get_output().stdout);
    assert_eq!(v["ok"], true);
    let data = &v["data"];
    assert_eq!(data["schemaVersion"], 1);
    assert!(data["fixtures"].as_array().is_some());
    assert!(data.get("fixRate").is_some());
    assert!(data.get("solveRate").is_some());
}

#[test]
fn regression_exit_1() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    // Crafted failing fixture: pass_to_pass test always fails → solveRate 0.
    write_file(
        root,
        "fixtures/bench/suite.json",
        r#"{"schemaVersion":1,"name":"reg","cases":[{"id":"fail-case","path":"cases/fail-case"}]}"#,
    );
    write_file(root, "fixtures/bench/cases/fail-case/patch.diff", "");
    write_file(
        root,
        "fixtures/bench/cases/fail-case/fail_to_pass.txt",
        "tests::never_listed\n",
    );
    write_file(
        root,
        "fixtures/bench/cases/fail-case/pass_to_pass.txt",
        "tests::always_passes\n",
    );
    write_file(root, "fixtures/bench/cases/fail-case/stack.txt", "rust-axum\n");
    write_file(
        root,
        "fixtures/bench/cases/fail-case/repo/Cargo.toml",
        "[package]\nname = \"fail-case\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write_file(
        root,
        "fixtures/bench/cases/fail-case/repo/src/lib.rs",
        "#[cfg(test)]\nmod tests {\n    #[test]\n    fn always_passes() {\n        assert!(false, \"intentional fail for bench regression\");\n    }\n}\n",
    );
    write_file(
        root,
        "bench-baseline.json",
        r#"{"schemaVersion":1,"solveRate":1.0,"fixRate":1.0,"suiteName":"reg"}"#,
    );

    let dir_str = root.to_str().expect("utf8 path");
    let assert = Command::new(cargo_bin("dare"))
        .args([
            "bench",
            "--suite",
            "fixtures/bench",
            "--baseline",
            "bench-baseline.json",
            "--fail-on-regression",
            "0",
            "--json",
            "-d",
            dir_str,
            "--no-color",
        ])
        .assert()
        .code(1);

    let v = parse_json_envelope(&assert.get_output().stdout);
    assert_eq!(v["ok"], false);
    let data = &v["data"];
    assert_eq!(data["schemaVersion"], 1);
    assert_eq!(data["baseline"]["regressionFailed"], true);
    let drop_pp = data["baseline"]["dropSolvePp"].as_f64().expect("drop");
    assert!(drop_pp > 0.0, "drop_pp={drop_pp}");
}
