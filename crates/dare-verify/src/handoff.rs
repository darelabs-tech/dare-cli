//! Receipt emission from on-disk verification evidence (RF-03, RF-09).
//!
//! The central control of this module is what it refuses to accept.
//! [`receipt_from_verification`] takes no report, no `bool` and no
//! `&VerificationReport`: it reads `.dare/verification/<task_id>.json` from disk
//! itself. A caller that could hand in its own report would be able to build an
//! `ok: true` in memory and get a valid receipt back — the circular proof §7/A04
//! of the DESIGN forbids. Reading from disk forces the evidence to have been
//! produced and persisted before a receipt can exist.
//!
//! The aggregate `report.ok` is likewise treated as convenience only. Gate
//! satisfaction is proven per aspect by [`gates_satisfied`].

use dare_contracts::{
    digest_file, digest_tree, full_digest, material_digest, Producer, RepoRevision, RevisionState,
    SourceRevision, TaskContract, Verdict, VerificationReceipt, HANDOFF_SCHEMA_VERSION,
    MAX_ARTIFACT_REFS,
};
use dare_core::{redact, CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use globset::{Glob, GlobSetBuilder};

use crate::ralph::GateAspect;
use crate::verification::{load_verification, VerificationReport, VERIFICATION_DIR_REL};

/// Alias used for the worktree digest when neither the revision nor the
/// contract names a repository.
const DEFAULT_REPO_ALIAS: &str = ".";

/// Everything needed to emit a receipt, except the evidence itself.
///
/// The evidence is deliberately absent: it is read from disk inside
/// [`receipt_from_verification`].
///
/// `created_at` is a `&str` because the `Clock` trait lives in `dare-dag`, not
/// in `dare-core`, so the caller injects the timestamp — the same pattern
/// [`crate::verification::verification_from_ralph`] already follows.
///
/// `changed_files` carries the project-relative paths the caller observed as
/// changed (typically from `dare_project::git::changed_files`). The A-06
/// strategy for `Clean` and `Dirty` revisions is defined in terms of
/// "caller-supplied changed files", and this is how they are supplied; it is
/// plain path data, never evidence of success, so it cannot reintroduce the
/// circular proof the report-shaped parameters would.
#[derive(Debug, Clone)]
pub struct ReceiptRequest<'a> {
    pub task_id: &'a str,
    pub contract: &'a TaskContract,
    pub revision: &'a SourceRevision,
    pub created_at: &'a str,
    pub driver: &'a str,
    pub changed_files: &'a [String],
}

/// Proves every required gate actually passed, per aspect.
///
/// `Ok(())` if and only if for every `a` in `required` there is a step in
/// `report.aspects` with `step.aspect == a`, `step.exit_code == 0` and
/// `!step.timed_out`.
///
/// `report.ok` is never consulted. A report claiming `ok == true` while a
/// required aspect is missing from `aspects` returns `Err`: the aggregate is
/// convenience, the per-aspect steps are the proof. A duplicated aspect needs
/// only one passing step. `Err` lists the unsatisfied aspects in the order they
/// appear in `required`.
pub fn gates_satisfied(
    report: &VerificationReport,
    required: &[GateAspect],
) -> Result<(), Vec<GateAspect>> {
    let unsatisfied: Vec<GateAspect> = required
        .iter()
        .copied()
        .filter(|aspect| {
            !report
                .aspects
                .iter()
                .any(|step| step.aspect == *aspect && step.exit_code == 0 && !step.timed_out)
        })
        .collect();
    if unsatisfied.is_empty() {
        Ok(())
    } else {
        Err(unsatisfied)
    }
}

/// Emits a [`VerificationReceipt`] from the evidence already on disk.
///
/// The report is read through [`load_verification`], never received as an
/// argument — see the module docs.
///
/// # Errors
///
/// - [`CoreError::NotFound`] when no report exists for the task. The caller
///   maps that to `Verdict::Insufficient` and emits no receipt.
/// - [`CoreError::Config`] when `report.ok == false`, carrying
///   `"refusing to emit receipt for failed verification"`. A report that timed
///   out has `ok == false` and takes this same path.
/// - [`CoreError::Config`] propagated from the digest size caps.
pub fn receipt_from_verification(
    root: &ProjectRoot,
    req: &ReceiptRequest<'_>,
) -> CoreResult<VerificationReceipt> {
    let verification_ref = format!("{VERIFICATION_DIR_REL}/{}.json", req.task_id);
    let verification_rel = SafeRelativePath::new(&verification_ref)?;

    // `load_verification` reports a missing file as `Io`. The contract of this
    // function is `NotFound`, because that is what the caller translates into
    // `Verdict::Insufficient` without emitting a receipt, so the absence is
    // classified here before reading.
    if !root.resolve(&verification_rel)?.as_path().exists() {
        return Err(CoreError::not_found(format!(
            "verification report not found: {verification_ref}"
        )));
    }

    let report = load_verification(root, req.task_id)?;
    if !report.ok {
        return Err(CoreError::config(
            "refusing to emit receipt for failed verification",
        ));
    }

    let verification_digest = digest_file(root, &verification_rel)?;

    // A secret pasted into `owner` must not travel into the emitted digests.
    // `owner` is outside the material projection, so redacting it moves
    // `contract_full_digest` only — `contract_digest` is unaffected by design.
    let contract = redacted_owner_contract(req.contract);
    let contract_digest = material_digest(&contract)?;
    let contract_full_digest = full_digest(&contract)?;

    let allowed = &contract.scope.allowed_paths;
    let (artifact_refs, artifact_digests) = match req.revision.state {
        RevisionState::Clean | RevisionState::Dirty => {
            artifacts_from_changed(root, req.changed_files, allowed)?
        }
        // Without an attested revision there is no changed-file set to trust,
        // so the tree digest alone stands in for the artifacts.
        RevisionState::Unattested => (Vec::new(), Vec::new()),
    };

    let mut source_revision = req.revision.clone();
    if matches!(
        req.revision.state,
        RevisionState::Dirty | RevisionState::Unattested
    ) {
        let tree = digest_tree(root, allowed)?;
        place_worktree_digest(&mut source_revision, req.contract, tree.digest);
    }

    // Same index means the same file; a consumer compares them pairwise.
    if artifact_refs.len() != artifact_digests.len() {
        return Err(CoreError::internal(
            "artifact refs and digests diverged in length",
        ));
    }

    Ok(VerificationReceipt {
        schema_version: HANDOFF_SCHEMA_VERSION,
        task_id: req.task_id.to_string(),
        contract_digest,
        contract_full_digest,
        source_revision,
        artifact_refs,
        artifact_digests,
        verification_ref,
        verification_digest,
        // Only reachable once the gates passed, so the verdict is not a guess.
        verdict: Verdict::Passed,
        created_at: req.created_at.to_string(),
        producer: Producer {
            cli_version: env!("CARGO_PKG_VERSION").to_string(),
            driver: req.driver.to_string(),
        },
    })
}

/// Clone of `contract` with `owner` redacted, leaving every other field intact.
fn redacted_owner_contract(contract: &TaskContract) -> TaskContract {
    let mut out = contract.clone();
    out.owner = out.owner.as_deref().map(redact);
    out
}

/// Changed files confined to `allowed` globs, sorted, with a digest each.
///
/// A path that no longer exists is skipped: `changed_files` includes deletions,
/// and a deleted file has no bytes left to attest.
fn artifacts_from_changed(
    root: &ProjectRoot,
    changed: &[String],
    allowed: &[String],
) -> CoreResult<(Vec<String>, Vec<String>)> {
    if changed.is_empty() || allowed.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in allowed {
        let glob = Glob::new(pattern).map_err(|e| {
            CoreError::invalid_input(format!("invalid allowed path {pattern}: {e}"))
        })?;
        builder.add(glob);
    }
    let set = builder
        .build()
        .map_err(|e| CoreError::internal(format!("glob set did not build: {e}")))?;

    let mut refs: Vec<String> = changed
        .iter()
        .map(|p| p.replace('\\', "/"))
        .filter(|p| set.is_match(p.as_str()))
        .collect();
    refs.sort();
    refs.dedup();

    if refs.len() > MAX_ARTIFACT_REFS {
        return Err(CoreError::config("artifact refs exceed size limit"));
    }

    let mut kept = Vec::with_capacity(refs.len());
    let mut digests = Vec::with_capacity(refs.len());
    for rel_str in refs {
        let rel = SafeRelativePath::new(&rel_str)?;
        match digest_file(root, &rel) {
            Ok(digest) => {
                kept.push(rel_str);
                digests.push(digest);
            }
            Err(CoreError::NotFound(_)) => continue,
            Err(e) => return Err(e),
        }
    }
    Ok((kept, digests))
}

/// Stores `digest` on the repository revision the tree belongs to.
///
/// Selection is deterministic: the revision whose alias matches the contract's
/// first repository, else the only revision present, else the
/// lexicographically smallest alias. When the revision names no repository at
/// all one is appended, so an unattested run still carries its tree digest.
fn place_worktree_digest(revision: &mut SourceRevision, contract: &TaskContract, digest: String) {
    let primary = contract
        .starting_context
        .repositories
        .first()
        .map(|r| r.alias.as_str());

    if revision.repositories.is_empty() {
        revision.repositories.push(RepoRevision {
            alias: primary.unwrap_or(DEFAULT_REPO_ALIAS).to_string(),
            commit_sha: None,
            worktree_digest: Some(digest),
        });
        return;
    }

    let index = primary
        .and_then(|alias| revision.repositories.iter().position(|r| r.alias == alias))
        .or_else(|| (revision.repositories.len() == 1).then_some(0))
        .or_else(|| {
            revision
                .repositories
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.alias.cmp(&b.alias))
                .map(|(i, _)| i)
        });

    if let Some(i) = index {
        revision.repositories[i].worktree_digest = Some(digest);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ralph::GateStep;
    use crate::verification::write_verification;
    use dare_contracts::{parse_contract_yaml, RepoRef};
    use tempfile::tempdir;

    const CREATED_AT: &str = "2026-10-13T12:00:00Z";

    const CONTRACT_YAML: &str = r#"
schemaVersion: 1
taskId: task-005
goal: "Emit the receipt"
scope:
  allowedPaths:
    - "crates/dare-verify/src/**"
acceptance:
  - id: A-01
    assertion: "The receipt is emitted"
verification:
  requiredGates: ["build", "test"]
exitCriteria:
  kind: verified-deliverable
startingContext:
  source: project-revision
  repositories:
    - alias: dare-cli
"#;

    fn contract() -> TaskContract {
        parse_contract_yaml(CONTRACT_YAML).expect("contract parses")
    }

    fn step(aspect: GateAspect, exit_code: i32, timed_out: bool) -> GateStep {
        GateStep {
            aspect,
            program: "cargo".to_string(),
            args: vec![aspect.as_str().to_string()],
            exit_code,
            timed_out,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
            duration_ms: 1,
        }
    }

    fn report(ok: bool, aspects: Vec<GateStep>) -> VerificationReport {
        VerificationReport {
            schema_version: 1,
            task_id: "task-005".to_string(),
            ok,
            timed_out: false,
            stack: "rust".to_string(),
            aspects,
            updated_at: CREATED_AT.to_string(),
        }
    }

    fn temp_root() -> (tempfile::TempDir, ProjectRoot) {
        let dir = tempdir().expect("tempdir");
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

    fn revision(state: RevisionState, repos: Vec<RepoRevision>) -> SourceRevision {
        SourceRevision {
            state,
            repositories: repos,
        }
    }

    fn dare_cli_repo(commit: Option<&str>) -> RepoRevision {
        RepoRevision {
            alias: "dare-cli".to_string(),
            commit_sha: commit.map(str::to_string),
            worktree_digest: None,
        }
    }

    // --- gates_satisfied --------------------------------------------------

    #[test]
    fn gates_all_pass() {
        let r = report(
            true,
            vec![
                step(GateAspect::Build, 0, false),
                step(GateAspect::Test, 0, false),
            ],
        );
        assert_eq!(
            gates_satisfied(&r, &[GateAspect::Build, GateAspect::Test]),
            Ok(())
        );
    }

    #[test]
    fn gates_missing_aspect_despite_ok_true() {
        // The anti-circularity test: the aggregate claims success, the
        // per-aspect evidence does not contain the required gate.
        let r = report(true, vec![step(GateAspect::Build, 0, false)]);
        assert_eq!(
            gates_satisfied(&r, &[GateAspect::Build, GateAspect::Test]),
            Err(vec![GateAspect::Test]),
            "report.ok must never stand in for a missing aspect"
        );
    }

    #[test]
    fn gates_failed_exit_code() {
        let r = report(true, vec![step(GateAspect::Build, 1, false)]);
        assert_eq!(
            gates_satisfied(&r, &[GateAspect::Build]),
            Err(vec![GateAspect::Build])
        );
    }

    #[test]
    fn gates_timed_out() {
        let r = report(true, vec![step(GateAspect::Build, 0, true)]);
        assert_eq!(
            gates_satisfied(&r, &[GateAspect::Build]),
            Err(vec![GateAspect::Build])
        );
    }

    #[test]
    fn gates_duplicate_one_passing() {
        let r = report(
            true,
            vec![
                step(GateAspect::Test, 1, false),
                step(GateAspect::Test, 0, false),
            ],
        );
        assert_eq!(gates_satisfied(&r, &[GateAspect::Test]), Ok(()));
    }

    #[test]
    fn gates_empty_required() {
        let r = report(false, Vec::new());
        assert_eq!(gates_satisfied(&r, &[]), Ok(()));
    }

    #[test]
    fn gates_err_follows_required_order() {
        let r = report(true, Vec::new());
        assert_eq!(
            gates_satisfied(&r, &[GateAspect::Lint, GateAspect::Build]),
            Err(vec![GateAspect::Lint, GateAspect::Build])
        );
    }

    // --- receipt_from_verification ----------------------------------------

    #[test]
    fn receipt_refuses_missing_report() {
        let (_dir, root) = temp_root();
        let c = contract();
        let rev = revision(RevisionState::Clean, vec![dare_cli_repo(None)]);
        let req = ReceiptRequest {
            task_id: "task-005",
            contract: &c,
            revision: &rev,
            created_at: CREATED_AT,
            driver: "cli",
            changed_files: &[],
        };
        let err = receipt_from_verification(&root, &req).expect_err("missing report refuses");
        assert!(
            matches!(err, CoreError::NotFound(_)),
            "the caller maps NotFound to Insufficient, so the kind is part of the contract; got {err:?}"
        );
    }

    #[test]
    fn receipt_refuses_failed_report() {
        let (_dir, root) = temp_root();
        write_verification(
            &root,
            &report(false, vec![step(GateAspect::Build, 1, false)]),
        )
        .expect("report written");
        let c = contract();
        let rev = revision(RevisionState::Clean, vec![dare_cli_repo(None)]);
        let req = ReceiptRequest {
            task_id: "task-005",
            contract: &c,
            revision: &rev,
            created_at: CREATED_AT,
            driver: "cli",
            changed_files: &[],
        };
        let err = receipt_from_verification(&root, &req).expect_err("failed report refuses");
        assert!(
            err.message().contains("refusing to emit receipt"),
            "got {err:?}"
        );
        assert!(matches!(err, CoreError::Config(_)), "got {err:?}");
    }

    #[test]
    fn receipt_digests_match_recomputed() {
        let (_dir, root) = temp_root();
        write_verification(
            &root,
            &report(true, vec![step(GateAspect::Build, 0, false)]),
        )
        .expect("report written");
        write_at(&root, "crates/dare-verify/src/handoff.rs", b"contents");
        let c = contract();
        let rev = revision(
            RevisionState::Clean,
            vec![dare_cli_repo(Some(&"a".repeat(40)))],
        );
        let changed = vec!["crates/dare-verify/src/handoff.rs".to_string()];
        let req = ReceiptRequest {
            task_id: "task-005",
            contract: &c,
            revision: &rev,
            created_at: CREATED_AT,
            driver: "cli",
            changed_files: &changed,
        };
        let receipt = receipt_from_verification(&root, &req).expect("receipt emitted");

        // Recompute every digest independently of the emission path.
        let v_rel = SafeRelativePath::new(".dare/verification/task-005.json").expect("rel");
        assert_eq!(
            receipt.verification_digest,
            digest_file(&root, &v_rel).expect("digests")
        );
        assert_eq!(
            receipt.contract_digest,
            material_digest(&c).expect("digests")
        );
        assert_eq!(
            receipt.contract_full_digest,
            full_digest(&c).expect("digests")
        );
        let a_rel = SafeRelativePath::new("crates/dare-verify/src/handoff.rs").expect("rel");
        assert_eq!(
            receipt.artifact_digests,
            vec![digest_file(&root, &a_rel).expect("digests")]
        );
        assert_eq!(receipt.verdict, Verdict::Passed);
        assert_eq!(receipt.created_at, CREATED_AT);
        assert_eq!(receipt.producer.driver, "cli");
        assert_eq!(receipt.producer.cli_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn receipt_clean_revision_shape() {
        let (_dir, root) = temp_root();
        write_verification(
            &root,
            &report(true, vec![step(GateAspect::Build, 0, false)]),
        )
        .expect("report written");
        write_at(&root, "crates/dare-verify/src/handoff.rs", b"contents");
        let c = contract();
        let rev = revision(
            RevisionState::Clean,
            vec![dare_cli_repo(Some(&"b".repeat(40)))],
        );
        let changed = vec!["crates/dare-verify/src/handoff.rs".to_string()];
        let req = ReceiptRequest {
            task_id: "task-005",
            contract: &c,
            revision: &rev,
            created_at: CREATED_AT,
            driver: "cli",
            changed_files: &changed,
        };
        let receipt = receipt_from_verification(&root, &req).expect("receipt emitted");
        assert_eq!(receipt.artifact_refs.len(), 1);
        assert!(
            receipt
                .source_revision
                .repositories
                .iter()
                .all(|r| r.worktree_digest.is_none()),
            "a clean revision carries no worktree digest"
        );
    }

    #[test]
    fn receipt_dirty_revision_shape() {
        let (_dir, root) = temp_root();
        write_verification(
            &root,
            &report(true, vec![step(GateAspect::Build, 0, false)]),
        )
        .expect("report written");
        write_at(&root, "crates/dare-verify/src/handoff.rs", b"contents");
        let c = contract();
        let rev = revision(
            RevisionState::Dirty,
            vec![dare_cli_repo(Some(&"c".repeat(40)))],
        );
        let changed = vec!["crates/dare-verify/src/handoff.rs".to_string()];
        let req = ReceiptRequest {
            task_id: "task-005",
            contract: &c,
            revision: &rev,
            created_at: CREATED_AT,
            driver: "cli",
            changed_files: &changed,
        };
        let receipt = receipt_from_verification(&root, &req).expect("receipt emitted");
        assert_eq!(
            receipt.artifact_refs.len(),
            1,
            "dirty keeps the changed refs"
        );
        let repo = receipt
            .source_revision
            .repositories
            .iter()
            .find(|r| r.alias == "dare-cli")
            .expect("dare-cli revision present");
        let tree = digest_tree(&root, &c.scope.allowed_paths).expect("digests");
        assert_eq!(repo.worktree_digest.as_deref(), Some(tree.digest.as_str()));
    }

    #[test]
    fn receipt_unattested_revision_shape() {
        let (_dir, root) = temp_root();
        write_verification(
            &root,
            &report(true, vec![step(GateAspect::Build, 0, false)]),
        )
        .expect("report written");
        write_at(&root, "crates/dare-verify/src/handoff.rs", b"contents");
        let c = contract();
        // No git, so no repository revision at all.
        let rev = revision(RevisionState::Unattested, Vec::new());
        let changed = vec!["crates/dare-verify/src/handoff.rs".to_string()];
        let req = ReceiptRequest {
            task_id: "task-005",
            contract: &c,
            revision: &rev,
            created_at: CREATED_AT,
            driver: "cli",
            changed_files: &changed,
        };
        let receipt =
            receipt_from_verification(&root, &req).expect("unattested still emits a receipt");
        assert!(
            receipt.artifact_refs.is_empty(),
            "unattested carries no changed-file refs"
        );
        let tree = digest_tree(&root, &c.scope.allowed_paths).expect("digests");
        assert_eq!(
            receipt.source_revision.repositories.len(),
            1,
            "the tree digest needs somewhere to live"
        );
        assert_eq!(
            receipt.source_revision.repositories[0]
                .worktree_digest
                .as_deref(),
            Some(tree.digest.as_str())
        );
        assert_eq!(receipt.verdict, Verdict::Passed);
    }

    #[test]
    fn receipt_refs_and_digests_same_len() {
        let (_dir, root) = temp_root();
        write_verification(
            &root,
            &report(true, vec![step(GateAspect::Build, 0, false)]),
        )
        .expect("report written");
        for name in ["a.rs", "b.rs", "c.rs"] {
            write_at(&root, &format!("crates/dare-verify/src/{name}"), b"x");
        }
        let c = contract();
        let rev = revision(RevisionState::Clean, vec![dare_cli_repo(None)]);
        let changed = vec![
            "crates/dare-verify/src/c.rs".to_string(),
            "crates/dare-verify/src/a.rs".to_string(),
            "crates/dare-verify/src/b.rs".to_string(),
            "crates/dare-core/src/outside.rs".to_string(),
        ];
        let req = ReceiptRequest {
            task_id: "task-005",
            contract: &c,
            revision: &rev,
            created_at: CREATED_AT,
            driver: "cli",
            changed_files: &changed,
        };
        let receipt = receipt_from_verification(&root, &req).expect("receipt emitted");
        assert_eq!(receipt.artifact_refs.len(), receipt.artifact_digests.len());
        assert_eq!(
            receipt.artifact_refs,
            vec![
                "crates/dare-verify/src/a.rs",
                "crates/dare-verify/src/b.rs",
                "crates/dare-verify/src/c.rs",
            ],
            "refs are confined to allowedPaths and sorted"
        );
    }

    #[test]
    fn receipt_redacts_owner() {
        let (_dir, root) = temp_root();
        write_verification(
            &root,
            &report(true, vec![step(GateAspect::Build, 0, false)]),
        )
        .expect("report written");
        let mut c = contract();
        c.owner = Some("token=abc123".to_string());
        let rev = revision(RevisionState::Clean, vec![dare_cli_repo(None)]);
        let req = ReceiptRequest {
            task_id: "task-005",
            contract: &c,
            revision: &rev,
            created_at: CREATED_AT,
            driver: "cli",
            changed_files: &[],
        };
        let receipt = receipt_from_verification(&root, &req).expect("receipt emitted");

        let mut redacted = c.clone();
        redacted.owner = Some(redact("token=abc123"));
        assert_ne!(
            redacted.owner, c.owner,
            "the fixture must actually be redactable, else this test proves nothing"
        );
        assert_eq!(
            receipt.contract_full_digest,
            full_digest(&redacted).expect("digests"),
            "the emitted full digest is over the redacted contract"
        );
        // `owner` is outside the material projection, so this digest is the
        // same either way — asserted so a change to A-05 breaks here loudly.
        assert_eq!(
            receipt.contract_digest,
            material_digest(&c).expect("digests")
        );
    }

    #[test]
    fn receipt_uses_real_git_revision() {
        // Exercises the dare-project provenance primitives the caller is
        // expected to feed into `revision`.
        let (_dir, root) = temp_root();
        write_verification(
            &root,
            &report(true, vec![step(GateAspect::Build, 0, false)]),
        )
        .expect("report written");
        let c = contract();
        let state = match dare_project::git::worktree_is_dirty(&root) {
            Some(true) => RevisionState::Dirty,
            Some(false) => RevisionState::Clean,
            None => RevisionState::Unattested,
        };
        let rev = revision(
            state,
            vec![RepoRevision {
                alias: "dare-cli".to_string(),
                commit_sha: dare_project::git::head_sha(&root),
                worktree_digest: None,
            }],
        );
        let req = ReceiptRequest {
            task_id: "task-005",
            contract: &c,
            revision: &rev,
            created_at: CREATED_AT,
            driver: "cli",
            changed_files: &[],
        };
        let receipt = receipt_from_verification(&root, &req).expect("receipt emitted");
        assert_eq!(receipt.source_revision.state, state);
        assert_eq!(receipt.task_id, "task-005");
    }

    #[test]
    fn place_worktree_digest_appends_when_no_repository() {
        let mut rev = revision(RevisionState::Unattested, Vec::new());
        let mut c = contract();
        c.starting_context.repositories = vec![RepoRef {
            alias: "primary".to_string(),
            repository_uri: None,
        }];
        place_worktree_digest(&mut rev, &c, "d".repeat(64));
        assert_eq!(rev.repositories.len(), 1);
        assert_eq!(rev.repositories[0].alias, "primary");
        assert_eq!(
            rev.repositories[0].worktree_digest.as_deref(),
            Some(&"d".repeat(64)[..])
        );
    }

    #[test]
    fn place_worktree_digest_prefers_contract_primary_alias() {
        let mut rev = revision(
            RevisionState::Dirty,
            vec![
                RepoRevision {
                    alias: "zeta".to_string(),
                    commit_sha: None,
                    worktree_digest: None,
                },
                dare_cli_repo(None),
            ],
        );
        let c = contract();
        place_worktree_digest(&mut rev, &c, "e".repeat(64));
        let target = rev
            .repositories
            .iter()
            .find(|r| r.alias == "dare-cli")
            .expect("dare-cli present");
        assert!(target.worktree_digest.is_some());
        let other = rev
            .repositories
            .iter()
            .find(|r| r.alias == "zeta")
            .expect("zeta present");
        assert!(
            other.worktree_digest.is_none(),
            "only the contract's primary repository receives the digest"
        );
    }
}
