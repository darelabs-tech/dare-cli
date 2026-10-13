//! CLI surface of the strict-handoff opt-in
//! (cycle `rust-verified-task-contracts-012`, task-009).
//!
//! Three properties are asserted here, each of which can only be seen from
//! outside the process:
//!
//! 1. `dare execute --help` offers the two new flags and no existing flag's
//!    help text changed.
//! 2. A strict refusal exits **6** and, under `--json`, carries
//!    `error.kind == "GuardFail"`.
//! 3. In compatibility mode a refusal reaches **stderr only**: exit stays 0 and
//!    stdout is what it was before the feature existed.
//!
//! Property 3 is the one that matters for O-04, and the byte-exact half of it
//! lives in `handoff_baseline.rs`, which replays committed snapshots of stdout.
//! Here it is asserted against a live run with handoffs in use, which the
//! baseline fixtures deliberately do not exercise.

use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

const CONFIG_JSON: &str =
    r#"{"name":"t","structure":"unknown","ide":"antigravity","version":"4.0.0"}"#;

const DAG_YAML: &str = r#"title: "Handoff policy CLI"
version: "1.0.0"
tasks:
  - id: task-a
    title: Alpha
    depends_on: []
    complexity: LOW
    subtask_prompt: do-a
  - id: task-b
    title: Beta
    depends_on: [task-a]
    complexity: LOW
    subtask_prompt: do-b
"#;

/// `task-a` DONE with output, `task-b` ready and depending on it.
const STATE_JSON: &str = r#"{"version":1,"updatedAt":"2026-01-01T00:00:00Z","tasks":{"task-a":{"status":"DONE","output":"alpha output","error":"","attempts":[],"dependsOn":[]},"task-b":{"status":"PENDING","output":"","error":"","attempts":[],"dependsOn":["task-a"]}}}"#;

/// A sidecar for a task that is *not* `task-a`, so the project counts as using
/// handoffs while `task-a` still has none. That is the compatibility case: the
/// project opted in, one parent lacks evidence.
const SIDECAR_JSON: &str = r#"{"schemaVersion":1,"taskId":"task-z","kind":"verified-deliverable","deliverable":"seeded","contextBounds":2000,"nextSteps":[],"openItems":[],"superseded":[],"receipt":{"schemaVersion":1,"taskId":"task-z","contractDigest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","contractFullDigest":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","sourceRevision":{"state":"clean","repositories":[]},"artifactRefs":[],"artifactDigests":[],"verificationRef":".dare/verification/task-z.json","verificationDigest":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd","verdict":"passed","createdAt":"2026-10-13T12:00:00Z","producer":{"cliVersion":"4.0.0","driver":"cli"}}}"#;

/// A project with the DAG, config and seeded state; `sidecar` adds
/// `.dare/handoff/task-z.json` so the project counts as using handoffs.
fn project(sidecar: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("DARE")).expect("mkdir DARE");
    std::fs::write(dir.path().join("DARE/dare-dag.yaml"), DAG_YAML).expect("write dag");
    std::fs::write(dir.path().join("dare.config.json"), CONFIG_JSON).expect("write config");
    std::fs::create_dir_all(dir.path().join(".dare")).expect("mkdir .dare");
    std::fs::write(dir.path().join(".dare/state.json"), STATE_JSON).expect("write state");
    if sidecar {
        std::fs::create_dir_all(dir.path().join(".dare/handoff")).expect("mkdir handoff");
        std::fs::write(dir.path().join(".dare/handoff/task-z.json"), SIDECAR_JSON)
            .expect("write sidecar");
    }
    dir
}

#[test]
fn help_lists_the_two_new_flags() {
    let out = Command::new(cargo_bin("dare"))
        .args(["execute", "--help"])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let help = String::from_utf8(out).expect("help is utf-8");

    assert!(help.contains("--strict-handoff"), "{help}");
    assert!(help.contains("--no-strict-handoff"), "{help}");
    assert!(
        help.contains("Enforce verified handoffs before importing parent context"),
        "{help}"
    );
    assert!(
        help.contains("Disable strict handoff enforcement for this run"),
        "{help}"
    );
}

#[test]
fn help_text_of_existing_flags_is_unchanged() {
    let out = Command::new(cargo_bin("dare"))
        .args(["execute", "--help"])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let help = String::from_utf8(out).expect("help is utf-8");

    // A sample across the whole subcommand, including the neighbours of the
    // new pair, so a stray edit to an existing description is caught.
    for expected in [
        "Show DAG runtime status",
        "Print next executable tasks at the minimum ready rank",
        "Mark task DONE after Ralph gates pass",
        "Mark task FAILED (cascade skip dependents)",
        "Reset task to PENDING (preserves attempts)",
        "Run advanced verify after Ralph (default: on / config `verify.enabled`)",
        "Skip advanced verify (and treat verify as off)",
        "Path to dare-dag.yaml (default: DARE/dare-dag.yaml)",
    ] {
        assert!(
            help.contains(expected),
            "existing help text changed, missing: {expected}\n{help}"
        );
    }
}

#[test]
fn strict_refusal_exits_6() {
    let dir = project(false);
    // `task-a` is DONE with no sidecar, so strict mode cannot import it.
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--next", "--strict-handoff", "--no-color"])
        .assert()
        .code(6)
        .stderr(predicate::str::contains("handoff"));
}

#[test]
fn strict_refusal_json_is_guard_fail() {
    let dir = project(false);
    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["--json", "execute", "--next", "--strict-handoff"])
        .assert()
        .code(6)
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(out).expect("json is utf-8");
    let value: serde_json::Value = serde_json::from_str(body.trim()).expect("parses as json");

    assert_eq!(value["ok"], serde_json::json!(false), "{body}");
    assert_eq!(
        value["error"]["kind"],
        serde_json::json!("GuardFail"),
        "{body}"
    );
    let message = value["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("HANDOFF_STRICT_NO_CONTRACTS") || message.contains("handoff refused"),
        "the refusal must carry a stable code: {body}"
    );
}

#[test]
fn no_strict_handoff_flag_overrides_strict_config() {
    let dir = project(false);
    std::fs::write(
        dir.path().join("dare.config.json"),
        r#"{"name":"t","structure":"unknown","ide":"antigravity","version":"4.0.0","handoff":{"strict":true}}"#,
    )
    .expect("write config");

    // With the config alone this run would exit 6; the flag takes it back to 0.
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--next", "--strict-handoff", "--no-color"])
        .assert()
        .code(6);

    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--next", "--no-strict-handoff", "--no-color"])
        .assert()
        .code(0);
}

#[test]
fn compat_warns_on_stderr_only() {
    let with_sidecar = project(true);
    let out = Command::new(cargo_bin("dare"))
        .current_dir(with_sidecar.path())
        .args(["execute", "--next", "--no-color"])
        .assert()
        .code(0)
        .get_output()
        .clone();

    let stdout = String::from_utf8(out.stdout.clone()).expect("stdout is utf-8");
    let stderr = String::from_utf8(out.stderr.clone()).expect("stderr is utf-8");

    assert!(
        stderr.contains("not verified"),
        "the compatibility warning belongs on stderr: {stderr}"
    );
    assert!(stderr.contains("imported anyway"), "{stderr}");
    assert!(
        stderr.contains("this is not verification"),
        "the warning must never read as a verification: {stderr}"
    );
    assert!(
        !stdout.contains("not verified"),
        "stdout must stay clean: {stdout}"
    );

    // The decisive half: stdout is identical to the same project without any
    // handoff in use, so turning the feature on changes nothing a pipe sees.
    let without = project(false);
    let baseline = Command::new(cargo_bin("dare"))
        .current_dir(without.path())
        .args(["execute", "--next", "--no-color"])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let baseline = String::from_utf8(baseline).expect("stdout is utf-8");

    assert_eq!(
        stdout.replace("\r\n", "\n"),
        baseline.replace("\r\n", "\n"),
        "a compatibility warning changed stdout"
    );
}

#[test]
fn default_run_emits_no_handoff_output_at_all() {
    let dir = project(false);
    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--next", "--no-color"])
        .assert()
        .code(0)
        .get_output()
        .clone();

    let stderr = String::from_utf8(out.stderr).expect("stderr is utf-8");
    assert!(
        stderr.is_empty(),
        "a project not using handoffs must stay completely silent: {stderr}"
    );
}
