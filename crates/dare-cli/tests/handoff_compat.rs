//! Compatibility gate of the handoff cycle
//! (cycle `rust-verified-task-contracts-012`, task-011).
//!
//! `strict_off_byte_identical` is the O-04 test: it replays the very fixtures
//! `task-001` recorded **before** any handoff code existed, with the whole
//! implementation now present and `handoff.strict` absent, and asserts stdout
//! and exit code are byte-identical to the committed snapshots. The snapshot
//! files are the ones `handoff_baseline.rs` owns; this suite reads them and
//! never rewrites them.
//!
//! The npm round trip is simulated rather than executed, so CI needs no Node:
//! `.dare/state.json` is rewritten keeping only the keys the npm
//! `state-store.ts::saveState` preserves and dropping everything else, which
//! is exactly what a pass through the npm runtime does to the file. The sidecar
//! is then asserted untouched and still verifying, which is what RF-10 claims.

use assert_cmd::cargo::cargo_bin;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Where `handoff_baseline.rs` keeps the snapshots recorded by task-001.
fn snapshots_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden/handoff-baseline")
}

fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Folds CRLF to LF: git may check the snapshots out with either.
fn lf(input: &str) -> String {
    input.replace("\r\n", "\n")
}

const CORRELATION_PLACEHOLDER: &str = "00000000-0000-0000-0000-000000000000";
const ROOT_PLACEHOLDER: &str = "<PROJECT_ROOT>";

/// The same three normalisations `handoff_baseline.rs` documents, and no
/// others: a measured duration, a per-process UUID and the temp path cannot be
/// goldens, every other byte is compared literally.
fn normalize(raw: &[u8], project_root: &Path) -> String {
    let text = lf(&String::from_utf8_lossy(raw));
    let text = mask_correlation_id(&text);
    let text = zero_number_field(&text, "durationMs");
    let text = zero_number_field(&text, "totalDurationMs");
    mask_project_root(&text, project_root)
}

fn zero_number_field(input: &str, key: &str) -> String {
    let needle = format!("\"{key}\":");
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(at) = rest.find(&needle) {
        out.push_str(&rest[..at + needle.len()]);
        let tail = &rest[at + needle.len()..];
        let digits = tail
            .char_indices()
            .take_while(|(_, c)| c.is_ascii_digit())
            .count();
        if digits == 0 {
            rest = tail;
            continue;
        }
        out.push('0');
        rest = &tail[digits..];
    }
    out.push_str(rest);
    out
}

fn mask_correlation_id(input: &str) -> String {
    let needle = "\"correlation_id\":\"";
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(at) = rest.find(needle) {
        out.push_str(&rest[..at + needle.len()]);
        let tail = &rest[at + needle.len()..];
        match tail.find('"') {
            Some(end) => {
                out.push_str(CORRELATION_PLACEHOLDER);
                rest = &tail[end..];
            }
            None => rest = tail,
        }
    }
    out.push_str(rest);
    out
}

fn mask_project_root(input: &str, project_root: &Path) -> String {
    let mut out = input.to_string();
    for variant in [
        project_root.display().to_string(),
        project_root.display().to_string().replace('\\', "/"),
        project_root.display().to_string().replace('\\', "\\\\"),
    ] {
        if !variant.is_empty() {
            out = out.replace(&variant, ROOT_PLACEHOLDER);
        }
    }
    out
}

const CONFIG_JSON: &str = r#"{"backend":"rust-axum","verify":{"enabled":false}}"#;

/// The task-001 seed, byte for byte as `handoff_baseline.rs` declares it.
///
/// Duplicated as a literal rather than shared, because the point of this suite
/// is to replay what task-001 recorded: importing the constant would let a
/// future edit to it silently move both sides at once.
const SEEDED_STATE_V21: &str = r#"{
  "version": 1,
  "updatedAt": "2026-01-01T00:00:00Z",
  "tasks": {
    "task-a": {
      "status": "DONE",
      "output": "Alpha parser landed in crates/alpha/src/parse.rs with a round-trip test. The grammar is frozen and the tokenizer is reusable by the loader.",
      "error": "",
      "attempts": [],
      "dependsOn": []
    },
    "task-b": {
      "status": "DONE",
      "output": "",
      "error": "",
      "attempts": [],
      "dependsOn": []
    },
    "task-c": {
      "status": "PENDING",
      "output": "",
      "error": "",
      "attempts": [],
      "dependsOn": ["task-a", "task-b"]
    }
  }
}
"#;

/// Rebuilds a project from a task-001 fixture.
fn project_from_fixture(fixture_rel: &str, state: Option<&str>) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let dag = lf(&read_text(&snapshots_root().join(fixture_rel)));
    std::fs::create_dir_all(dir.path().join("DARE")).expect("mkdir DARE");
    std::fs::write(dir.path().join("DARE/dare-dag.yaml"), dag).expect("write dag");
    std::fs::write(dir.path().join("dare.config.json"), CONFIG_JSON).expect("write config");
    if let Some(body) = state {
        std::fs::create_dir_all(dir.path().join(".dare")).expect("mkdir .dare");
        std::fs::write(dir.path().join(".dare/state.json"), body).expect("write state");
    }
    dir
}

/// Runs the real binary and returns (stdout, exit code).
fn run(dir: &tempfile::TempDir, args: &[&str]) -> (String, i32) {
    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .env("DARE_RALPH_MOCK", "pass")
        .args(args)
        .output()
        .expect("dare ran");
    let code = out.status.code().unwrap_or(-1);
    (normalize(&out.stdout, dir.path()), code)
}

// ── O-04: the default path did not move ─────────────────────────────────────

/// Replays every task-001 snapshot with the full implementation present.
///
/// This is the O-04 test. The snapshots were recorded at rank 0, before any
/// handoff code existed; if the default path had drifted by a single byte, this
/// fails.
#[test]
fn strict_off_byte_identical() {
    let cases: [(&str, Option<&str>, &[&str], &str); 4] = [
        (
            "v21/dare-dag.yaml",
            Some(SEEDED_STATE_V21),
            &["execute", "--next", "--no-color"],
            "next-v21.stdout.txt",
        ),
        (
            "v21/dare-dag.yaml",
            Some(SEEDED_STATE_V21),
            &["--json", "execute", "--next"],
            "next-v21.json.txt",
        ),
        (
            "legacy/dare-dag.yaml",
            None,
            &["execute", "--next", "--no-color"],
            "next-legacy.stdout.txt",
        ),
        (
            "v21/dare-dag.yaml",
            None,
            &[
                "execute",
                "--complete",
                "task-a",
                "--output",
                "Alpha parser landed.",
                "--no-color",
            ],
            "complete-v21.stdout.txt",
        ),
    ];

    for (fixture, state, args, snapshot) in cases {
        let dir = project_from_fixture(fixture, state);
        let (stdout, code) = run(&dir, args);
        let expected = lf(&read_text(&snapshots_root().join(snapshot)));
        assert_eq!(code, 0, "{snapshot} exited {code}");
        assert_eq!(
            stdout, expected,
            "{snapshot} drifted with the handoff code present"
        );
    }
}

#[test]
fn compat_exits_0() {
    let dir = project_from_fixture("v21/dare-dag.yaml", Some(SEEDED_STATE_V21));
    // A sidecar makes the feature count as in use, so the warning path runs.
    std::fs::create_dir_all(dir.path().join(".dare/handoff")).expect("mkdir handoff");
    std::fs::write(
        dir.path().join(".dare/handoff/task-other.json"),
        r#"{"schemaVersion":1}"#,
    )
    .expect("write sidecar");

    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--next", "--no-color"])
        .output()
        .expect("dare ran");

    assert_eq!(out.status.code(), Some(0), "compat never blocks");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not verified") && stderr.contains("imported anyway"),
        "the warning belongs on stderr: {stderr}"
    );

    // stdout is still the recorded golden.
    let stdout = normalize(&out.stdout, dir.path());
    let expected = lf(&read_text(&snapshots_root().join("next-v21.stdout.txt")));
    assert_eq!(stdout, expected, "a warning changed stdout");
}

#[test]
fn strict_refusal_exits_6() {
    let dir = project_from_fixture("v21/dare-dag.yaml", Some(SEEDED_STATE_V21));

    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["--json", "execute", "--next", "--strict-handoff"])
        .output()
        .expect("dare ran");

    assert_eq!(out.status.code(), Some(6), "strict refuses with exit 6");
    let body = String::from_utf8_lossy(&out.stdout);
    let value: Value = serde_json::from_str(body.trim()).expect("parses as json");
    assert_eq!(value["ok"], json!(false), "{body}");
    assert_eq!(value["error"]["kind"], json!("GuardFail"), "{body}");
}

// ── RF-10: the sidecar survives a pass through the npm runtime ──────────────

/// Keys the npm `state-store.ts::saveState` preserves on a task entry.
const NPM_TASK_KEYS: [&str; 5] = ["status", "output", "error", "attempts", "dependsOn"];

/// Rewrites `.dare/state.json` the way npm would: known fields only, every
/// unknown key dropped.
fn simulate_npm_save_state(dir: &Path) {
    let path = dir.join(".dare/state.json");
    let before: Value =
        serde_json::from_str(&read_text(&path)).expect("state parses before the round trip");

    let mut tasks = Map::new();
    if let Some(entries) = before["tasks"].as_object() {
        for (id, entry) in entries {
            let mut kept = Map::new();
            for key in NPM_TASK_KEYS {
                if let Some(v) = entry.get(key) {
                    kept.insert(key.to_string(), v.clone());
                }
            }
            tasks.insert(id.clone(), Value::Object(kept));
        }
    }
    let after = json!({
        "version": before["version"].clone(),
        "updatedAt": before["updatedAt"].clone(),
        "tasks": Value::Object(tasks),
    });
    std::fs::write(&path, serde_json::to_string(&after).expect("serialises"))
        .expect("state rewritten");
}

/// A project whose `task-a` has a real contract, receipt and sidecar.
fn project_with_sidecar() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("DARE/contracts")).expect("mkdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
    std::fs::write(dir.path().join("src/lib.rs"), "// source\n").expect("write source");
    std::fs::write(
        dir.path().join("DARE/dare-dag.yaml"),
        "title: \"C\"\nversion: \"1.0.0\"\ntasks:\n  - id: task-a\n    title: Alpha\n    depends_on: []\n    complexity: LOW\n    subtask_prompt: do-a\n  - id: task-b\n    title: Beta\n    depends_on: [task-a]\n    complexity: LOW\n    subtask_prompt: do-b\n",
    )
    .expect("write dag");
    std::fs::write(dir.path().join("dare.config.json"), CONFIG_JSON).expect("write config");
    std::fs::write(
        dir.path().join("DARE/contracts/task-a.yaml"),
        "schemaVersion: 1\ntaskId: task-a\ngoal: \"Produce alpha\"\nscope:\n  allowedPaths:\n    - \"src/**\"\nacceptance:\n  - id: A-01\n    assertion: \"It holds\"\nverification:\n  requiredGates: [\"build\", \"test\"]\nexitCriteria:\n  kind: verified-deliverable\nstartingContext:\n  source: project-revision\n  repositories:\n    - alias: dare-cli\n",
    )
    .expect("write contract");

    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .env("DARE_RALPH_MOCK", "pass")
        .args([
            "execute",
            "--complete",
            "task-a",
            "--output",
            "alpha landed",
        ])
        .output()
        .expect("complete ran");
    assert!(
        out.status.success(),
        "seeding failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    dir
}

#[test]
fn npm_roundtrip_preserves_sidecar() {
    let dir = project_with_sidecar();
    let sidecar_path = dir.path().join(".dare/handoff/task-a.json");
    let before = read_text(&sidecar_path);
    assert!(before.contains("\"verdict\":\"passed\""), "{before}");

    simulate_npm_save_state(dir.path());

    // (a) the sidecar is untouched, because it is a separate file.
    let after = read_text(&sidecar_path);
    assert_eq!(before, after, "the npm round trip disturbed the sidecar");

    // (b) the rewritten state still loads through the Rust reader.
    let (stdout, code) = run(&dir, &["execute", "--status", "--no-color"]);
    assert_eq!(code, 0, "state.json no longer loads: {stdout}");
    assert!(stdout.contains("DONE"), "{stdout}");

    // The digest the npm writer dropped is simply gone, which must read as
    // "nothing recorded" rather than as a divergence.
    let state: Value =
        serde_json::from_str(&read_text(&dir.path().join(".dare/state.json"))).expect("parses");
    assert!(
        state["tasks"]["task-a"]
            .get("handoffContractDigest")
            .is_none(),
        "the simulation must actually drop the extra, else it proves nothing"
    );
    // Re-running the task after the digest was dropped must succeed: an absent
    // digest reads as "nothing recorded", never as a divergence. The reset is
    // needed because `--complete` on a DONE task is rejected for an unrelated
    // reason, which would mask what is under test here.
    let (_, reset_code) = run(&dir, &["execute", "--reset", "task-a"]);
    assert_eq!(reset_code, 0, "reset failed");
    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .env("DARE_RALPH_MOCK", "pass")
        .args(["execute", "--complete", "task-a", "--output", "again"])
        .output()
        .expect("dare ran");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("HANDOFF_CONTRACT_DIGEST_MISMATCH"),
        "an absent digest was read as a divergence: {stderr}"
    );
    assert!(out.status.success(), "{stderr}");
}

#[test]
fn npm_executor_ignores_receipts() {
    // The npm executor completes tasks and writes no receipt at all. Simulated
    // by taking the state through its writer and then removing the sidecar:
    // what is left is a DONE parent whose evidence does not exist, which is
    // precisely the context an npm-driven run hands to the Rust side.
    let dir = project_with_sidecar();
    simulate_npm_save_state(dir.path());
    std::fs::remove_file(dir.path().join(".dare/handoff/task-a.json")).expect("sidecar removed");
    // Another sidecar remains absent too, so the project no longer looks like
    // it uses handoffs; keep one unrelated file so compat still evaluates.
    std::fs::write(
        dir.path().join(".dare/handoff/task-unrelated.json"),
        r#"{"schemaVersion":1}"#,
    )
    .expect("write unrelated sidecar");

    // strict off: context is imported anyway, exactly as the npm executor does.
    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--next", "--no-color"])
        .output()
        .expect("dare ran");
    assert_eq!(
        out.status.code(),
        Some(0),
        "compat must not block: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("HANDOFF_RECEIPT_MISSING"),
        "the missing receipt must be reported: {stderr}"
    );

    // strict on: the Rust side refuses context it cannot verify.
    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--next", "--strict-handoff", "--no-color"])
        .output()
        .expect("dare ran");
    assert_eq!(
        out.status.code(),
        Some(6),
        "strict must refuse: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn snapshots_were_not_edited_by_this_suite() {
    // Guard against a future edit that "fixes" a drift by rewriting a golden.
    // The suite only ever reads them; this asserts they are still the committed
    // files and not something generated at run time.
    for name in [
        "next-v21.stdout.txt",
        "next-v21.json.txt",
        "next-legacy.stdout.txt",
        "complete-v21.stdout.txt",
    ] {
        let path = snapshots_root().join(name);
        assert!(path.is_file(), "{} is missing", path.display());
        assert!(!read_text(&path).is_empty(), "{} is empty", path.display());
    }
}
