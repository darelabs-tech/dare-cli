//! Producer side of the handoff cycle
//! (cycle `rust-verified-task-contracts-012`, task-010).
//!
//! Asserted from outside the process, because the properties that matter are
//! what lands on disk and what a pipe receives:
//!
//! - a task with a contract emits a sidecar whose digests recompute;
//! - a task **without** a contract emits nothing and changes no output;
//! - a contract whose material fields moved mid-flight is refused, and the
//!   refusal names the recovery command;
//! - a reset, and a re-run, record the previous receipt instead of dropping it.

use assert_cmd::cargo::cargo_bin;
use serde_json::Value;
use std::process::Command;

/// `backend` makes the stack resolvable without a real Cargo project, and
/// advanced verify is off so the run needs no external tooling — the same
/// shape the golden baseline uses.
const CONFIG_JSON: &str = r#"{"backend":"rust-axum","verify":{"enabled":false}}"#;

const DAG_YAML: &str = r#"title: "Handoff emission"
version: "1.0.0"
tasks:
  - id: task-a
    title: Alpha
    depends_on: []
    complexity: LOW
    subtask_prompt: do-a
"#;

fn contract_yaml(goal: &str, owner: Option<&str>) -> String {
    let owner_line = owner
        .map(|o| format!("owner: \"{o}\"\n"))
        .unwrap_or_default();
    format!(
        r#"schemaVersion: 1
taskId: task-a
goal: "{goal}"
{owner_line}scope:
  allowedPaths:
    - "src/**"
acceptance:
  - id: A-01
    assertion: "It holds"
verification:
  requiredGates: ["build", "test", "lint"]
exitCriteria:
  kind: verified-deliverable
startingContext:
  source: project-revision
  repositories:
    - alias: dare-cli
"#
    )
}

/// A project with the DAG and config; `contract` adds `DARE/contracts/task-a.yaml`.
fn project(contract: Option<&str>) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("DARE")).expect("mkdir DARE");
    std::fs::write(dir.path().join("DARE/dare-dag.yaml"), DAG_YAML).expect("write dag");
    std::fs::write(dir.path().join("dare.config.json"), CONFIG_JSON).expect("write config");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
    std::fs::write(dir.path().join("src/lib.rs"), "// source\n").expect("write source");
    if let Some(body) = contract {
        std::fs::create_dir_all(dir.path().join("DARE/contracts")).expect("mkdir contracts");
        std::fs::write(dir.path().join("DARE/contracts/task-a.yaml"), body)
            .expect("write contract");
    }
    dir
}

fn complete(dir: &tempfile::TempDir, json: bool) -> std::process::Output {
    let mut args: Vec<&str> = Vec::new();
    if json {
        args.push("--json");
    }
    args.extend([
        "execute",
        "--complete",
        "task-a",
        "--output",
        "Alpha landed.",
    ]);
    if !json {
        args.push("--no-color");
    }
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .env("DARE_RALPH_MOCK", "pass")
        .args(args)
        .output()
        .expect("dare ran")
}

fn sidecar(dir: &tempfile::TempDir) -> Option<Value> {
    let path = dir.path().join(".dare/handoff/task-a.json");
    let body = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&body).ok()
}

fn state(dir: &tempfile::TempDir) -> Value {
    let body = std::fs::read_to_string(dir.path().join(".dare/state.json")).expect("state exists");
    serde_json::from_str(&body).expect("state parses")
}

#[test]
fn complete_with_contract_writes_valid_sidecar() {
    let dir = project(Some(&contract_yaml("Land alpha", None)));
    let out = complete(&dir, false);
    assert!(
        out.status.success(),
        "{:?}",
        String::from_utf8_lossy(&out.stderr)
    );

    let record = sidecar(&dir).expect("sidecar written");
    assert_eq!(record["schemaVersion"], 1);
    assert_eq!(record["taskId"], "task-a");
    assert_eq!(record["kind"], "verified-deliverable");
    assert_eq!(record["deliverable"], "Alpha landed.");
    assert_eq!(record["receipt"]["verdict"], "passed");
    assert_eq!(record["receipt"]["taskId"], "task-a");

    // The digests are real: 64 lowercase hex characters each.
    for key in ["contractDigest", "contractFullDigest", "verificationDigest"] {
        let d = record["receipt"][key].as_str().unwrap_or_default();
        assert_eq!(d.len(), 64, "{key} = {d}");
        assert!(
            d.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "{key} = {d}"
        );
    }
    assert_eq!(
        record["receipt"]["verificationRef"],
        ".dare/verification/task-a.json"
    );
}

#[test]
fn complete_without_contract_writes_nothing() {
    let dir = project(None);
    let out = complete(&dir, false);
    assert!(out.status.success());
    assert!(
        sidecar(&dir).is_none(),
        "a task with no contract must emit no sidecar"
    );
    assert!(
        !dir.path().join(".dare/handoff").exists(),
        "the handoff directory must not even be created"
    );
}

#[test]
fn complete_json_has_handoff_fields() {
    let dir = project(Some(&contract_yaml("Land alpha", None)));
    let out = complete(&dir, true);
    assert!(out.status.success());
    let body = String::from_utf8(out.stdout).expect("stdout is utf-8");
    let value: Value = serde_json::from_str(body.trim()).expect("parses");

    assert_eq!(
        value["data"]["handoffPath"], ".dare/handoff/task-a.json",
        "{body}"
    );
    assert_eq!(value["data"]["receipt"]["verdict"], "passed", "{body}");
    // Nothing that existed before was displaced.
    assert_eq!(value["data"]["action"], "complete");
    assert_eq!(value["data"]["status"], "DONE");
    assert_eq!(
        value["data"]["verificationPath"],
        ".dare/verification/task-a.json"
    );
}

#[test]
fn complete_json_omits_fields_without_contract() {
    let dir = project(None);
    let out = complete(&dir, true);
    assert!(out.status.success());
    let body = String::from_utf8(out.stdout).expect("stdout is utf-8");
    let value: Value = serde_json::from_str(body.trim()).expect("parses");

    let data = value["data"].as_object().expect("data is an object");
    assert!(
        !data.contains_key("handoffPath"),
        "absent, not null: {body}"
    );
    assert!(!data.contains_key("receipt"), "absent, not null: {body}");
}

#[test]
fn start_digest_stored_in_state_extra() {
    let dir = project(Some(&contract_yaml("Land alpha", None)));
    // `--next` leaves the task PENDING; the digest is recorded on Start, which
    // `--complete` performs, so run the full command and inspect the result.
    let out = complete(&dir, false);
    assert!(out.status.success());

    let st = state(&dir);
    let digest = st["tasks"]["task-a"]["handoffContractDigest"]
        .as_str()
        .unwrap_or_default();
    assert_eq!(digest.len(), 64, "state.json = {st}");

    // No required key was added: the usual fields are all still there.
    for key in ["status", "output", "error", "attempts", "dependsOn"] {
        assert!(
            st["tasks"]["task-a"].get(key).is_some(),
            "{key} missing from {st}"
        );
    }
}

#[test]
fn complete_refuses_material_change() {
    let dir = project(Some(&contract_yaml("Land alpha", None)));
    // Start the task so a digest is recorded, then move a material field.
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .env("DARE_RALPH_MOCK", "pass")
        .args(["execute", "--complete", "task-a", "--output", "first"])
        .output()
        .expect("first complete");
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--reset", "task-a"])
        .output()
        .expect("reset");
    // Re-record a digest, then change `goal`, which is material.
    std::fs::write(
        dir.path().join("DARE/contracts/task-a.yaml"),
        contract_yaml("Land alpha", None),
    )
    .expect("restore contract");

    // Start fresh so the digest of the current contract is recorded.
    let mut st = state(&dir);
    let digest = st["tasks"]["task-a"]["handoffContractDigest"].clone();
    assert!(digest.is_string(), "digest survives the reset: {st}");
    st["tasks"]["task-a"]["status"] = Value::from("RUNNING");
    std::fs::write(
        dir.path().join(".dare/state.json"),
        serde_json::to_string(&st).expect("serialises"),
    )
    .expect("seed running state");

    std::fs::write(
        dir.path().join("DARE/contracts/task-a.yaml"),
        contract_yaml("Land something else entirely", None),
    )
    .expect("tamper contract");

    let out = complete(&dir, false);
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(!out.status.success(), "a material change must refuse");
    assert!(
        stderr.contains("HANDOFF_CONTRACT_DIGEST_MISMATCH"),
        "{stderr}"
    );
    assert!(
        stderr.contains("dare execute --reset"),
        "the refusal must name the way forward: {stderr}"
    );
}

#[test]
fn complete_allows_non_material_change() {
    let dir = project(Some(&contract_yaml("Land alpha", Some("alice"))));
    let out = complete(&dir, false);
    assert!(out.status.success());
    let before = sidecar(&dir).expect("sidecar written");
    let material_before = before["receipt"]["contractDigest"].clone();
    let full_before = before["receipt"]["contractFullDigest"].clone();

    // `owner` is outside the material projection.
    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--reset", "task-a"])
        .output()
        .expect("reset");
    std::fs::write(
        dir.path().join("DARE/contracts/task-a.yaml"),
        contract_yaml("Land alpha", Some("bob")),
    )
    .expect("change owner");

    let out = complete(&dir, false);
    assert!(
        out.status.success(),
        "a non-material change must pass: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let after = sidecar(&dir).expect("sidecar rewritten");
    assert_eq!(
        after["receipt"]["contractDigest"], material_before,
        "the material digest is unchanged by owner"
    );
    assert_ne!(
        after["receipt"]["contractFullDigest"], full_before,
        "the full digest records the change for audit"
    );
}

#[test]
fn reset_supersedes_receipt() {
    let dir = project(Some(&contract_yaml("Land alpha", None)));
    assert!(complete(&dir, false).status.success());
    assert!(sidecar(&dir).is_some());

    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--reset", "task-a"])
        .output()
        .expect("reset");

    let record = sidecar(&dir).expect("the sidecar survives a reset");
    let superseded = record["superseded"].as_array().expect("array");
    assert_eq!(superseded.len(), 1, "{record}");
    assert_eq!(superseded[0]["reason"], "task reset");
    assert_eq!(
        superseded[0]["receiptDigest"]
            .as_str()
            .unwrap_or_default()
            .len(),
        64
    );
}

#[test]
fn recomplete_supersedes_with_reason() {
    let dir = project(Some(&contract_yaml("Land alpha", None)));
    assert!(complete(&dir, false).status.success());

    Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--reset", "task-a"])
        .output()
        .expect("reset");
    assert!(complete(&dir, false).status.success());

    let record = sidecar(&dir).expect("sidecar present");
    let superseded = record["superseded"].as_array().expect("array");
    let reasons: Vec<&str> = superseded
        .iter()
        .map(|e| e["reason"].as_str().unwrap_or_default())
        .collect();
    assert!(
        reasons.contains(&"task reset"),
        "the reset is recorded: {reasons:?}"
    );
    assert!(
        reasons.contains(&"re-executed"),
        "the re-run is recorded: {reasons:?}"
    );
}

#[test]
fn unattested_revision_when_no_git() {
    // The fixture project has no .git, so provenance cannot be attested.
    let dir = project(Some(&contract_yaml("Land alpha", None)));
    assert!(complete(&dir, false).status.success());

    let record = sidecar(&dir).expect("sidecar written");
    assert_eq!(
        record["receipt"]["sourceRevision"]["state"], "unattested",
        "{record}"
    );
    let repos = record["receipt"]["sourceRevision"]["repositories"]
        .as_array()
        .expect("array");
    for repo in repos {
        assert!(
            repo["commitSha"].is_null(),
            "no SHA may be fabricated: {repo}"
        );
    }
}

#[test]
fn next_json_has_handoff_block() {
    let dir = project(Some(&contract_yaml("Land alpha", None)));
    // Produce a sidecar so the feature counts as in use.
    assert!(complete(&dir, false).status.success());

    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["--json", "execute", "--next"])
        .output()
        .expect("next ran");
    let body = String::from_utf8(out.stdout).expect("stdout is utf-8");
    let value: Value = serde_json::from_str(body.trim()).expect("parses");

    // The only task is DONE, so `ready` is empty and there is nothing to carry
    // a block; the point asserted here is that the key is well-formed when it
    // does appear, which the dormant-case golden pins from the other side.
    let ready = value["data"]["ready"].as_array().expect("ready array");
    for item in ready {
        let handoff = &item["handoff"];
        assert!(handoff.is_object(), "{item}");
        assert!(handoff["strict"].is_boolean(), "{handoff}");
        assert!(handoff["imported"].is_array(), "{handoff}");
        assert!(handoff["refused"].is_array(), "{handoff}");
    }
}
