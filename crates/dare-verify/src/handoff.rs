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
    digest_bytes, digest_file, digest_tree, full_digest, load_contract, material_digest,
    EvidenceState, HandoffDecision, HandoffPolicy, Producer, RefusalCode, RepoRevision,
    RevisionState, SourceRevision, TaskContract, Verdict, VerificationReceipt,
    HANDOFF_SCHEMA_VERSION, MAX_ARTIFACT_REFS,
};
use dare_core::{
    redact, to_canonical_json_string, CoreError, CoreResult, ProjectRoot, SafeRelativePath,
};
use globset::{Glob, GlobSetBuilder};

use crate::handoff_store::{handoff_exists, load_handoff};
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

// ---------------------------------------------------------------------------
// Consumer evaluation (RF-05)
// ---------------------------------------------------------------------------

/// How many characters of a digest a diagnostic may name.
///
/// Enough to tell two digests apart while reading a log, not enough to be
/// mistaken for the digest itself.
const DETAIL_DIGEST_CHARS: usize = 12;

/// Decides whether one parent's handoff may be imported.
///
/// Returns a [`HandoffDecision`] rather than a `Result`, because the decision
/// *is* the product: an `Err` would force every caller to tell "failed to
/// evaluate" apart from "evaluated and refused", and here those are the same
/// outcome.
///
/// The twelve checks run in a fixed order and the first failure wins. The order
/// is testable contract, not an implementation detail: a parent broken two ways
/// must report the lower-numbered cause, otherwise a refusal message would be
/// non-deterministic and useless for diagnosis.
///
/// The order is: the sidecar exists; it deserialises, is schema 1 and has
/// matching ref and digest counts; every path it names is confined to the
/// project; the contract exists, when the policy requires one; the contract on
/// disk still matches the receipt's material digest; the verification report
/// still matches its digest; every required gate passed, per aspect; the
/// verdict is `Passed`; every artifact still matches its digest; the revision
/// is attested, when the policy requires it; the handoff is of the required
/// kind. A size cap tripped by any read refuses with
/// [`RefusalCode::SizeLimit`] whichever check was reading when it tripped.
///
/// Checks 5, 6 and 9 re-read the disk at call time and never reuse a value read
/// earlier in the process. That is what makes "changing a byte after
/// verification invalidates the import" hold in the window between verifying
/// and consuming: under `--agent` composition happens inside the attempt loop,
/// so the re-validation happens per attempt.
pub fn evaluate_parent(
    root: &ProjectRoot,
    dag_rel: &SafeRelativePath,
    parent_id: &str,
    policy: &HandoffPolicy,
) -> HandoffDecision {
    // 1 — the sidecar has to be there at all.
    match handoff_exists(root, parent_id) {
        Ok(true) => {}
        Ok(false) => {
            return refuse(
                parent_id,
                EvidenceState::Missing,
                RefusalCode::ReceiptMissing,
                format!("no handoff sidecar for {parent_id}"),
            )
        }
        Err(e) => return classify(parent_id, &e, RefusalCode::ReceiptInvalid),
    }

    // 2 — it parses, is schema 1, and its two parallel vectors line up.
    let record = match load_handoff(root, parent_id) {
        Ok(r) => r,
        Err(e) => return classify(parent_id, &e, RefusalCode::ReceiptInvalid),
    };
    let receipt = &record.receipt;
    if receipt.artifact_refs.len() != receipt.artifact_digests.len() {
        return refuse(
            parent_id,
            EvidenceState::Invalid,
            RefusalCode::ReceiptInvalid,
            format!(
                "receipt lists {} refs and {} digests",
                receipt.artifact_refs.len(),
                receipt.artifact_digests.len()
            ),
        );
    }

    // 3 — no path in the receipt may leave the project.
    let verification_rel = match confined(&receipt.verification_ref) {
        Some(rel) => rel,
        None => {
            return refuse(
                parent_id,
                EvidenceState::Invalid,
                RefusalCode::RefEscape,
                format!(
                    "verification ref escapes the project: {}",
                    receipt.verification_ref
                ),
            )
        }
    };
    let mut artifact_rels = Vec::with_capacity(receipt.artifact_refs.len());
    for raw in &receipt.artifact_refs {
        match confined(raw) {
            Some(rel) => artifact_rels.push(rel),
            None => {
                return refuse(
                    parent_id,
                    EvidenceState::Invalid,
                    RefusalCode::RefEscape,
                    format!("artifact ref escapes the project: {raw}"),
                )
            }
        }
    }
    if root.resolve(&verification_rel).is_err() {
        return refuse(
            parent_id,
            EvidenceState::Invalid,
            RefusalCode::RefEscape,
            format!(
                "verification ref escapes the project: {}",
                receipt.verification_ref
            ),
        );
    }

    // 4 — the contract, when the policy demands one.
    let contract = match load_contract(root, dag_rel, parent_id) {
        Ok(Some(c)) => Some(c),
        Ok(None) => {
            if policy.require_contracts {
                return refuse(
                    parent_id,
                    EvidenceState::Missing,
                    RefusalCode::ContractMissing,
                    format!("no contract for {parent_id}"),
                );
            }
            None
        }
        // A contract that exists but cannot be read can never be shown to match
        // the receipt, so check 5 is what refuses it.
        Err(e) => return classify(parent_id, &e, RefusalCode::ContractDigestMismatch),
    };

    // 5 — the contract on disk, read now, still matches the receipt (TOCTOU).
    if let Some(c) = &contract {
        let current = match material_digest(c) {
            Ok(d) => d,
            Err(e) => return classify(parent_id, &e, RefusalCode::ContractDigestMismatch),
        };
        if current != receipt.contract_digest {
            return refuse(
                parent_id,
                EvidenceState::Invalid,
                RefusalCode::ContractDigestMismatch,
                format!(
                    "contract digest is {} but the receipt recorded {}",
                    short(&current),
                    short(&receipt.contract_digest)
                ),
            );
        }
    }

    // 6 — the verification report, read now, still matches (TOCTOU).
    let current_verification = match digest_file(root, &verification_rel) {
        Ok(d) => d,
        Err(e) => return classify(parent_id, &e, RefusalCode::VerificationDigestMismatch),
    };
    if current_verification != receipt.verification_digest {
        return refuse(
            parent_id,
            EvidenceState::Invalid,
            RefusalCode::VerificationDigestMismatch,
            format!(
                "verification digest is {} but the receipt recorded {}",
                short(&current_verification),
                short(&receipt.verification_digest)
            ),
        );
    }

    // 7 — every required gate passed, proven per aspect rather than by `ok`.
    if let Some(c) = &contract {
        let required = match required_aspects(&c.verification.required_gates) {
            Ok(a) => a,
            Err(unknown) => {
                return refuse(
                    parent_id,
                    EvidenceState::Insufficient,
                    RefusalCode::GateNotSatisfied,
                    format!("contract requires an unknown gate: {unknown}"),
                )
            }
        };
        let report = match read_report(root, &verification_rel) {
            Ok(r) => r,
            Err(e) => return classify(parent_id, &e, RefusalCode::GateNotSatisfied),
        };
        if let Err(missing) = gates_satisfied(&report, &required) {
            let names: Vec<&str> = missing.iter().map(|a| a.as_str()).collect();
            return refuse(
                parent_id,
                EvidenceState::Insufficient,
                RefusalCode::GateNotSatisfied,
                format!("gates not satisfied: {}", names.join(", ")),
            );
        }
    }

    // 8 — the recorded verdict.
    if receipt.verdict != Verdict::Passed {
        return refuse(
            parent_id,
            EvidenceState::Insufficient,
            RefusalCode::VerdictNotPassed,
            format!("receipt verdict is {}", receipt.verdict.as_str()),
        );
    }

    // 9 — every artifact still hashes to what the receipt recorded (TOCTOU).
    for (rel, expected) in artifact_rels.iter().zip(&receipt.artifact_digests) {
        let current = match digest_file(root, rel) {
            Ok(d) => d,
            Err(e) => return classify(parent_id, &e, RefusalCode::ArtifactDigestMismatch),
        };
        if &current != expected {
            return refuse(
                parent_id,
                EvidenceState::Invalid,
                RefusalCode::ArtifactDigestMismatch,
                format!(
                    "{} hashes to {} but the receipt recorded {}",
                    rel.as_str(),
                    short(&current),
                    short(expected)
                ),
            );
        }
    }

    // 10 — an attested revision, when the policy demands one.
    if policy.require_attested_revision
        && receipt.source_revision.state == RevisionState::Unattested
    {
        return refuse(
            parent_id,
            EvidenceState::Insufficient,
            RefusalCode::RevisionUnattested,
            format!("source revision for {parent_id} is unattested"),
        );
    }

    // 11 — the handoff is of the kind the consumer requires.
    if record.kind != policy.required_kind {
        return refuse(
            parent_id,
            EvidenceState::NotApplicable,
            RefusalCode::NotFinal,
            format!(
                "handoff kind is {} but {} is required",
                record.kind, policy.required_kind
            ),
        );
    }

    let imported_digest = match canonical_receipt_digest(receipt) {
        Ok(d) => d,
        Err(e) => return classify(parent_id, &e, RefusalCode::ReceiptInvalid),
    };
    finish(HandoffDecision::valid(
        parent_id,
        record.deliverable.clone(),
        Some(imported_digest),
    ))
}

/// Digest of the receipt as canonical JSON, recording what was imported.
fn canonical_receipt_digest(receipt: &VerificationReceipt) -> CoreResult<String> {
    let value = serde_json::to_value(receipt)
        .map_err(|e| CoreError::internal(format!("receipt is not serializable: {e}")))?;
    Ok(digest_bytes(to_canonical_json_string(&value)?.as_bytes()))
}

/// Reads the verification report from the path the receipt names.
///
/// Read from `rel` rather than rebuilt from the task id, so check 7 inspects
/// exactly the bytes check 6 digested.
fn read_report(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<VerificationReport> {
    let abs = root.resolve(rel)?;
    let bytes = std::fs::read(abs.as_path().as_std_path()).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CoreError::not_found(format!("verification report not found: {}", rel.as_str()))
        } else {
            CoreError::io(e.to_string())
        }
    })?;
    serde_json::from_slice(&bytes)
        .map_err(|e| CoreError::config(format!("unreadable verification report: {e}")))
}

/// Maps the contract's gate strings onto [`GateAspect`], naming the first
/// unknown one.
fn required_aspects(gates: &[String]) -> Result<Vec<GateAspect>, String> {
    gates
        .iter()
        .map(|g| match g.as_str() {
            "build" => Ok(GateAspect::Build),
            "test" => Ok(GateAspect::Test),
            "lint" => Ok(GateAspect::Lint),
            other => Err(other.to_string()),
        })
        .collect()
}

/// `Some` when `raw` is a relative path that stays inside the project.
fn confined(raw: &str) -> Option<SafeRelativePath> {
    SafeRelativePath::new(raw).ok()
}

/// A refusal, with `detail` redacted on the way out.
fn refuse(
    parent_id: &str,
    state: EvidenceState,
    code: RefusalCode,
    detail: impl Into<String>,
) -> HandoffDecision {
    finish(HandoffDecision::refused(
        parent_id,
        state,
        code,
        redact(&detail.into()),
    ))
}

/// Refuses with `fallback`, unless the error is a size cap — which refuses with
/// [`RefusalCode::SizeLimit`] whichever check was reading when it tripped.
fn classify(parent_id: &str, e: &CoreError, fallback: RefusalCode) -> HandoffDecision {
    if is_size_limit(e) {
        return refuse(
            parent_id,
            EvidenceState::Insufficient,
            RefusalCode::SizeLimit,
            e.message().to_string(),
        );
    }
    let state = match fallback {
        RefusalCode::GateNotSatisfied => EvidenceState::Insufficient,
        _ => EvidenceState::Invalid,
    };
    refuse(parent_id, state, fallback, e.message().to_string())
}

/// Whether an error is one of the digest or read size caps.
fn is_size_limit(e: &CoreError) -> bool {
    matches!(e, CoreError::Config(_)) && e.message().contains("size limit")
}

/// First [`DETAIL_DIGEST_CHARS`] characters of a digest.
///
/// A diagnostic names a prefix, never the whole digest and never the content.
fn short(digest: &str) -> String {
    digest.chars().take(DETAIL_DIGEST_CHARS).collect()
}

/// Checks the three-field invariant on the way out of every return path.
fn finish(decision: HandoffDecision) -> HandoffDecision {
    debug_assert!(
        decision.invariant_holds(),
        "decision broke the Valid/refusal/deliverable invariant: {decision:?}"
    );
    decision
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

    // --- evaluate_parent (task-007) ---------------------------------------

    use crate::handoff_store::write_handoff;
    use dare_contracts::{contract_rel, HandoffKind, HandoffRecord};

    const PARENT: &str = "task-parent";
    const DAG_REL: &str = "DARE/dare-dag.yaml";
    const ARTIFACT: &str = "crates/dare-verify/src/handoff.rs";

    const PARENT_CONTRACT_YAML: &str = r#"
schemaVersion: 1
taskId: task-parent
goal: "Produce the parent deliverable"
scope:
  allowedPaths:
    - "crates/dare-verify/src/**"
acceptance:
  - id: A-01
    assertion: "It produced the deliverable"
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

    /// Builds a fully coherent parent on disk: contract, verification report,
    /// artifact and sidecar, with every digest computed from the real bytes.
    ///
    /// Returns the record as written, so a test can tamper with exactly one
    /// link of the chain and watch the matching check fire.
    fn valid_chain(root: &ProjectRoot) -> HandoffRecord {
        write_at(root, ARTIFACT, b"artifact bytes");
        let c = parse_contract_yaml(PARENT_CONTRACT_YAML).expect("contract parses");
        let c_rel = contract_rel(&dag_rel(), PARENT).expect("contract rel");
        write_at(
            root,
            c_rel.as_str(),
            PARENT_CONTRACT_YAML.trim_start().as_bytes(),
        );

        let mut rep = report(
            true,
            vec![
                step(GateAspect::Build, 0, false),
                step(GateAspect::Test, 0, false),
            ],
        );
        rep.task_id = PARENT.to_string();
        write_verification(root, &rep).expect("report written");

        let rev = revision(
            RevisionState::Clean,
            vec![dare_cli_repo(Some(&"a".repeat(40)))],
        );
        let changed = vec![ARTIFACT.to_string()];
        let req = ReceiptRequest {
            task_id: PARENT,
            contract: &c,
            revision: &rev,
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
            deliverable: "the parent deliverable".to_string(),
            context_bounds: 2000,
            next_steps: Vec::new(),
            open_items: Vec::new(),
            superseded: Vec::new(),
            extra: serde_json::Map::new(),
        };
        write_handoff(root, &record).expect("sidecar written");
        record
    }

    /// Overwrites the sidecar without going through the write-side guards.
    fn force_write_sidecar(root: &ProjectRoot, record: &HandoffRecord) {
        let rel = dare_contracts::handoff_rel(&record.task_id).expect("rel");
        let value = serde_json::to_value(record).expect("serializable");
        let body = to_canonical_json_string(&value).expect("canonical");
        dare_core::fs::atomic_write(root, &rel, body.as_bytes()).expect("written");
    }

    fn strict_policy() -> HandoffPolicy {
        HandoffPolicy {
            strict: true,
            require_attested_revision: true,
            require_contracts: true,
            required_kind: HandoffKind::VerifiedDeliverable,
        }
    }

    fn assert_refused(d: &HandoffDecision, code: RefusalCode, state: EvidenceState) {
        assert_eq!(d.refusal, Some(code), "wrong code: {d:?}");
        assert_eq!(d.state, state, "wrong state: {d:?}");
        assert!(d.deliverable.is_none(), "a refusal carries no deliverable");
        assert!(d.imported_digest.is_none(), "a refusal imports nothing");
        assert!(d.invariant_holds(), "invariant broken: {d:?}");
    }

    #[test]
    fn accepts_full_valid_chain() {
        let (_dir, root) = temp_root();
        let record = valid_chain(&root);
        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_eq!(d.state, EvidenceState::Valid, "{d:?}");
        assert_eq!(d.refusal, None);
        assert_eq!(d.deliverable.as_deref(), Some(record.deliverable.as_str()));
        assert_eq!(
            d.imported_digest,
            Some(canonical_receipt_digest(&record.receipt).expect("digests"))
        );
        assert!(d.detail.is_empty());
        assert!(d.invariant_holds());
    }

    #[test]
    fn refuses_missing_record() {
        let (_dir, root) = temp_root();
        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(&d, RefusalCode::ReceiptMissing, EvidenceState::Missing);
    }

    #[test]
    fn refuses_invalid_record() {
        // Case A: an unsupported schema version.
        let (_dir, root) = temp_root();
        let mut record = valid_chain(&root);
        record.schema_version = 2;
        force_write_sidecar(&root, &record);
        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(&d, RefusalCode::ReceiptInvalid, EvidenceState::Invalid);

        // Case B: refs and digests of different lengths.
        let (_dir2, root2) = temp_root();
        let mut record2 = valid_chain(&root2);
        record2.receipt.artifact_digests.push("f".repeat(64));
        force_write_sidecar(&root2, &record2);
        let d2 = evaluate_parent(&root2, &dag_rel(), PARENT, &strict_policy());
        assert_refused(&d2, RefusalCode::ReceiptInvalid, EvidenceState::Invalid);
    }

    #[test]
    fn refuses_ref_escape_absolute() {
        let (_dir, root) = temp_root();
        let mut record = valid_chain(&root);
        record.receipt.artifact_refs = vec!["/etc/passwd".to_string()];
        record.receipt.artifact_digests = vec!["f".repeat(64)];
        force_write_sidecar(&root, &record);
        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(&d, RefusalCode::RefEscape, EvidenceState::Invalid);
    }

    #[test]
    fn refuses_ref_escape_dotdot() {
        let (_dir, root) = temp_root();
        let mut record = valid_chain(&root);
        record.receipt.artifact_refs = vec!["../outside.txt".to_string()];
        record.receipt.artifact_digests = vec!["f".repeat(64)];
        force_write_sidecar(&root, &record);
        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(&d, RefusalCode::RefEscape, EvidenceState::Invalid);
    }

    #[test]
    fn refuses_missing_contract() {
        let (_dir, root) = temp_root();
        valid_chain(&root);
        let c_rel = contract_rel(&dag_rel(), PARENT).expect("rel");
        let abs = root.resolve(&c_rel).expect("resolve");
        std::fs::remove_file(abs.as_path().as_std_path()).expect("contract removed");

        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(&d, RefusalCode::ContractMissing, EvidenceState::Missing);
    }

    #[test]
    fn permits_missing_contract_when_not_required() {
        let (_dir, root) = temp_root();
        valid_chain(&root);
        let c_rel = contract_rel(&dag_rel(), PARENT).expect("rel");
        let abs = root.resolve(&c_rel).expect("resolve");
        std::fs::remove_file(abs.as_path().as_std_path()).expect("contract removed");

        let policy = HandoffPolicy {
            require_contracts: false,
            ..strict_policy()
        };
        let d = evaluate_parent(&root, &dag_rel(), PARENT, &policy);
        assert_eq!(
            d.state,
            EvidenceState::Valid,
            "check 5 is skipped when no contract is required: {d:?}"
        );
    }

    #[test]
    fn refuses_tampered_contract() {
        let (_dir, root) = temp_root();
        valid_chain(&root);
        // Change `goal`, which is inside the material projection.
        let c_rel = contract_rel(&dag_rel(), PARENT).expect("rel");
        let tampered = PARENT_CONTRACT_YAML
            .trim_start()
            .replace("Produce the parent deliverable", "Produce something else");
        write_at(&root, c_rel.as_str(), tampered.as_bytes());

        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(
            &d,
            RefusalCode::ContractDigestMismatch,
            EvidenceState::Invalid,
        );
        assert!(
            d.detail.contains("contract digest is"),
            "detail must diagnose: {}",
            d.detail
        );
    }

    #[test]
    fn refuses_tampered_verification() {
        let (_dir, root) = temp_root();
        valid_chain(&root);
        // Rewrite the report after the receipt recorded its digest.
        let mut rep = report(true, vec![step(GateAspect::Build, 0, false)]);
        rep.task_id = PARENT.to_string();
        rep.updated_at = "2026-10-14T00:00:00Z".to_string();
        write_verification(&root, &rep).expect("report rewritten");

        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(
            &d,
            RefusalCode::VerificationDigestMismatch,
            EvidenceState::Invalid,
        );
    }

    #[test]
    fn refuses_missing_required_gate() {
        let (_dir, root) = temp_root();
        write_at(&root, ARTIFACT, b"artifact bytes");
        // The contract requires lint; the report never ran it.
        let yaml = PARENT_CONTRACT_YAML
            .trim_start()
            .replace(r#"["build", "test"]"#, r#"["build", "test", "lint"]"#);
        let c = parse_contract_yaml(&yaml).expect("contract parses");
        let c_rel = contract_rel(&dag_rel(), PARENT).expect("rel");
        write_at(&root, c_rel.as_str(), yaml.as_bytes());

        let mut rep = report(
            true,
            vec![
                step(GateAspect::Build, 0, false),
                step(GateAspect::Test, 0, false),
            ],
        );
        rep.task_id = PARENT.to_string();
        write_verification(&root, &rep).expect("report written");

        let rev = revision(
            RevisionState::Clean,
            vec![dare_cli_repo(Some(&"a".repeat(40)))],
        );
        let changed = vec![ARTIFACT.to_string()];
        let req = ReceiptRequest {
            task_id: PARENT,
            contract: &c,
            revision: &rev,
            created_at: CREATED_AT,
            driver: "cli",
            changed_files: &changed,
        };
        let receipt = receipt_from_verification(&root, &req).expect("receipt emitted");
        let record = HandoffRecord {
            schema_version: HANDOFF_SCHEMA_VERSION,
            task_id: PARENT.to_string(),
            kind: HandoffKind::VerifiedDeliverable,
            receipt,
            deliverable: "parent".to_string(),
            context_bounds: 2000,
            next_steps: Vec::new(),
            open_items: Vec::new(),
            superseded: Vec::new(),
            extra: serde_json::Map::new(),
        };
        write_handoff(&root, &record).expect("sidecar written");

        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(
            &d,
            RefusalCode::GateNotSatisfied,
            EvidenceState::Insufficient,
        );
        assert!(
            d.detail.contains("lint"),
            "detail must name it: {}",
            d.detail
        );
    }

    #[test]
    fn refuses_verdict_failed() {
        let (_dir, root) = temp_root();
        let mut record = valid_chain(&root);
        record.receipt.verdict = Verdict::Failed;
        force_write_sidecar(&root, &record);
        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(
            &d,
            RefusalCode::VerdictNotPassed,
            EvidenceState::Insufficient,
        );
    }

    #[test]
    fn refuses_tampered_artifact() {
        let (_dir, root) = temp_root();
        valid_chain(&root);
        // One byte different, after the receipt was emitted.
        write_at(&root, ARTIFACT, b"artifact bytez");
        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(
            &d,
            RefusalCode::ArtifactDigestMismatch,
            EvidenceState::Invalid,
        );
        assert!(
            d.detail.contains(ARTIFACT),
            "detail names the offending path: {}",
            d.detail
        );
    }

    #[test]
    fn refuses_unattested_when_required() {
        let (_dir, root) = temp_root();
        let mut record = valid_chain(&root);
        record.receipt.source_revision.state = RevisionState::Unattested;
        force_write_sidecar(&root, &record);
        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(
            &d,
            RefusalCode::RevisionUnattested,
            EvidenceState::Insufficient,
        );

        // The same chain is accepted when attestation is not required.
        let policy = HandoffPolicy {
            require_attested_revision: false,
            ..strict_policy()
        };
        let d2 = evaluate_parent(&root, &dag_rel(), PARENT, &policy);
        assert_eq!(d2.state, EvidenceState::Valid, "{d2:?}");
    }

    #[test]
    fn refuses_followup_when_final_required() {
        let (_dir, root) = temp_root();
        let mut record = valid_chain(&root);
        record.kind = HandoffKind::FollowUpHandoff;
        record.open_items = vec!["still pending".to_string()];
        force_write_sidecar(&root, &record);
        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(&d, RefusalCode::NotFinal, EvidenceState::NotApplicable);
    }

    #[test]
    fn refuses_oversize_contract() {
        let (_dir, root) = temp_root();
        valid_chain(&root);
        let c_rel = contract_rel(&dag_rel(), PARENT).expect("rel");
        let padding = format!(
            "{}\n# {}\n",
            PARENT_CONTRACT_YAML.trim_start(),
            "x".repeat(2 * 1024 * 1024)
        );
        write_at(&root, c_rel.as_str(), padding.as_bytes());

        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_refused(&d, RefusalCode::SizeLimit, EvidenceState::Insufficient);
        assert!(
            d.detail.contains("size limit"),
            "detail must say why: {}",
            d.detail
        );
    }

    #[test]
    fn check_order_lower_number_wins() {
        let (_dir, root) = temp_root();
        valid_chain(&root);
        // Break check 5 (contract) *and* check 9 (artifact) at once.
        let c_rel = contract_rel(&dag_rel(), PARENT).expect("rel");
        let tampered = PARENT_CONTRACT_YAML
            .trim_start()
            .replace("Produce the parent deliverable", "Produce something else");
        write_at(&root, c_rel.as_str(), tampered.as_bytes());
        write_at(&root, ARTIFACT, b"also tampered");

        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());
        assert_eq!(
            d.refusal,
            Some(RefusalCode::ContractDigestMismatch),
            "check 5 must win over check 9, otherwise diagnosis is non-deterministic: {d:?}"
        );
    }

    #[test]
    fn detail_never_carries_a_whole_digest() {
        let (_dir, root) = temp_root();
        let record = valid_chain(&root);
        write_at(&root, ARTIFACT, b"tampered");
        let d = evaluate_parent(&root, &dag_rel(), PARENT, &strict_policy());

        let full = &record.receipt.artifact_digests[0];
        assert_eq!(full.len(), 64);
        assert!(
            !d.detail.contains(full.as_str()),
            "detail leaked the whole digest: {}",
            d.detail
        );
        assert!(
            d.detail
                .contains(&full.chars().take(12).collect::<String>()),
            "detail should carry the 12-char prefix: {}",
            d.detail
        );
    }

    #[test]
    fn every_refusal_code_is_reachable() {
        // StrictNoContracts belongs to the task-009 preflight; the other
        // twelve are produced by evaluate_parent and each has its own test.
        let covered = [
            RefusalCode::ContractMissing,
            RefusalCode::ReceiptMissing,
            RefusalCode::ReceiptInvalid,
            RefusalCode::ContractDigestMismatch,
            RefusalCode::ArtifactDigestMismatch,
            RefusalCode::VerificationDigestMismatch,
            RefusalCode::GateNotSatisfied,
            RefusalCode::VerdictNotPassed,
            RefusalCode::NotFinal,
            RefusalCode::RefEscape,
            RefusalCode::SizeLimit,
            RefusalCode::RevisionUnattested,
        ];
        assert_eq!(covered.len(), 12);
        for code in RefusalCode::ALL {
            assert!(
                covered.contains(&code) || code == RefusalCode::StrictNoContracts,
                "{code} is neither covered here nor owned by the preflight"
            );
        }
    }

    proptest::proptest! {
        /// The three coupled fields stay coherent however a decision is built:
        /// `Valid` iff no refusal iff a deliverable is present.
        #[test]
        fn decision_invariant_holds_under_generation(
            parent in "[a-z0-9-]{1,24}",
            valid in proptest::prelude::any::<bool>(),
            state_pick in 0usize..6,
            code_pick in 0usize..13,
            deliverable in ".{0,64}",
            detail in ".{0,64}",
        ) {
            let states = [
                EvidenceState::Valid,
                EvidenceState::Missing,
                EvidenceState::Invalid,
                EvidenceState::Stale,
                EvidenceState::NotApplicable,
                EvidenceState::Insufficient,
            ];
            let d = if valid {
                HandoffDecision::valid(&parent, &deliverable, None)
            } else {
                HandoffDecision::refused(
                    &parent,
                    states[state_pick],
                    RefusalCode::ALL[code_pick],
                    &detail,
                )
            };
            proptest::prop_assert!(d.invariant_holds(), "{:?}", d);
            proptest::prop_assert_eq!(d.state == EvidenceState::Valid, d.refusal.is_none());
            proptest::prop_assert_eq!(d.state == EvidenceState::Valid, d.deliverable.is_some());
            if !valid {
                proptest::prop_assert!(d.imported_digest.is_none());
            }
        }
    }
}
