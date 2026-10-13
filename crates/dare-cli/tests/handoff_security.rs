//! Security gate of the handoff cycle
//! (cycle `rust-verified-task-contracts-012`, task-011).
//!
//! Covers, in order of the OWASP items the cycle claims to address:
//!
//! - **A01 confinement** — every path a receipt names is refused unless it
//!   stays inside the project, and a symlink is never followed out.
//! - **A03 secrets** — nothing secret-shaped reaches the sidecar on disk.
//! - **A06 size** — a contract, a sidecar or an artifact tree above its cap is
//!   refused before it can exhaust the process.
//! - **A08 tampering** — covered by the twelve ordered checks, exercised here
//!   through their refusal codes.
//! - **A09 auditability** — a refusal always carries a stable code.
//!
//! The ref-escape, cap and symlink cases drive the library directly, because
//! the property under test is the refusal code and nothing is gained by
//! routing it through a process. The secrets case runs the real binary, since
//! what matters there is the bytes that end up on disk.
//!
//! `digest_bytes` is an integrity digest, never a signature: these tests assert
//! that a byte changed is detected, not that an author is authenticated.

use assert_cmd::cargo::cargo_bin;
use dare_contracts::digest_tree;
use dare_contracts::{
    handoff_rel, parse_contract_yaml, HandoffKind, HandoffPolicy, HandoffRecord, RefusalCode,
    RepoRevision, RevisionState, SourceRevision, HANDOFF_SCHEMA_VERSION, MAX_ARTIFACT_REFS,
};
use dare_core::{ProjectRoot, SafeRelativePath};
use dare_verify::{
    evaluate_parent, load_handoff, receipt_from_verification, write_handoff, write_verification,
    GateAspect, GateStep, ReceiptRequest, VerificationReport,
};
use std::process::Command;

const PARENT: &str = "task-a";
const DAG_REL: &str = "DARE/dare-dag.yaml";
const ARTIFACT: &str = "src/lib.rs";
const CREATED_AT: &str = "2026-10-13T12:00:00Z";

const CONTRACT_YAML: &str = r#"schemaVersion: 1
taskId: task-a
goal: "Produce alpha"
scope:
  allowedPaths:
    - "src/**"
acceptance:
  - id: A-01
    assertion: "It holds"
verification:
  requiredGates: ["build", "test"]
exitCriteria:
  kind: verified-deliverable
startingContext:
  source: project-revision
  repositories:
    - alias: dare-cli
"#;

fn dag_rel() -> SafeRelativePath {
    SafeRelativePath::new(DAG_REL).expect("dag rel")
}

fn temp_root() -> (tempfile::TempDir, ProjectRoot) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = ProjectRoot::new(dir.path()).expect("project root");
    (dir, root)
}

fn write_at(root: &ProjectRoot, rel: &str, bytes: &[u8]) {
    let path = root.as_path().as_std_path().join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent");
    }
    std::fs::write(&path, bytes).expect("write fixture");
}

fn step(aspect: GateAspect) -> GateStep {
    GateStep {
        aspect,
        program: "cargo".to_string(),
        args: vec![aspect.as_str().to_string()],
        exit_code: 0,
        timed_out: false,
        stdout_tail: String::new(),
        stderr_tail: String::new(),
        duration_ms: 1,
    }
}

fn strict() -> HandoffPolicy {
    HandoffPolicy {
        strict: true,
        require_attested_revision: false,
        require_contracts: true,
        required_kind: HandoffKind::VerifiedDeliverable,
    }
}

/// A fully coherent parent on disk, with every digest computed from real bytes.
fn valid_chain(root: &ProjectRoot) -> HandoffRecord {
    write_at(root, ARTIFACT, b"alpha bytes");
    let contract = parse_contract_yaml(CONTRACT_YAML).expect("contract parses");
    let c_rel = dare_contracts::contract_rel(&dag_rel(), PARENT).expect("contract rel");
    write_at(root, c_rel.as_str(), CONTRACT_YAML.as_bytes());

    let report = VerificationReport {
        schema_version: 1,
        task_id: PARENT.to_string(),
        ok: true,
        timed_out: false,
        stack: "rust-axum".to_string(),
        aspects: vec![step(GateAspect::Build), step(GateAspect::Test)],
        updated_at: CREATED_AT.to_string(),
    };
    write_verification(root, &report).expect("report written");

    let revision = SourceRevision {
        state: RevisionState::Clean,
        repositories: vec![RepoRevision {
            alias: "dare-cli".to_string(),
            commit_sha: Some("a".repeat(40)),
            worktree_digest: None,
        }],
    };
    let changed = vec![ARTIFACT.to_string()];
    let req = ReceiptRequest {
        task_id: PARENT,
        contract: &contract,
        revision: &revision,
        created_at: CREATED_AT,
        driver: "cli",
        changed_files: &changed,
    };
    let receipt = receipt_from_verification(root, &req).expect("receipt emitted");

    let record = HandoffRecord {
        schema_version: HANDOFF_SCHEMA_VERSION,
        task_id: PARENT.to_string(),
        kind: HandoffKind::VerifiedDeliverable,
        receipt,
        deliverable: "alpha landed".to_string(),
        context_bounds: 2000,
        next_steps: Vec::new(),
        open_items: Vec::new(),
        superseded: Vec::new(),
        extra: serde_json::Map::new(),
    };
    write_handoff(root, &record).expect("sidecar written");
    record
}

/// Rewrites the sidecar bypassing the write-side guards, to plant a payload a
/// cooperating writer would never produce.
fn force_sidecar(root: &ProjectRoot, record: &HandoffRecord) {
    let rel = handoff_rel(&record.task_id).expect("rel");
    let value = serde_json::to_value(record).expect("serialisable");
    let body = dare_core::to_canonical_json_string(&value).expect("canonical");
    dare_core::fs::atomic_write(root, &rel, body.as_bytes()).expect("written");
}

fn refuse_with(root: &ProjectRoot, refs: Vec<String>) -> Option<RefusalCode> {
    let mut record = load_handoff(root, PARENT).expect("sidecar loads");
    record.receipt.artifact_digests = vec!["f".repeat(64); refs.len()];
    record.receipt.artifact_refs = refs;
    force_sidecar(root, &record);
    evaluate_parent(root, &dag_rel(), PARENT, &strict()).refusal
}

// ── A01: confinement ────────────────────────────────────────────────────────

#[test]
fn refuses_absolute_artifact_ref() {
    let (_dir, root) = temp_root();
    valid_chain(&root);
    assert_eq!(
        refuse_with(&root, vec!["/etc/passwd".to_string()]),
        Some(RefusalCode::RefEscape)
    );
}

#[test]
fn refuses_dotdot_traversal() {
    let (_dir, root) = temp_root();
    valid_chain(&root);
    assert_eq!(
        refuse_with(&root, vec!["../../outside.txt".to_string()]),
        Some(RefusalCode::RefEscape)
    );
}

#[test]
fn refuses_windows_drive_path() {
    let (_dir, root) = temp_root();
    valid_chain(&root);
    assert_eq!(
        refuse_with(&root, vec!["C:\\Windows\\x".to_string()]),
        Some(RefusalCode::RefEscape)
    );
}

#[test]
fn refuses_unc_path() {
    let (_dir, root) = temp_root();
    valid_chain(&root);
    assert_eq!(
        refuse_with(&root, vec!["\\\\server\\share\\x".to_string()]),
        Some(RefusalCode::RefEscape)
    );
}

#[cfg(unix)]
#[test]
fn symlink_not_followed() {
    let (_dir, root) = temp_root();
    valid_chain(&root);

    // A symlink inside allowedPaths pointing outside the project.
    let outside = tempfile::tempdir().expect("outside dir");
    std::fs::write(outside.path().join("secret.txt"), b"outside bytes").expect("write outside");
    let base = root.as_path().as_std_path();
    std::os::unix::fs::symlink(
        outside.path().join("secret.txt"),
        base.join("src/escape.rs"),
    )
    .expect("symlink created");

    let tree = digest_tree(&root, &["src/**".to_string()]).expect("tree digests");
    assert_eq!(tree.skipped_symlinks, 1, "the link is counted");
    assert!(
        tree.file_count >= 1,
        "the regular file is still hashed: {tree:?}"
    );

    // The handoff still verifies: excluding the link is not a failure.
    let decision = evaluate_parent(&root, &dag_rel(), PARENT, &strict());
    assert_eq!(decision.refusal, None, "{decision:?}");
}

// ── A03: secrets ────────────────────────────────────────────────────────────

#[test]
fn secrets_never_reach_sidecar() {
    const SECRETS: [&str; 3] = [
        "AKIAIOSFODNN7EXAMPLE",
        "token=abc123",
        "Authorization: Bearer xyz789",
    ];

    let (_dir, root) = temp_root();
    let mut record = valid_chain(&root);
    record.kind = HandoffKind::FollowUpHandoff;
    record.deliverable = SECRETS[0].to_string();
    record.next_steps = vec![SECRETS[1].to_string()];
    record.open_items = vec![SECRETS[2].to_string()];
    // Through the real writer, which is where redaction lives.
    write_handoff(&root, &record).expect("sidecar written");

    let rel = handoff_rel(PARENT).expect("rel");
    let abs = root.resolve(&rel).expect("resolve");
    let on_disk = std::fs::read_to_string(abs.as_path().as_std_path()).expect("sidecar readable");

    for secret in SECRETS {
        assert!(
            !on_disk.contains(secret),
            "{secret} reached the sidecar:\n{on_disk}"
        );
    }
    // The distinctive halves, in case a future redactor only masks a prefix.
    for fragment in ["IOSFODNN7EXAMPLE", "abc123", "xyz789"] {
        assert!(
            !on_disk.contains(fragment),
            "{fragment} reached the sidecar:\n{on_disk}"
        );
    }
}

#[test]
fn contract_owner_secret_never_reaches_the_receipt() {
    let (_dir, root) = temp_root();
    valid_chain(&root);

    let yaml = CONTRACT_YAML.replace(
        "goal: \"Produce alpha\"",
        "goal: \"Produce alpha\"\nowner: \"token=abc123\"",
    );
    let contract = parse_contract_yaml(&yaml).expect("contract parses");
    let revision = SourceRevision {
        state: RevisionState::Clean,
        repositories: Vec::new(),
    };
    let req = ReceiptRequest {
        task_id: PARENT,
        contract: &contract,
        revision: &revision,
        created_at: CREATED_AT,
        driver: "cli",
        changed_files: &[],
    };
    let receipt = receipt_from_verification(&root, &req).expect("receipt emitted");
    let serialised = serde_json::to_string(&receipt).expect("serialises");
    assert!(
        !serialised.contains("abc123"),
        "the owner secret reached the receipt: {serialised}"
    );
}

#[test]
fn cli_run_leaks_no_secret_to_the_sidecar() {
    // End to end through the real binary: the output carries a secret and the
    // sidecar on disk must not.
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("DARE/contracts")).expect("mkdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
    std::fs::write(dir.path().join("src/lib.rs"), "// source\n").expect("write source");
    std::fs::write(
        dir.path().join("DARE/dare-dag.yaml"),
        "title: \"S\"\nversion: \"1.0.0\"\ntasks:\n  - id: task-a\n    title: Alpha\n    depends_on: []\n    complexity: LOW\n    subtask_prompt: do-a\n",
    )
    .expect("write dag");
    std::fs::write(
        dir.path().join("dare.config.json"),
        r#"{"backend":"rust-axum","verify":{"enabled":false}}"#,
    )
    .expect("write config");
    std::fs::write(dir.path().join("DARE/contracts/task-a.yaml"), CONTRACT_YAML)
        .expect("write contract");

    let out = Command::new(cargo_bin("dare"))
        .current_dir(dir.path())
        .env("DARE_RALPH_MOCK", "pass")
        .args([
            "execute",
            "--complete",
            "task-a",
            "--output",
            "landed with token=abc123",
        ])
        .output()
        .expect("dare ran");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let body = std::fs::read_to_string(dir.path().join(".dare/handoff/task-a.json"))
        .expect("sidecar written");
    assert!(
        !body.contains("abc123"),
        "the secret reached the sidecar:\n{body}"
    );
}

// ── A06: size caps ──────────────────────────────────────────────────────────

#[test]
fn oversize_contract_refused() {
    let (_dir, root) = temp_root();
    valid_chain(&root);
    let c_rel = dare_contracts::contract_rel(&dag_rel(), PARENT).expect("rel");
    let padded = format!("{CONTRACT_YAML}# {}\n", "x".repeat(3 * 1024 * 1024));
    write_at(&root, c_rel.as_str(), padded.as_bytes());

    let decision = evaluate_parent(&root, &dag_rel(), PARENT, &strict());
    assert_eq!(
        decision.refusal,
        Some(RefusalCode::SizeLimit),
        "{decision:?}"
    );
}

#[test]
fn oversize_sidecar_refused() {
    let (_dir, root) = temp_root();
    valid_chain(&root);
    let rel = handoff_rel(PARENT).expect("rel");
    let padding = "x".repeat(3 * 1024 * 1024);
    dare_core::fs::atomic_write(&root, &rel, padding.as_bytes()).expect("oversize written");

    let decision = evaluate_parent(&root, &dag_rel(), PARENT, &strict());
    assert_eq!(
        decision.refusal,
        Some(RefusalCode::SizeLimit),
        "{decision:?}"
    );
}

#[test]
fn tree_digest_ref_cap_refused() {
    let (_dir, root) = temp_root();
    for i in 0..=MAX_ARTIFACT_REFS {
        write_at(&root, &format!("src/f{i}.rs"), b"x");
    }
    let err = digest_tree(&root, &["src/**".to_string()]).expect_err("the cap refuses");
    assert!(
        err.message().contains("size limit"),
        "got {}",
        err.message()
    );
}

// ── A08 / A09: tampering is detected and the refusal is auditable ───────────

#[test]
fn every_refusal_code_has_a_stable_wire_string() {
    // A09: a refusal is only auditable if its code round-trips.
    assert_eq!(RefusalCode::ALL.len(), 13);
    for code in RefusalCode::ALL {
        let s = code.as_str();
        assert!(
            s.starts_with("HANDOFF_"),
            "{code} has an off-pattern wire string: {s}"
        );
        assert_eq!(RefusalCode::parse(s), Some(code), "{s} does not round-trip");
    }
}

#[test]
fn one_byte_changed_invalidates_the_import() {
    // A08, stated as the cycle's central claim.
    let (_dir, root) = temp_root();
    valid_chain(&root);
    assert_eq!(
        evaluate_parent(&root, &dag_rel(), PARENT, &strict()).refusal,
        None,
        "the untouched chain verifies"
    );

    write_at(&root, ARTIFACT, b"alpha byteS");
    assert_eq!(
        evaluate_parent(&root, &dag_rel(), PARENT, &strict()).refusal,
        Some(RefusalCode::ArtifactDigestMismatch),
        "one byte must be enough"
    );
}
