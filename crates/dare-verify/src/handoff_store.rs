//! Sidecar persistence for [`HandoffRecord`] (RF-04, RF-08, RF-10, D-02).
//!
//! The record lives in its own file, `.dare/handoff/<task-id>.json`, rather
//! than inside `.dare/state.json`. The npm runtime rebuilds the state file from
//! the fields it knows and drops the rest, so a sidecar in a separate file
//! survives a legacy writer by construction.
//!
//! Writes carry the same guarantees as the state file: fail-fast lock, atomic
//! replace, canonical JSON and redaction before serialisation. Lock contention
//! is deliberately not queued, retried or backed off — `CoreError::io("file
//! lock held")` propagates, which is the documented contract of
//! `.dare/state.json`.

use dare_contracts::{
    digest_bytes, from_json_slice, handoff_rel, read_limited, HandoffKind, HandoffRecord,
    SupersededReceipt, HANDOFF_SCHEMA_VERSION, MAX_SUPERSEDED,
};
use dare_core::fs::{atomic_write, FileLock};
use dare_core::{redact, to_canonical_json_string, CoreError, CoreResult, ProjectRoot};

/// Persists a handoff record, atomically and under an exclusive lock.
///
/// The order of operations is normative: the record is rejected before any
/// path is touched, redacted before it is serialised, and only then written.
///
/// # Errors
///
/// - [`CoreError::InvalidInput`] for a task id that is not a safe single path
///   segment.
/// - [`CoreError::Config`] for an unsupported `schema_version`, or for a
///   follow-up handoff that declares no open item — a follow-up without a
///   pending item is a `DONE` in disguise (D-05).
/// - [`CoreError::Io`] carrying `"file lock held"` when another writer holds
///   the lock. This is fail-fast by design.
pub fn write_handoff(root: &ProjectRoot, record: &HandoffRecord) -> CoreResult<()> {
    let rel = handoff_rel(&record.task_id)?;

    if record.schema_version != HANDOFF_SCHEMA_VERSION {
        return Err(CoreError::config("unsupported handoff schema version"));
    }
    if record.kind == HandoffKind::FollowUpHandoff && record.open_items.is_empty() {
        return Err(CoreError::config(
            "follow-up handoff requires at least one open item",
        ));
    }

    let _lock = FileLock::try_acquire(root, &rel)?;
    write_body(root, &rel, record)
}

/// Reads a handoff record, refusing anything above the 2 MiB read cap.
///
/// # Errors
///
/// [`CoreError::Config`] with `"unsupported handoff schema version"` when the
/// file on disk is not schema 1, and with `"size limit"` when it exceeds the
/// read cap.
pub fn load_handoff(root: &ProjectRoot, task_id: &str) -> CoreResult<HandoffRecord> {
    let rel = handoff_rel(task_id)?;
    let bytes = read_limited(root, &rel)?;
    let record: HandoffRecord = from_json_slice(&bytes)?;
    if record.schema_version != HANDOFF_SCHEMA_VERSION {
        return Err(CoreError::config("unsupported handoff schema version"));
    }
    Ok(record)
}

/// Whether a task has a handoff sidecar, without deserialising it.
pub fn handoff_exists(root: &ProjectRoot, task_id: &str) -> CoreResult<bool> {
    let rel = handoff_rel(task_id)?;
    Ok(root.resolve(&rel)?.as_path().exists())
}

/// Records that the current receipt was replaced, keeping its history.
///
/// Idempotent by design: a task with no sidecar yields `Ok(())`, so a reset
/// path can call this unconditionally. The history is FIFO-capped at
/// [`MAX_SUPERSEDED`], discarding the oldest entry, so a task re-run many times
/// cannot grow the sidecar without bound.
///
/// This is what makes re-execution non-destructive (RF-03): nothing replaces a
/// receipt without leaving the previous one's digest behind.
pub fn supersede_handoff(
    root: &ProjectRoot,
    task_id: &str,
    reason: &str,
    at: &str,
) -> CoreResult<()> {
    let rel = handoff_rel(task_id)?;
    if !root.resolve(&rel)?.as_path().exists() {
        return Ok(());
    }

    let _lock = FileLock::try_acquire(root, &rel)?;

    let bytes = read_limited(root, &rel)?;
    let mut record: HandoffRecord = from_json_slice(&bytes)?;
    if record.schema_version != HANDOFF_SCHEMA_VERSION {
        return Err(CoreError::config("unsupported handoff schema version"));
    }

    let receipt_value = serde_json::to_value(&record.receipt)
        .map_err(|e| CoreError::internal(format!("receipt is not serializable: {e}")))?;
    let receipt_digest = digest_bytes(to_canonical_json_string(&receipt_value)?.as_bytes());

    record.superseded.push(SupersededReceipt {
        receipt_digest,
        created_at: at.to_string(),
        reason: redact(reason),
    });
    while record.superseded.len() > MAX_SUPERSEDED {
        record.superseded.remove(0);
    }

    write_body(root, &rel, &record)
}

/// Redacts, serialises canonically and atomically replaces the sidecar.
///
/// Assumes the caller holds the lock, so both write paths can share it without
/// re-acquiring — a second acquisition in the same process would contend with
/// itself.
fn write_body(
    root: &ProjectRoot,
    rel: &dare_core::SafeRelativePath,
    record: &HandoffRecord,
) -> CoreResult<()> {
    let redacted = redacted_record(record);
    let value = serde_json::to_value(&redacted)
        .map_err(|e| CoreError::internal(format!("handoff is not serializable: {e}")))?;
    let body = to_canonical_json_string(&value)?;
    atomic_write(root, rel, body.as_bytes())
}

/// Clone of `record` with every free-text field redacted.
///
/// A clone, never the caller's record: the caller keeps its own values intact,
/// exactly as `write_verification` does for the stdio tails.
fn redacted_record(record: &HandoffRecord) -> HandoffRecord {
    let mut out = record.clone();
    out.deliverable = redact(&out.deliverable);
    out.next_steps = out.next_steps.iter().map(|s| redact(s)).collect();
    out.open_items = out.open_items.iter().map(|s| redact(s)).collect();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_contracts::{
        Producer, RepoRevision, RevisionState, SourceRevision, Verdict, VerificationReceipt,
    };
    use dare_core::SafeRelativePath;
    use tempfile::tempdir;

    const AT: &str = "2026-10-13T12:00:00Z";
    const SECRET: &str = "token=abc123";

    fn temp_root() -> (tempfile::TempDir, ProjectRoot) {
        let dir = tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("project root");
        (dir, root)
    }

    fn receipt() -> VerificationReceipt {
        VerificationReceipt {
            schema_version: HANDOFF_SCHEMA_VERSION,
            task_id: "task-006".to_string(),
            contract_digest: "a".repeat(64),
            contract_full_digest: "b".repeat(64),
            source_revision: SourceRevision {
                state: RevisionState::Clean,
                repositories: vec![RepoRevision {
                    alias: "dare-cli".to_string(),
                    commit_sha: Some("0".repeat(40)),
                    worktree_digest: None,
                }],
            },
            artifact_refs: vec!["crates/dare-verify/src/handoff_store.rs".to_string()],
            artifact_digests: vec!["c".repeat(64)],
            verification_ref: ".dare/verification/task-006.json".to_string(),
            verification_digest: "d".repeat(64),
            verdict: Verdict::Passed,
            created_at: AT.to_string(),
            producer: Producer {
                cli_version: "4.0.0".to_string(),
                driver: "cli".to_string(),
            },
        }
    }

    fn record(kind: HandoffKind) -> HandoffRecord {
        HandoffRecord {
            schema_version: HANDOFF_SCHEMA_VERSION,
            task_id: "task-006".to_string(),
            kind,
            receipt: receipt(),
            deliverable: "sidecar store landed".to_string(),
            context_bounds: 2000,
            next_steps: vec!["task-007 consumes this".to_string()],
            open_items: match kind {
                HandoffKind::FollowUpHandoff => vec!["wire the CLI".to_string()],
                HandoffKind::VerifiedDeliverable => Vec::new(),
            },
            superseded: Vec::new(),
            extra: serde_json::Map::new(),
        }
    }

    fn on_disk(root: &ProjectRoot, task_id: &str) -> String {
        let rel = handoff_rel(task_id).expect("rel");
        let abs = root.resolve(&rel).expect("resolve");
        std::fs::read_to_string(abs.as_path().as_std_path()).expect("sidecar readable")
    }

    // --- write / load -----------------------------------------------------

    #[test]
    fn write_load_roundtrip() {
        let (_dir, root) = temp_root();
        let r = record(HandoffKind::VerifiedDeliverable);
        write_handoff(&root, &r).expect("written");
        assert_eq!(load_handoff(&root, "task-006").expect("loaded"), r);

        // Canonical key order on disk: `kind` precedes `schemaVersion`.
        let body = on_disk(&root, "task-006");
        let kind_at = body.find("\"kind\"").expect("kind present");
        let schema_at = body
            .find("\"schemaVersion\"")
            .expect("schemaVersion present");
        assert!(
            kind_at < schema_at,
            "keys are not in canonical order: {body}"
        );
    }

    #[test]
    fn write_redacts_deliverable() {
        let (_dir, root) = temp_root();
        let mut r = record(HandoffKind::VerifiedDeliverable);
        r.deliverable = SECRET.to_string();
        write_handoff(&root, &r).expect("written");

        let body = on_disk(&root, "task-006");
        assert!(!body.contains("abc123"), "secret reached disk: {body}");
        assert_eq!(
            load_handoff(&root, "task-006").expect("loaded").deliverable,
            redact(SECRET)
        );
        assert_eq!(r.deliverable, SECRET, "the caller's record was mutated");
    }

    #[test]
    fn write_redacts_next_steps_and_open_items() {
        let (_dir, root) = temp_root();
        let mut r = record(HandoffKind::FollowUpHandoff);
        r.next_steps = vec![SECRET.to_string(), "plain step".to_string()];
        r.open_items = vec![SECRET.to_string()];
        write_handoff(&root, &r).expect("written");

        let body = on_disk(&root, "task-006");
        assert!(!body.contains("abc123"), "secret reached disk: {body}");
        let loaded = load_handoff(&root, "task-006").expect("loaded");
        assert_eq!(
            loaded.next_steps,
            vec![redact(SECRET), "plain step".to_string()]
        );
        assert_eq!(loaded.open_items, vec![redact(SECRET)]);
        assert_eq!(
            r.open_items,
            vec![SECRET.to_string()],
            "caller's record mutated"
        );
    }

    #[test]
    fn write_rejects_unsafe_id() {
        let (_dir, root) = temp_root();
        for bad in ["a/b", "../x", "", ".hidden"] {
            let mut r = record(HandoffKind::VerifiedDeliverable);
            r.task_id = bad.to_string();
            let err = write_handoff(&root, &r).expect_err("unsafe id refused");
            assert!(
                matches!(err, CoreError::InvalidInput(_)),
                "id {bad:?} gave {err:?}"
            );
        }
    }

    #[test]
    fn write_rejects_bad_schema_version() {
        let (_dir, root) = temp_root();
        let mut r = record(HandoffKind::VerifiedDeliverable);
        r.schema_version = 2;
        let err = write_handoff(&root, &r).expect_err("schema 2 refused");
        assert!(matches!(err, CoreError::Config(_)), "got {err:?}");
    }

    #[test]
    fn write_rejects_followup_without_open_items() {
        let (_dir, root) = temp_root();
        let mut r = record(HandoffKind::FollowUpHandoff);
        r.open_items = Vec::new();
        let err = write_handoff(&root, &r).expect_err("follow-up without pending refused");
        assert!(
            err.message().contains("requires at least one open item"),
            "got {err:?}"
        );
        assert!(matches!(err, CoreError::Config(_)), "got {err:?}");
    }

    #[test]
    fn load_rejects_oversize() {
        let (_dir, root) = temp_root();
        let rel = handoff_rel("task-006").expect("rel");
        let padding = "x".repeat(2 * 1024 * 1024 + 1);
        atomic_write(&root, &rel, padding.as_bytes()).expect("oversize written");
        let err = load_handoff(&root, "task-006").expect_err("oversize refused");
        assert!(err.message().contains("size limit"), "got {err:?}");
        assert!(matches!(err, CoreError::Config(_)), "got {err:?}");
    }

    #[test]
    fn load_rejects_bad_schema_version() {
        let (_dir, root) = temp_root();
        let r = record(HandoffKind::VerifiedDeliverable);
        write_handoff(&root, &r).expect("written");

        // Rewrite with schema 2 directly, bypassing the write-side guard.
        let rel = handoff_rel("task-006").expect("rel");
        let body = on_disk(&root, "task-006").replace("\"schemaVersion\":1", "\"schemaVersion\":2");
        atomic_write(&root, &rel, body.as_bytes()).expect("rewritten");

        let err = load_handoff(&root, "task-006").expect_err("schema 2 refused");
        assert!(
            err.message().contains("unsupported handoff schema version"),
            "got {err:?}"
        );
    }

    #[test]
    fn handoff_exists_without_deserializing() {
        let (_dir, root) = temp_root();
        assert!(!handoff_exists(&root, "task-006").expect("checked"));

        // Deliberately unparseable: existence must not require deserialisation.
        let rel = handoff_rel("task-006").expect("rel");
        atomic_write(&root, &rel, b"{ not json").expect("written");
        assert!(handoff_exists(&root, "task-006").expect("checked"));
        assert!(load_handoff(&root, "task-006").is_err());
    }

    #[test]
    fn write_fails_fast_when_lock_held() {
        let (_dir, root) = temp_root();
        let rel = handoff_rel("task-006").expect("rel");
        let held = FileLock::try_acquire(&root, &rel).expect("first lock");

        let err = write_handoff(&root, &record(HandoffKind::VerifiedDeliverable))
            .expect_err("contended write refuses");
        assert!(matches!(err, CoreError::Io(_)), "got {err:?}");
        assert!(err.message().contains("lock"), "got {err:?}");

        drop(held);
        write_handoff(&root, &record(HandoffKind::VerifiedDeliverable))
            .expect("write succeeds once the lock is free");
    }

    // --- supersede --------------------------------------------------------

    #[test]
    fn supersede_missing_is_ok() {
        let (_dir, root) = temp_root();
        assert_eq!(
            supersede_handoff(&root, "task-006", "reset", AT),
            Ok(()),
            "absence is idempotent so a reset path needs no conditional"
        );
        assert!(!handoff_exists(&root, "task-006").expect("checked"));
    }

    #[test]
    fn supersede_twice_accumulates() {
        let (_dir, root) = temp_root();
        write_handoff(&root, &record(HandoffKind::VerifiedDeliverable)).expect("written");

        supersede_handoff(&root, "task-006", "first", AT).expect("superseded");
        supersede_handoff(&root, "task-006", "second", AT).expect("superseded");

        let loaded = load_handoff(&root, "task-006").expect("loaded");
        assert_eq!(loaded.superseded.len(), 2);
        assert_eq!(loaded.superseded[0].reason, "first");
        assert_eq!(loaded.superseded[1].reason, "second");
        assert_eq!(loaded.superseded[0].receipt_digest.len(), 64);
    }

    #[test]
    fn supersede_fifo_cap() {
        let (_dir, root) = temp_root();
        write_handoff(&root, &record(HandoffKind::VerifiedDeliverable)).expect("written");

        for i in 0..=MAX_SUPERSEDED {
            supersede_handoff(&root, "task-006", &format!("reason-{i}"), AT).expect("superseded");
        }

        let loaded = load_handoff(&root, "task-006").expect("loaded");
        assert_eq!(loaded.superseded.len(), MAX_SUPERSEDED);
        assert_eq!(
            loaded.superseded[0].reason, "reason-1",
            "the oldest entry is the one discarded"
        );
        assert_eq!(
            loaded.superseded[MAX_SUPERSEDED - 1].reason,
            format!("reason-{MAX_SUPERSEDED}")
        );
    }

    #[test]
    fn supersede_redacts_reason() {
        let (_dir, root) = temp_root();
        write_handoff(&root, &record(HandoffKind::VerifiedDeliverable)).expect("written");
        supersede_handoff(&root, "task-006", SECRET, AT).expect("superseded");

        let body = on_disk(&root, "task-006");
        assert!(!body.contains("abc123"), "secret reached disk: {body}");
        let loaded = load_handoff(&root, "task-006").expect("loaded");
        assert_eq!(loaded.superseded[0].reason, redact(SECRET));
    }

    #[test]
    fn supersede_preserves_the_receipt() {
        let (_dir, root) = temp_root();
        let r = record(HandoffKind::VerifiedDeliverable);
        write_handoff(&root, &r).expect("written");
        supersede_handoff(&root, "task-006", "re-run", AT).expect("superseded");

        let loaded = load_handoff(&root, "task-006").expect("loaded");
        assert_eq!(
            loaded.receipt, r.receipt,
            "superseding records history, it does not drop the receipt"
        );
    }

    #[test]
    fn supersede_rejects_unsafe_id() {
        let (_dir, root) = temp_root();
        let err =
            supersede_handoff(&root, "../escape", "reset", AT).expect_err("unsafe id refused");
        assert!(matches!(err, CoreError::InvalidInput(_)), "got {err:?}");
    }

    #[test]
    fn sidecar_path_is_confined() {
        let rel = handoff_rel("task-006").expect("rel");
        assert_eq!(rel.as_str(), ".dare/handoff/task-006.json");
        assert!(SafeRelativePath::new("../escape").is_err());
    }
}
