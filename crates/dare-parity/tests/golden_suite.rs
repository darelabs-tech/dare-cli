//! Integration: golden suite runner against committed `tests/golden` cases.

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use assert_cmd::cargo::cargo_bin;
use dare_core::CoreError;
use dare_parity::{
    load_case, run_case, run_suite, CaseStatus, DiffClass, DiffLogIndex, SkipSpec, SuiteOpts,
    MSG_SKIP_NEEDS_CLASS, MSG_UNCLASSIFIED_DIFF, CASE_SCHEMA_VERSION, CompareAxis, CaseSpec,
};
use std::collections::BTreeMap;
use tempfile::tempdir;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn dare_bin() -> PathBuf {
    let candidate = cargo_bin("dare");
    if candidate.is_file() {
        return candidate;
    }

    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("target"));
    #[cfg(windows)]
    let bin = target_dir.join("debug").join("dare.exe");
    #[cfg(not(windows))]
    let bin = target_dir.join("debug").join("dare");

    if !bin.is_file() {
        let status = Command::new("cargo")
            .args(["build", "-p", "dare-cli", "--bin", "dare"])
            .current_dir(workspace_root())
            .status()
            .expect("spawn cargo build -p dare-cli");
        assert!(status.success(), "cargo build -p dare-cli --bin dare failed");
    }
    assert!(bin.is_file(), "dare binary missing at {}", bin.display());
    bin
}

#[test]
fn golden_suite_help_cases_pass() {
    let root = workspace_root();
    let golden_root = root.join("tests/golden");
    let report_out = golden_root.join("last-report.json");

    let report = run_suite(
        &golden_root,
        SuiteOpts {
            timeout: Duration::from_secs(30),
            bin: Some(dare_bin()),
            fixtures_root: Some(root.join("tests/fixtures")),
            diff_log_path: Some(root.join("docs/compatibility/parity-diff-log.md")),
            report_out: Some(report_out.clone()),
        },
    )
    .expect("run_suite");

    assert!(
        report.summary.fail == 0,
        "golden failures: {:#?}",
        report
            .cases
            .iter()
            .filter(|c| c.status == CaseStatus::Fail)
            .collect::<Vec<_>>()
    );
    assert!(
        report.summary.pass >= 4,
        "expected ≥4 help cases, got pass={}",
        report.summary.pass
    );
    assert!(report_out.is_file(), "SHOULD write last-report.json");

    // Spot-check key command names appear in root help expected snapshot.
    let root_stdout = std::fs::read_to_string(
        golden_root
            .join("cases/help.root/expected/stdout.txt"),
    )
    .expect("root stdout");
    for needle in ["welcome", "self", "mcp", "update"] {
        assert!(
            root_stdout.contains(needle),
            "root help missing {needle}"
        );
    }
}

#[test]
fn skip_without_class_fails_load() {
    let dir = tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("case.yaml"),
        r#"
schemaVersion: 1
id: golden.tmp.skip
command: ["--help"]
axes: [exit]
skip:
  reason: "fixture not materialized yet"
"#,
    )
    .expect("write");
    let err = load_case(dir.path()).expect_err("skip without class");
    match err {
        CoreError::InvalidInput(m) => assert_eq!(m, MSG_SKIP_NEEDS_CLASS),
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}

#[test]
fn unclassified_class_c_skip_fails() {
    let spec = CaseSpec {
        schema_version: CASE_SCHEMA_VERSION,
        id: "golden.tmp.unclassified".into(),
        command: vec!["--help".into()],
        cwd_fixture: None,
        env: BTreeMap::new(),
        axes: vec![CompareAxis::Exit],
        expected_exit: Some(0),
        expected_stdout_path: None,
        expected_stderr_path: None,
        expected_tree_path: None,
        expected_content: vec![],
        expected_state_path: None,
        expected_http: None,
        skip: Some(SkipSpec {
            reason: "not in diff log".into(),
            class: DiffClass::C,
            adr_ref: None,
        }),
    };
    let err = run_case(
        &spec,
        &dare_bin(),
        &workspace_root().join("tests/fixtures"),
        &DiffLogIndex::empty(),
    )
    .expect_err("unclassified");
    match err {
        CoreError::InvalidInput(m) => assert_eq!(m, MSG_UNCLASSIFIED_DIFF),
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}
