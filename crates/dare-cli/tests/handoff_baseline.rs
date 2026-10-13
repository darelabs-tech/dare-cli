//! Golden baseline of `dare execute --next` / `--complete`
//! (cycle `rust-verified-task-contracts-012`, task-001).
//!
//! This suite records the behaviour of `dare execute` **before** any handoff
//! or task-contract code exists. `task-011` replays the very same fixtures
//! with the full handoff implementation present and asserts the stdout and
//! exit codes are byte-identical to the snapshots committed here. Recording
//! the baseline after a change to `compose_task_prompt` or `execute.rs` would
//! capture the already-changed behaviour and prove nothing, which is why this
//! task is at rank 0 and touches nothing under `src/`.
//!
//! Snapshots live in `tests/fixtures/golden/handoff-baseline/` as committed
//! files and are compared in full — never as substring predicates.
//!
//! # Normalisation
//!
//! Only three classes of bytes are rewritten before comparison, each one a
//! value that cannot be a golden because it is not a property of the CLI's
//! behaviour:
//!
//! 1. `correlation_id` — a fresh UUID v4 per process (`output.rs`
//!    `write_success`). Rewritten to a fixed placeholder.
//! 2. The absolute path of the temporary project. Rewritten to the literal
//!    token `<PROJECT_ROOT>` — *not* deleted. No snapshot contains that
//!    token today, so should a future change start leaking the project path
//!    into stdout the diff shows up as a stable, reviewable failure instead
//!    of a machine-dependent one.
//! 3. `durationMs` / `totalDurationMs` — measured wall-clock, in milliseconds,
//!    of the Ralph gate steps. `DARE_RALPH_MOCK=pass` makes them 0 in
//!    practice, but a scheduler hiccup can round to 1; a measured duration can
//!    never be a byte-exact golden. Rewritten to 0.
//!
//! Line endings are folded to LF on both sides because git may check the
//! snapshot files out with CRLF, exactly as `init_npm_parity_tests.rs` does.
//!
//! Nothing else is touched: every other byte of stdout is asserted literally,
//! so a real regression has nowhere to hide.
//!
//! # Regenerating
//!
//! `DARE_HANDOFF_BASELINE_BLESS=1 cargo test -p dare-cli --test handoff_baseline`
//! rewrites the snapshot files from a real run of the `dare` binary and then
//! **fails on purpose**, so a blessed run can never be mistaken for a green
//! one. Review the diff, then re-run without the variable.

use std::path::{Path, PathBuf};

use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;

/// Fixture directory, relative to the `dare-cli` manifest.
const FIXTURE_DIR: &str = "tests/fixtures/golden/handoff-baseline";

/// Placeholder written in place of the per-run `correlation_id`.
const CORRELATION_PLACEHOLDER: &str = "00000000-0000-0000-0000-000000000000";

/// Placeholder written in place of the temporary project path.
const ROOT_PLACEHOLDER: &str = "<PROJECT_ROOT>";

/// Env var that rewrites the snapshots instead of asserting against them.
const BLESS_ENV: &str = "DARE_HANDOFF_BASELINE_BLESS";

/// Fixture project config.
///
/// `backend` picks the `rust-axum` Ralph gates (build → test → lint), which
/// `DARE_RALPH_MOCK=pass` then satisfies with three mock exits.
///
/// `verify.enabled: false` switches advanced verify off. That is deliberate:
/// with it on, the `mutation` aspect probes for `cargo-mutants` and its
/// recorded `reason` differs between a machine that has the tool and one that
/// does not — an environment-dependent byte in a committed golden. Advanced
/// verify is outside this cycle's blast radius, so the baseline pins the
/// deterministic verdict shape (`"aspects": []`) and still records every field
/// `complete_json` emits.
const CONFIG_JSON: &str = r#"{"backend":"rust-axum","verify":{"enabled":false}}"#;

/// Seed for `.dare/state.json` in the v2.1 fixture project.
///
/// `task-a` is DONE with a non-empty `output` and `task-b` is DONE with an
/// empty one, so `--next` reaches `task-c` (which depends on both) and the
/// snapshot covers the two branches of `compose_task_prompt`: a parent that
/// contributes an `### From parent:` section and a DONE parent that is skipped
/// because its output is blank. `task-a`'s output is longer than the DAG's
/// `parent_context_chars: 120`, so `unicode_tail` truncation is recorded too.
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

// ─── fixture plumbing ───────────────────────────────────────────────────────

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIR)
}

fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn write_text(path: &Path, body: &str) {
    std::fs::write(path, body).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
}

fn lf(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn bless_enabled() -> bool {
    std::env::var(BLESS_ENV).is_ok_and(|v| v == "1")
}

/// Build a project whose `DARE/dare-dag.yaml` is the committed fixture DAG.
///
/// The DAG is folded to LF before being written into the temporary project:
/// `core.autocrlf` may hand the fixture over with CRLF, and a stray `\r`
/// inside a `subtask_prompt` block scalar would end up in the recorded
/// prompt and make the snapshot depend on the checkout, not on the CLI.
fn project_from_fixture(fixture_rel: &str, seed_state: Option<&str>) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let dag_yaml = lf(&read_text(&fixtures_root().join(fixture_rel)));
    std::fs::create_dir_all(dir.path().join("DARE")).expect("mkdir DARE");
    write_text(&dir.path().join("DARE/dare-dag.yaml"), &dag_yaml);
    write_text(&dir.path().join("dare.config.json"), CONFIG_JSON);
    if let Some(state) = seed_state {
        std::fs::create_dir_all(dir.path().join(".dare")).expect("mkdir .dare");
        write_text(&dir.path().join(".dare/state.json"), state);
    }
    dir
}

// ─── normalisation ──────────────────────────────────────────────────────────

/// Replace every `"{key}":<digits>` occurrence with `"{key}":0`.
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

/// Replace the 36-character UUID that follows `"correlation_id":"`.
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
            None => {
                rest = tail;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Replace the temporary project path in all the spellings it can appear in.
fn mask_project_root(input: &str, root: &Path) -> String {
    let mut variants: Vec<String> = Vec::new();
    let mut push = |p: &Path| {
        let raw = p.to_string_lossy().to_string();
        // JSON-escaped Windows separators first: the longest spelling wins.
        variants.push(raw.replace('\\', "\\\\"));
        variants.push(raw.replace('\\', "/"));
        variants.push(raw);
    };
    if let Ok(canon) = root.canonicalize() {
        push(&canon);
    }
    push(root);
    variants.sort_by_key(|v| std::cmp::Reverse(v.len()));
    variants.dedup();

    let mut out = input.to_string();
    for v in variants {
        if v.is_empty() {
            continue;
        }
        out = out.replace(&v, ROOT_PLACEHOLDER);
    }
    out
}

fn normalize(stdout: &[u8], root: &Path) -> String {
    let text = lf(&String::from_utf8_lossy(stdout));
    let text = mask_project_root(&text, root);
    let text = mask_correlation_id(&text);
    let text = zero_number_field(&text, "totalDurationMs");
    zero_number_field(&text, "durationMs")
}

/// Compare the normalised stdout with the committed snapshot, or rewrite it
/// under [`BLESS_ENV`] and fail so a blessed run is never green.
fn assert_snapshot(name: &str, actual: &str) {
    let path = fixtures_root().join(name);
    if bless_enabled() {
        write_text(&path, actual);
        panic!("blessed {name}: re-run without {BLESS_ENV} to assert against it");
    }
    let expected = lf(&read_text(&path));
    assert_eq!(
        actual, expected,
        "stdout does not match the committed baseline {name}"
    );
}

// ─── scenarios ──────────────────────────────────────────────────────────────

/// `dare execute --next` on the v2.1 DAG, human renderer.
///
/// Anchors the composed prompt with the `## Upstream context` header and the
/// per-parent section header, before either gains a verification gate.
#[test]
fn baseline_next_v21_human() {
    let dir = project_from_fixture("v21/dare-dag.yaml", Some(SEEDED_STATE_V21));
    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--next", "--no-color"])
        .assert()
        .code(0)
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();
    let actual = normalize(&out, dir.path());
    assert!(
        actual.contains("## Upstream context"),
        "fixture must exercise upstream composition, got:\n{actual}"
    );
    assert_snapshot("next-v21.stdout.txt", &actual);
}

/// `dare execute --next --json` on the v2.1 DAG.
///
/// Anchors the success envelope and every field of a `ready` item before they
/// gain a `handoff` block in task-010.
#[test]
fn baseline_next_v21_json() {
    let dir = project_from_fixture("v21/dare-dag.yaml", Some(SEEDED_STATE_V21));
    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["--json", "execute", "--next"])
        .assert()
        .code(0)
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();
    let actual = normalize(&out, dir.path());
    assert!(
        !actual.contains(ROOT_PLACEHOLDER),
        "the project path leaked into --next --json:\n{actual}"
    );
    assert_snapshot("next-v21.json.txt", &actual);
}

/// `dare execute --complete task-a` on the v2.1 DAG, human renderer.
#[test]
fn baseline_complete_v21_human() {
    let dir = project_from_fixture("v21/dare-dag.yaml", None);
    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .env("DARE_RALPH_MOCK", "pass")
        .args([
            "execute",
            "--complete",
            "task-a",
            "--output",
            "Alpha parser landed.",
            "--no-color",
        ])
        .assert()
        .code(0)
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();
    let actual = normalize(&out, dir.path());
    assert_snapshot("complete-v21.stdout.txt", &actual);
}

/// `dare execute --complete task-a --json` on the v2.1 DAG.
///
/// Anchors `complete_json` before it gains `handoffPath` and `receipt` in
/// task-010: those fields are additive, so every key recorded here must
/// survive unchanged.
#[test]
fn baseline_complete_v21_json() {
    let dir = project_from_fixture("v21/dare-dag.yaml", None);
    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .env("DARE_RALPH_MOCK", "pass")
        .args([
            "--json",
            "execute",
            "--complete",
            "task-a",
            "--output",
            "Alpha parser landed.",
        ])
        .assert()
        .code(0)
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();
    let actual = normalize(&out, dir.path());
    assert!(
        !actual.contains("handoffPath") && !actual.contains("\"receipt\""),
        "the baseline must predate the handoff fields:\n{actual}"
    );
    assert_snapshot("complete-v21.json.txt", &actual);
    assert!(
        dir.path().join(".dare/verification/task-a.json").is_file(),
        "complete must still write the verification sidecar"
    );
}

/// `dare execute --next` on the legacy flat DAG, human renderer.
///
/// Legacy documents carry no `subtask_prompt`, so the prompt is empty and no
/// `## Upstream context` header is ever emitted. O-04 ("legacy flows keep
/// working") is measured against this file.
#[test]
fn baseline_next_legacy_human() {
    let dir = project_from_fixture("legacy/dare-dag.yaml", None);
    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .args(["execute", "--next", "--no-color"])
        .assert()
        .code(0)
        .stderr(predicate::str::is_empty())
        .get_output()
        .stdout
        .clone();
    let actual = normalize(&out, dir.path());
    assert!(
        !actual.contains("## Upstream context"),
        "legacy DAG must not compose upstream context:\n{actual}"
    );
    assert_snapshot("next-legacy.stdout.txt", &actual);
}

// ─── guards on the suite itself ─────────────────────────────────────────────

/// The snapshots must be committed files, not strings built in this test.
#[test]
fn snapshots_are_committed_files() {
    let names = [
        "next-v21.stdout.txt",
        "next-v21.json.txt",
        "complete-v21.stdout.txt",
        "complete-v21.json.txt",
        "next-legacy.stdout.txt",
        "v21/dare-dag.yaml",
        "legacy/dare-dag.yaml",
    ];
    for name in names {
        let path = fixtures_root().join(name);
        assert!(path.is_file(), "missing fixture {}", path.display());
        assert!(
            !read_text(&path).is_empty(),
            "empty fixture {}",
            path.display()
        );
    }
}

/// No fixture may carry a secret or an absolute path from the machine that
/// recorded it: these files are reviewed in a PR and live in git forever.
#[test]
fn snapshots_carry_no_secret_or_absolute_path() {
    let root = fixtures_root();
    let mut checked = 0usize;
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read_dir") {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let body = read_text(&path);
            let shown = path.display();
            assert!(
                !body.contains(ROOT_PLACEHOLDER),
                "{shown} contains a normalised absolute path"
            );
            for pattern in ["AKIA", "token=", "Authorization:", "Bearer ", "secret="] {
                assert!(!body.contains(pattern), "{shown} contains `{pattern}`");
            }
            for pattern in ["C:\\Users", "C:/Users", "/home/", "/Users/"] {
                assert!(
                    !body.contains(pattern),
                    "{shown} contains the absolute path `{pattern}`"
                );
            }
            checked += 1;
        }
    }
    assert!(
        checked >= 7,
        "expected at least 7 fixture files, saw {checked}"
    );
}

#[test]
fn normalisation_helpers_are_surgical() {
    assert_eq!(
        zero_number_field(r#"{"durationMs":17,"exitCode":0}"#, "durationMs"),
        r#"{"durationMs":0,"exitCode":0}"#
    );
    // A field whose name merely ends with the key is left alone: the needle
    // carries the opening quote, so `totalDurationMs` is not a match.
    assert_eq!(
        zero_number_field(r#"{"totalDurationMs":9}"#, "durationMs"),
        r#"{"totalDurationMs":9}"#
    );
    assert_eq!(
        zero_number_field(r#"{"totalDurationMs":9}"#, "totalDurationMs"),
        r#"{"totalDurationMs":0}"#
    );
    // A non-numeric value after the key is left alone.
    assert_eq!(
        zero_number_field(r#"{"durationMs":null}"#, "durationMs"),
        r#"{"durationMs":null}"#
    );
    assert_eq!(
        mask_correlation_id(r#"{"correlation_id":"a6b7715d-b209-4edf-8048-36835134d12f"}"#),
        format!("{{\"correlation_id\":\"{CORRELATION_PLACEHOLDER}\"}}")
    );
    // Nothing to mask: the input survives byte for byte.
    assert_eq!(mask_correlation_id("plain output"), "plain output");
    assert_eq!(
        mask_project_root("at /tmp/x/DARE", Path::new("/tmp/x")),
        format!("at {ROOT_PLACEHOLDER}/DARE")
    );
}
