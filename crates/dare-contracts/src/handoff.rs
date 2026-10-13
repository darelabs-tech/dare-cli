//! Task Contract and Handoff schemas, validation and confined paths.
//!
//! This module is the shared foundation of the verified-handoff layer (A-01):
//! `dare-dag` and `dare-verify` both already depend on `dare-contracts`, so the
//! persisted types live here and neither side has to depend on the other.
//!
//! Parsing and validation are deliberately separate, mirroring
//! `parse_dag_yaml` vs `validate_dag`: [`parse_contract_yaml`] only rejects
//! malformed YAML and unsupported schema versions, while [`validate_contract`]
//! accumulates every business-rule violation so a preflight can report them all
//! at once.

use std::collections::BTreeSet;
use std::fmt;

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use globset::Glob;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::io::read_limited;

/// Schema version accepted for `<contract_dir>/<task-id>.yaml`.
pub const CONTRACT_SCHEMA_VERSION: u32 = 1;

/// Schema version accepted for `.dare/handoff/<task-id>.json`.
pub const HANDOFF_SCHEMA_VERSION: u32 = 1;

/// Directory holding handoff sidecars, relative to the project root.
pub const HANDOFF_DIR_REL: &str = ".dare/handoff";

/// Directory name holding contracts, sibling of the resolved DAG file (A-04).
pub const CONTRACT_DIR_NAME: &str = "contracts";

/// Hard cap on `artifactRefs` entries in a receipt.
pub const MAX_ARTIFACT_REFS: usize = 512;

/// Hard cap on the total bytes a tree digest may cover.
pub const MAX_TREE_DIGEST_BYTES: u64 = 64 * 1024 * 1024;

/// Hard cap on retained superseded receipts (FIFO, oldest dropped).
pub const MAX_SUPERSEDED: usize = 16;

const MAX_GOAL_CHARS: usize = 500;
const MAX_ALLOWED_PATHS: usize = 64;
const MAX_ACCEPTANCE: usize = 64;
const MAX_ASSERTION_CHARS: usize = 500;
const MAX_REQUIRED_GATES: usize = 3;
const MAX_REQUIREMENT_REFS: usize = 32;
const MAX_REQUIREMENT_REF_CHARS: usize = 128;
const MAX_REPOSITORIES: usize = 8;
const MAX_TASK_TYPE_CHARS: usize = 64;
const MAX_OWNER_CHARS: usize = 128;
const MAX_NEXT_STEPS: usize = 32;
const MAX_OPEN_ITEMS: usize = 32;
const MAX_ALIAS_CHARS: usize = 32;

/// Gate aspects a contract may require, in canonical order.
const GATE_ASPECTS: [&str; 3] = ["build", "test", "lint"];

/// Accepted values of `startingContext.source` (BLUEPRINT §2.1).
const STARTING_CONTEXT_SOURCES: [&str; 3] = ["project-revision", "parent-handoff", "none"];

// ---------------------------------------------------------------------------
// Persisted types
// ---------------------------------------------------------------------------

/// A Task Contract — `<contract_dir>/<task-id>.yaml`, versioned in Git.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskContract {
    pub schema_version: u32,
    pub task_id: String,
    pub goal: String,
    pub scope: ContractScope,
    #[serde(default)]
    pub acceptance: Vec<AcceptanceCriterion>,
    pub verification: ContractVerification,
    pub exit_criteria: ExitCriteria,
    pub starting_context: StartingContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default)]
    pub requirement_refs: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Path scope of a contract. `deniedPaths` is evaluated after `allowedPaths`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContractScope {
    #[serde(default)]
    pub allowed_paths: Vec<String>,
    #[serde(default)]
    pub denied_paths: Vec<String>,
}

/// One acceptance criterion: a stable id plus the assertion it stands for.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AcceptanceCriterion {
    pub id: String,
    pub assertion: String,
}

/// Gates the task must satisfy before a verified deliverable may be emitted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContractVerification {
    #[serde(default)]
    pub required_gates: Vec<String>,
}

/// How the task is expected to finish.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExitCriteria {
    pub kind: HandoffKind,
}

/// Where the task starts from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StartingContext {
    pub source: String,
    #[serde(default)]
    pub repositories: Vec<RepoRef>,
}

/// A repository the task starts against.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RepoRef {
    pub alias: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_uri: Option<String>,
}

/// Kind of handoff a task produces.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HandoffKind {
    VerifiedDeliverable,
    FollowUpHandoff,
}

impl HandoffKind {
    /// Stable wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            HandoffKind::VerifiedDeliverable => "verified-deliverable",
            HandoffKind::FollowUpHandoff => "follow-up-handoff",
        }
    }
}

impl fmt::Display for HandoffKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The handoff sidecar — `.dare/handoff/<task-id>.json`, canonical JSON.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HandoffRecord {
    pub schema_version: u32,
    pub task_id: String,
    pub kind: HandoffKind,
    pub receipt: VerificationReceipt,
    pub deliverable: String,
    pub context_bounds: u32,
    #[serde(default)]
    pub next_steps: Vec<String>,
    #[serde(default)]
    pub open_items: Vec<String>,
    #[serde(default)]
    pub superseded: Vec<SupersededReceipt>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Proof that a task's gates ran and what they ran against.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VerificationReceipt {
    pub schema_version: u32,
    pub task_id: String,
    /// Digest of the material projection of the contract (A-05).
    pub contract_digest: String,
    /// Digest of the whole canonical contract, `extra` included (A-05).
    pub contract_full_digest: String,
    pub source_revision: SourceRevision,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default)]
    pub artifact_digests: Vec<String>,
    pub verification_ref: String,
    pub verification_digest: String,
    pub verdict: Verdict,
    pub created_at: String,
    pub producer: Producer,
}

/// Revision the work was produced from (A-06 tri-state).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SourceRevision {
    pub state: RevisionState,
    #[serde(default)]
    pub repositories: Vec<RepoRevision>,
}

/// Whether the source revision could be attested at all.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RevisionState {
    Clean,
    Dirty,
    Unattested,
}

impl RevisionState {
    /// Stable wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            RevisionState::Clean => "clean",
            RevisionState::Dirty => "dirty",
            RevisionState::Unattested => "unattested",
        }
    }
}

/// Per-repository revision evidence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RepoRevision {
    pub alias: String,
    pub commit_sha: Option<String>,
    pub worktree_digest: Option<String>,
}

/// Outcome recorded by the receipt.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Passed,
    Failed,
    Insufficient,
}

impl Verdict {
    /// Stable wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Passed => "passed",
            Verdict::Failed => "failed",
            Verdict::Insufficient => "insufficient",
        }
    }
}

/// What produced a receipt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Producer {
    pub cli_version: String,
    pub driver: String,
}

/// A receipt that was replaced by a newer one for the same task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SupersededReceipt {
    pub receipt_digest: String,
    pub created_at: String,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Policy and decision
// ---------------------------------------------------------------------------

/// State of a parent's evidence as seen by a consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceState {
    Valid,
    Missing,
    Invalid,
    Stale,
    NotApplicable,
    Insufficient,
}

/// Stable refusal codes surfaced to users and goldens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalCode {
    StrictNoContracts,
    ContractMissing,
    ReceiptMissing,
    ReceiptInvalid,
    ContractDigestMismatch,
    ArtifactDigestMismatch,
    VerificationDigestMismatch,
    GateNotSatisfied,
    VerdictNotPassed,
    NotFinal,
    RefEscape,
    SizeLimit,
    RevisionUnattested,
}

impl RefusalCode {
    /// Every refusal code, in declaration order.
    pub const ALL: [RefusalCode; 13] = [
        RefusalCode::StrictNoContracts,
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

    /// Stable wire string, e.g. [`RefusalCode::ReceiptMissing`] →
    /// `"HANDOFF_RECEIPT_MISSING"`.
    pub fn as_str(self) -> &'static str {
        match self {
            RefusalCode::StrictNoContracts => "HANDOFF_STRICT_NO_CONTRACTS",
            RefusalCode::ContractMissing => "HANDOFF_CONTRACT_MISSING",
            RefusalCode::ReceiptMissing => "HANDOFF_RECEIPT_MISSING",
            RefusalCode::ReceiptInvalid => "HANDOFF_RECEIPT_INVALID",
            RefusalCode::ContractDigestMismatch => "HANDOFF_CONTRACT_DIGEST_MISMATCH",
            RefusalCode::ArtifactDigestMismatch => "HANDOFF_ARTIFACT_DIGEST_MISMATCH",
            RefusalCode::VerificationDigestMismatch => "HANDOFF_VERIFICATION_DIGEST_MISMATCH",
            RefusalCode::GateNotSatisfied => "HANDOFF_GATE_NOT_SATISFIED",
            RefusalCode::VerdictNotPassed => "HANDOFF_VERDICT_NOT_PASSED",
            RefusalCode::NotFinal => "HANDOFF_NOT_FINAL",
            RefusalCode::RefEscape => "HANDOFF_REF_ESCAPE",
            RefusalCode::SizeLimit => "HANDOFF_SIZE_LIMIT",
            RefusalCode::RevisionUnattested => "HANDOFF_REVISION_UNATTESTED",
        }
    }

    /// Inverse of [`RefusalCode::as_str`]; `None` for unknown strings.
    pub fn parse(s: &str) -> Option<Self> {
        RefusalCode::ALL.into_iter().find(|c| c.as_str() == s)
    }
}

impl fmt::Display for RefusalCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Decision taken about one parent's handoff.
///
/// Invariant: `state == Valid` ⟺ `refusal.is_none()` ⟺ `deliverable.is_some()`.
/// The constructors below are the only way to build one that upholds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffDecision {
    pub parent_id: String,
    pub state: EvidenceState,
    pub refusal: Option<RefusalCode>,
    /// Already redacted at construction time by the caller.
    pub detail: String,
    pub imported_digest: Option<String>,
    /// `Some` if and only if `state == EvidenceState::Valid`.
    pub deliverable: Option<String>,
}

impl HandoffDecision {
    /// A decision that imports the parent's deliverable.
    pub fn valid(
        parent_id: impl Into<String>,
        deliverable: impl Into<String>,
        imported_digest: Option<String>,
    ) -> Self {
        Self {
            parent_id: parent_id.into(),
            state: EvidenceState::Valid,
            refusal: None,
            detail: String::new(),
            imported_digest,
            deliverable: Some(deliverable.into()),
        }
    }

    /// A decision that refuses the parent's deliverable.
    ///
    /// `state` is forced away from [`EvidenceState::Valid`] — a refusal can
    /// never be valid — so the invariant holds by construction.
    pub fn refused(
        parent_id: impl Into<String>,
        state: EvidenceState,
        refusal: RefusalCode,
        detail: impl Into<String>,
    ) -> Self {
        let state = if state == EvidenceState::Valid {
            EvidenceState::Invalid
        } else {
            state
        };
        Self {
            parent_id: parent_id.into(),
            state,
            refusal: Some(refusal),
            detail: detail.into(),
            imported_digest: None,
            deliverable: None,
        }
    }

    /// Checks the three-way invariant documented on the type.
    pub fn invariant_holds(&self) -> bool {
        let valid = self.state == EvidenceState::Valid;
        valid == self.refusal.is_none() && valid == self.deliverable.is_some()
    }
}

/// Consumer-side policy, read from `dare.config.json` (BLUEPRINT §2.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffPolicy {
    pub strict: bool,
    pub require_attested_revision: bool,
    pub require_contracts: bool,
    pub required_kind: HandoffKind,
}

impl Default for HandoffPolicy {
    /// Absence of configuration means permissive: nothing is enforced and the
    /// expected kind is a verified deliverable.
    fn default() -> Self {
        Self {
            strict: false,
            require_attested_revision: false,
            require_contracts: false,
            required_kind: HandoffKind::VerifiedDeliverable,
        }
    }
}

// ---------------------------------------------------------------------------
// Violations
// ---------------------------------------------------------------------------

/// Whether a violation blocks or only informs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationSeverity {
    Error,
    Warning,
}

impl ViolationSeverity {
    /// Stable wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            ViolationSeverity::Error => "error",
            ViolationSeverity::Warning => "warning",
        }
    }
}

impl fmt::Display for ViolationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single business-rule violation found by [`validate_contract`].
///
/// `index` on the path variants is the position in the concatenation
/// `allowedPaths ++ deniedPaths`, which is the order the paths are checked in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractViolation {
    TaskIdUnsafe,
    TaskIdNotInDag,
    TaskIdFileMismatch,
    GoalEmpty,
    GoalTooLong,
    AllowedPathsEmpty,
    AllowedPathsTooMany,
    AllowedPathsTooBroad,
    GlobInvalid {
        index: usize,
        message: String,
    },
    PathNotRelative {
        index: usize,
    },
    AcceptanceEmpty,
    AcceptanceIdDuplicate {
        id: String,
    },
    AcceptanceIdMalformed {
        id: String,
    },
    AcceptanceAssertionEmpty {
        index: usize,
    },
    RequiredGatesEmpty,
    RequiredGatesDuplicate {
        aspect: String,
    },
    RequiredGatesUnknown {
        aspect: String,
    },
    RepoAliasMalformed {
        alias: String,
    },
    StartingContextSourceUnknown {
        source: String,
    },
    /// A count or length outside the normative range of BLUEPRINT §2.1/§2.2
    /// that has no dedicated variant above.
    FieldOutOfRange {
        field: &'static str,
        min: usize,
        max: usize,
        actual: usize,
    },
    OpenItemsRequiredForFollowUp,
}

impl ContractViolation {
    /// Blocking by default; only an over-broad scope is advisory.
    pub fn severity(&self) -> ViolationSeverity {
        match self {
            ContractViolation::AllowedPathsTooBroad => ViolationSeverity::Warning,
            _ => ViolationSeverity::Error,
        }
    }
}

impl fmt::Display for ContractViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContractViolation::TaskIdUnsafe => f.write_str("taskId is not path-safe"),
            ContractViolation::TaskIdNotInDag => f.write_str("taskId is not present in the DAG"),
            ContractViolation::TaskIdFileMismatch => {
                f.write_str("taskId does not match the contract file name")
            }
            ContractViolation::GoalEmpty => f.write_str("goal is empty"),
            ContractViolation::GoalTooLong => {
                write!(f, "goal exceeds {MAX_GOAL_CHARS} characters")
            }
            ContractViolation::AllowedPathsEmpty => f.write_str("scope.allowedPaths is empty"),
            ContractViolation::AllowedPathsTooMany => {
                write!(
                    f,
                    "scope.allowedPaths has more than {MAX_ALLOWED_PATHS} entries"
                )
            }
            ContractViolation::AllowedPathsTooBroad => {
                f.write_str("scope.allowedPaths matches the whole project")
            }
            ContractViolation::GlobInvalid { index, message } => {
                write!(f, "scope path #{index} is not a valid glob: {message}")
            }
            ContractViolation::PathNotRelative { index } => {
                write!(f, "scope path #{index} is absolute or escapes the project")
            }
            ContractViolation::AcceptanceEmpty => f.write_str("acceptance is empty"),
            ContractViolation::AcceptanceIdDuplicate { id } => {
                write!(f, "acceptance id {id} is duplicated")
            }
            ContractViolation::AcceptanceIdMalformed { id } => {
                write!(f, "acceptance id {id} does not match ^[A-Z]-[0-9]{{2,3}}$")
            }
            ContractViolation::AcceptanceAssertionEmpty { index } => {
                write!(f, "acceptance #{index} has an empty assertion")
            }
            ContractViolation::RequiredGatesEmpty => {
                f.write_str("verification.requiredGates is empty")
            }
            ContractViolation::RequiredGatesDuplicate { aspect } => {
                write!(f, "verification.requiredGates repeats {aspect}")
            }
            ContractViolation::RequiredGatesUnknown { aspect } => {
                write!(f, "verification.requiredGates has unknown aspect {aspect}")
            }
            ContractViolation::RepoAliasMalformed { alias } => {
                write!(
                    f,
                    "repository alias {alias} does not match ^[a-z][a-z0-9-]{{0,31}}$"
                )
            }
            ContractViolation::StartingContextSourceUnknown { source } => {
                write!(f, "startingContext.source {source} is not recognised")
            }
            ContractViolation::FieldOutOfRange {
                field,
                min,
                max,
                actual,
            } => write!(f, "{field} must be {min}..={max}, got {actual}"),
            ContractViolation::OpenItemsRequiredForFollowUp => {
                f.write_str("openItems must be non-empty for a follow-up-handoff")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Parses a contract from YAML text already read under `read_limited`.
///
/// Only two things fail here: malformed YAML and an unsupported schema
/// version. Business rules belong to [`validate_contract`].
pub fn parse_contract_yaml(text: &str) -> CoreResult<TaskContract> {
    let contract: TaskContract = serde_yaml::from_str(text)
        .map_err(|e| CoreError::config(format!("invalid contract yaml: {e}")))?;
    if contract.schema_version != CONTRACT_SCHEMA_VERSION {
        return Err(CoreError::config("unsupported contract schema version"));
    }
    Ok(contract)
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Collects every business-rule violation of a contract.
///
/// Never returns `Err`: the preflight needs the whole list at once. The order
/// is deterministic — declared-field order, then index within a field.
pub fn validate_contract(
    c: &TaskContract,
    file_stem: &str,
    dag_task_ids: &BTreeSet<String>,
) -> Vec<ContractViolation> {
    let mut out = Vec::new();

    // taskId
    if !task_id_is_path_safe(&c.task_id) {
        out.push(ContractViolation::TaskIdUnsafe);
    }
    if !dag_task_ids.contains(&c.task_id) {
        out.push(ContractViolation::TaskIdNotInDag);
    }
    if c.task_id != file_stem {
        out.push(ContractViolation::TaskIdFileMismatch);
    }

    // goal
    let goal = c.goal.trim();
    if goal.is_empty() {
        out.push(ContractViolation::GoalEmpty);
    } else if goal.chars().count() > MAX_GOAL_CHARS {
        out.push(ContractViolation::GoalTooLong);
    }

    // scope.allowedPaths / scope.deniedPaths
    if c.scope.allowed_paths.is_empty() {
        out.push(ContractViolation::AllowedPathsEmpty);
    } else if c.scope.allowed_paths.len() > MAX_ALLOWED_PATHS {
        out.push(ContractViolation::AllowedPathsTooMany);
    }
    for (index, pattern) in c
        .scope
        .allowed_paths
        .iter()
        .chain(c.scope.denied_paths.iter())
        .enumerate()
    {
        if !path_is_relative_glob(pattern) {
            out.push(ContractViolation::PathNotRelative { index });
        } else if let Err(e) = Glob::new(pattern) {
            out.push(ContractViolation::GlobInvalid {
                index,
                message: e.to_string(),
            });
        }
    }
    if c.scope.allowed_paths.iter().any(|p| glob_is_too_broad(p)) {
        out.push(ContractViolation::AllowedPathsTooBroad);
    }

    // acceptance
    if c.acceptance.is_empty() {
        out.push(ContractViolation::AcceptanceEmpty);
    } else if c.acceptance.len() > MAX_ACCEPTANCE {
        out.push(ContractViolation::FieldOutOfRange {
            field: "acceptance",
            min: 1,
            max: MAX_ACCEPTANCE,
            actual: c.acceptance.len(),
        });
    }
    let mut seen_ids: BTreeSet<&str> = BTreeSet::new();
    for (index, criterion) in c.acceptance.iter().enumerate() {
        if !acceptance_id_is_valid(&criterion.id) {
            out.push(ContractViolation::AcceptanceIdMalformed {
                id: criterion.id.clone(),
            });
        }
        if !seen_ids.insert(criterion.id.as_str()) {
            out.push(ContractViolation::AcceptanceIdDuplicate {
                id: criterion.id.clone(),
            });
        }
        let assertion = criterion.assertion.trim();
        if assertion.is_empty() {
            out.push(ContractViolation::AcceptanceAssertionEmpty { index });
        } else if assertion.chars().count() > MAX_ASSERTION_CHARS {
            out.push(ContractViolation::FieldOutOfRange {
                field: "acceptance[].assertion",
                min: 1,
                max: MAX_ASSERTION_CHARS,
                actual: assertion.chars().count(),
            });
        }
    }

    // verification.requiredGates
    let gates = &c.verification.required_gates;
    if gates.is_empty() {
        out.push(ContractViolation::RequiredGatesEmpty);
    } else if gates.len() > MAX_REQUIRED_GATES {
        out.push(ContractViolation::FieldOutOfRange {
            field: "verification.requiredGates",
            min: 1,
            max: MAX_REQUIRED_GATES,
            actual: gates.len(),
        });
    }
    let mut seen_gates: BTreeSet<&str> = BTreeSet::new();
    for gate in gates {
        if !GATE_ASPECTS.contains(&gate.as_str()) {
            out.push(ContractViolation::RequiredGatesUnknown {
                aspect: gate.clone(),
            });
        }
        if !seen_gates.insert(gate.as_str()) {
            out.push(ContractViolation::RequiredGatesDuplicate {
                aspect: gate.clone(),
            });
        }
    }

    // exitCriteria.kind is a closed enum — the parser already rejects anything else.

    // startingContext
    let source = c.starting_context.source.as_str();
    if !STARTING_CONTEXT_SOURCES.contains(&source) {
        out.push(ContractViolation::StartingContextSourceUnknown {
            source: source.to_string(),
        });
    }
    let repos = &c.starting_context.repositories;
    if repos.len() > MAX_REPOSITORIES {
        out.push(ContractViolation::FieldOutOfRange {
            field: "startingContext.repositories",
            min: 0,
            max: MAX_REPOSITORIES,
            actual: repos.len(),
        });
    }
    for repo in repos {
        if !repo_alias_is_valid(&repo.alias) {
            out.push(ContractViolation::RepoAliasMalformed {
                alias: repo.alias.clone(),
            });
        }
    }

    // taskType / owner
    if let Some(task_type) = &c.task_type {
        let len = task_type.trim().chars().count();
        if len == 0 || len > MAX_TASK_TYPE_CHARS {
            out.push(ContractViolation::FieldOutOfRange {
                field: "taskType",
                min: 1,
                max: MAX_TASK_TYPE_CHARS,
                actual: len,
            });
        }
    }
    if let Some(owner) = &c.owner {
        let len = owner.trim().chars().count();
        if len == 0 || len > MAX_OWNER_CHARS {
            out.push(ContractViolation::FieldOutOfRange {
                field: "owner",
                min: 1,
                max: MAX_OWNER_CHARS,
                actual: len,
            });
        }
    }

    // requirementRefs
    if c.requirement_refs.len() > MAX_REQUIREMENT_REFS {
        out.push(ContractViolation::FieldOutOfRange {
            field: "requirementRefs",
            min: 0,
            max: MAX_REQUIREMENT_REFS,
            actual: c.requirement_refs.len(),
        });
    }
    for reference in &c.requirement_refs {
        let len = reference.trim().chars().count();
        if len == 0 || len > MAX_REQUIREMENT_REF_CHARS {
            out.push(ContractViolation::FieldOutOfRange {
                field: "requirementRefs[]",
                min: 1,
                max: MAX_REQUIREMENT_REF_CHARS,
                actual: len,
            });
        }
    }

    out
}

/// Collects every business-rule violation of a handoff record (BLUEPRINT §2.2).
///
/// Same contract as [`validate_contract`]: never `Err`, deterministic order.
/// This is where [`ContractViolation::OpenItemsRequiredForFollowUp`] comes
/// from — `openItems` lives on the record, not on the contract.
pub fn validate_handoff_record(h: &HandoffRecord) -> Vec<ContractViolation> {
    let mut out = Vec::new();

    if !task_id_is_path_safe(&h.task_id) {
        out.push(ContractViolation::TaskIdUnsafe);
    }
    if h.next_steps.len() > MAX_NEXT_STEPS {
        out.push(ContractViolation::FieldOutOfRange {
            field: "nextSteps",
            min: 0,
            max: MAX_NEXT_STEPS,
            actual: h.next_steps.len(),
        });
    }
    if h.open_items.len() > MAX_OPEN_ITEMS {
        out.push(ContractViolation::FieldOutOfRange {
            field: "openItems",
            min: 0,
            max: MAX_OPEN_ITEMS,
            actual: h.open_items.len(),
        });
    }
    if h.kind == HandoffKind::FollowUpHandoff
        && h.open_items.iter().all(|item| item.trim().is_empty())
    {
        out.push(ContractViolation::OpenItemsRequiredForFollowUp);
    }
    if h.superseded.len() > MAX_SUPERSEDED {
        out.push(ContractViolation::FieldOutOfRange {
            field: "superseded",
            min: 0,
            max: MAX_SUPERSEDED,
            actual: h.superseded.len(),
        });
    }

    out
}

// ---------------------------------------------------------------------------
// Predicates (kept local on purpose — see A-01: no dare-verify dependency)
// ---------------------------------------------------------------------------

/// Same rule as `dare_verify::task_id_is_path_safe`, copied so that
/// `dare-contracts` keeps its current dependency tree.
fn task_id_is_path_safe(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

/// `^[A-Z]-[0-9]{2,3}$`, hand-rolled to avoid a `regex` dependency.
fn acceptance_id_is_valid(id: &str) -> bool {
    let bytes = id.as_bytes();
    if bytes.len() < 4 || bytes.len() > 5 {
        return false;
    }
    if !bytes[0].is_ascii_uppercase() || bytes[1] != b'-' {
        return false;
    }
    bytes[2..].iter().all(u8::is_ascii_digit)
}

/// `^[a-z][a-z0-9-]{0,31}$`, hand-rolled to avoid a `regex` dependency.
fn repo_alias_is_valid(alias: &str) -> bool {
    let bytes = alias.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_ALIAS_CHARS {
        return false;
    }
    if !bytes[0].is_ascii_lowercase() {
        return false;
    }
    bytes[1..]
        .iter()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}

/// A scope entry must stay inside the project: relative, no `..`, no drive.
fn path_is_relative_glob(pattern: &str) -> bool {
    if pattern.is_empty() || pattern.contains('\0') {
        return false;
    }
    if pattern.starts_with('/') || pattern.starts_with('\\') {
        return false;
    }
    let bytes = pattern.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return false;
    }
    !pattern.split(['/', '\\']).any(|segment| segment == "..")
}

/// True when every segment is a wildcard, i.e. the glob covers the whole tree.
fn glob_is_too_broad(pattern: &str) -> bool {
    let segments: Vec<&str> = pattern
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .collect();
    !segments.is_empty() && segments.iter().all(|s| *s == "*" || *s == "**")
}

// ---------------------------------------------------------------------------
// Paths (A-04)
// ---------------------------------------------------------------------------

fn unsafe_id_err(task_id: &str) -> CoreError {
    CoreError::invalid_input(format!("unsafe handoff task id: {task_id}"))
}

/// Contract directory for a resolved DAG path: `parent(dag_rel)/contracts`.
///
/// A DAG at the project root yields `contracts`. `dag_rel == "."` is not a
/// file and is rejected.
pub fn contract_dir_for_dag(dag_rel: &SafeRelativePath) -> CoreResult<SafeRelativePath> {
    let raw = dag_rel.as_str();
    if raw == "." {
        return Err(CoreError::invalid_input(
            "dag path has no parent directory for contracts",
        ));
    }
    let parent = match raw.rsplit_once('/') {
        Some((parent, _)) if !parent.is_empty() => format!("{parent}/{CONTRACT_DIR_NAME}"),
        _ => CONTRACT_DIR_NAME.to_string(),
    };
    SafeRelativePath::new(&parent)
}

/// Path of one task's contract: `<contract_dir>/<task-id>.yaml`.
pub fn contract_rel(dag_rel: &SafeRelativePath, task_id: &str) -> CoreResult<SafeRelativePath> {
    if !task_id_is_path_safe(task_id) {
        return Err(unsafe_id_err(task_id));
    }
    let dir = contract_dir_for_dag(dag_rel)?;
    SafeRelativePath::new(&format!("{}/{task_id}.yaml", dir.as_str()))
}

/// Path of one task's handoff sidecar: `.dare/handoff/<task-id>.json`.
pub fn handoff_rel(task_id: &str) -> CoreResult<SafeRelativePath> {
    if !task_id_is_path_safe(task_id) {
        return Err(unsafe_id_err(task_id));
    }
    SafeRelativePath::new(&format!("{HANDOFF_DIR_REL}/{task_id}.json"))
}

/// Loads a task's contract, if one exists.
///
/// Absence is a legitimate state (D-06), so a missing file is `Ok(None)`.
/// The file is read through `read_limited`, which refuses anything above
/// 2 MiB before parsing.
pub fn load_contract(
    root: &ProjectRoot,
    dag_rel: &SafeRelativePath,
    task_id: &str,
) -> CoreResult<Option<TaskContract>> {
    let rel = contract_rel(dag_rel, task_id)?;
    let abs = root.resolve(&rel)?;
    if !abs.as_path().exists() {
        return Ok(None);
    }
    let bytes = read_limited(root, &rel)?;
    let text = String::from_utf8(bytes)
        .map_err(|e| CoreError::config(format!("invalid contract yaml: {e}")))?;
    parse_contract_yaml(&text).map(Some)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::discriminant;
    use tempfile::tempdir;

    const MINIMAL_YAML: &str = r#"
schemaVersion: 1
taskId: task-001
goal: "Do the thing"
scope:
  allowedPaths:
    - "crates/dare-contracts/src/handoff.rs"
acceptance:
  - id: A-01
    assertion: "It does the thing"
verification:
  requiredGates: ["build", "test"]
exitCriteria:
  kind: verified-deliverable
startingContext:
  source: project-revision
  repositories:
    - alias: dare-cli
"#;

    fn dag_ids(ids: &[&str]) -> BTreeSet<String> {
        ids.iter().map(|s| (*s).to_string()).collect()
    }

    fn minimal_contract() -> TaskContract {
        parse_contract_yaml(MINIMAL_YAML).expect("minimal contract parses")
    }

    fn sample_receipt() -> VerificationReceipt {
        VerificationReceipt {
            schema_version: HANDOFF_SCHEMA_VERSION,
            task_id: "task-001".to_string(),
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
            artifact_refs: vec!["crates/dare-contracts/src/handoff.rs".to_string()],
            artifact_digests: vec!["c".repeat(64)],
            verification_ref: ".dare/verification/task-001.json".to_string(),
            verification_digest: "d".repeat(64),
            verdict: Verdict::Passed,
            created_at: "2026-10-13T12:00:00Z".to_string(),
            producer: Producer {
                cli_version: "4.0.0".to_string(),
                driver: "cli".to_string(),
            },
        }
    }

    fn sample_record(kind: HandoffKind) -> HandoffRecord {
        HandoffRecord {
            schema_version: HANDOFF_SCHEMA_VERSION,
            task_id: "task-001".to_string(),
            kind,
            receipt: sample_receipt(),
            deliverable: "handoff module landed".to_string(),
            context_bounds: 2000,
            next_steps: vec!["task-003 consumes this".to_string()],
            open_items: Vec::new(),
            superseded: Vec::new(),
            extra: Map::new(),
        }
    }

    // --- parsing ---------------------------------------------------------

    #[test]
    fn parse_contract_roundtrip_preserves_extra() {
        let yaml = format!("{MINIMAL_YAML}customKey:\n  nested: 42\n");
        let parsed = parse_contract_yaml(&yaml).expect("parses");
        assert_eq!(parsed.extra.len(), 1);
        assert_eq!(
            parsed.extra.get("customKey").and_then(|v| v.get("nested")),
            Some(&Value::from(42))
        );

        let serialized = serde_yaml::to_string(&parsed).expect("serializes");
        let reparsed = parse_contract_yaml(&serialized).expect("reparses");
        assert_eq!(reparsed, parsed);
        assert_eq!(
            reparsed
                .extra
                .get("customKey")
                .and_then(|v| v.get("nested")),
            Some(&Value::from(42))
        );
    }

    #[test]
    fn parse_rejects_unsupported_schema_version() {
        let yaml = MINIMAL_YAML.replace("schemaVersion: 1", "schemaVersion: 2");
        let err = parse_contract_yaml(&yaml).expect_err("schema 2 is rejected");
        assert!(matches!(err, CoreError::Config(_)));
        assert!(err
            .to_string()
            .contains("unsupported contract schema version"));
    }

    #[test]
    fn parse_rejects_malformed_yaml() {
        let err = parse_contract_yaml("schemaVersion: 1\ngoal: [unclosed\n")
            .expect_err("malformed yaml is rejected");
        assert!(matches!(err, CoreError::Config(_)));
        assert!(err.to_string().contains("invalid contract yaml"));
    }

    #[test]
    fn parse_rejects_empty_document() {
        let err = parse_contract_yaml("").expect_err("empty document is rejected");
        assert!(matches!(err, CoreError::Config(_)));
    }

    #[test]
    fn handoff_record_roundtrip_preserves_extra() {
        let mut record = sample_record(HandoffKind::VerifiedDeliverable);
        record
            .extra
            .insert("auditTrail".to_string(), Value::from("kept"));
        let json = serde_json::to_string(&record).expect("serializes");
        assert!(json.contains("\"auditTrail\":\"kept\""));
        let back: HandoffRecord = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(back, record);
    }

    // --- validation ------------------------------------------------------

    #[test]
    fn validate_accepts_minimal_valid_contract() {
        let contract = minimal_contract();
        let violations = validate_contract(&contract, "task-001", &dag_ids(&["task-001"]));
        assert_eq!(violations, Vec::new());
    }

    #[test]
    fn validate_is_deterministic() {
        let mut contract = minimal_contract();
        contract.task_id = "../bad".to_string();
        contract.goal = "  ".to_string();
        contract.scope.allowed_paths = vec!["/abs".to_string(), "a[".to_string()];
        contract.acceptance = vec![
            AcceptanceCriterion {
                id: "A-01".to_string(),
                assertion: String::new(),
            },
            AcceptanceCriterion {
                id: "A-01".to_string(),
                assertion: "ok".to_string(),
            },
        ];
        contract.verification.required_gates = vec!["build".to_string(), "build".to_string()];

        let ids = dag_ids(&["task-001"]);
        let first = validate_contract(&contract, "task-001", &ids);
        let second = validate_contract(&contract, "task-001", &ids);
        assert_eq!(first, second);
        assert!(!first.is_empty());
        assert_eq!(first[0], ContractViolation::TaskIdUnsafe);
        assert_eq!(first[1], ContractViolation::TaskIdNotInDag);
        assert_eq!(first[2], ContractViolation::TaskIdFileMismatch);
        assert_eq!(first[3], ContractViolation::GoalEmpty);
    }

    #[test]
    fn validate_produces_every_violation() {
        let ids = dag_ids(&["task-001"]);
        let mut produced: Vec<ContractViolation> = Vec::new();

        // Case 1: unsafe id, missing from DAG, file mismatch, empty goal,
        // empty scope, empty acceptance, empty gates, unknown source.
        let mut c = minimal_contract();
        c.task_id = "../bad".to_string();
        c.goal = "   ".to_string();
        c.scope.allowed_paths = Vec::new();
        c.acceptance = Vec::new();
        c.verification.required_gates = Vec::new();
        c.starting_context.source = "telepathy".to_string();
        produced.extend(validate_contract(&c, "task-001", &ids));

        // Case 2: goal too long, too many paths.
        let mut c = minimal_contract();
        c.goal = "g".repeat(MAX_GOAL_CHARS + 1);
        c.scope.allowed_paths = (0..=MAX_ALLOWED_PATHS)
            .map(|i| format!("a{i}.rs"))
            .collect();
        produced.extend(validate_contract(&c, "task-001", &ids));

        // Case 3: too broad, absolute path, invalid glob.
        let mut c = minimal_contract();
        c.scope.allowed_paths = vec!["**".to_string(), "/etc/passwd".to_string()];
        c.scope.denied_paths = vec!["src/a[".to_string()];
        produced.extend(validate_contract(&c, "task-001", &ids));

        // Case 4: acceptance problems and an over-long assertion.
        let mut c = minimal_contract();
        c.acceptance = vec![
            AcceptanceCriterion {
                id: "bad".to_string(),
                assertion: "x".repeat(MAX_ASSERTION_CHARS + 1),
            },
            AcceptanceCriterion {
                id: "bad".to_string(),
                assertion: "  ".to_string(),
            },
        ];
        produced.extend(validate_contract(&c, "task-001", &ids));

        // Case 5: gate problems and a malformed repo alias.
        let mut c = minimal_contract();
        c.verification.required_gates =
            vec!["build".to_string(), "build".to_string(), "fmt".to_string()];
        c.starting_context.repositories = vec![RepoRef {
            alias: "Bad Alias".to_string(),
            repository_uri: None,
        }];
        produced.extend(validate_contract(&c, "task-001", &ids));

        // Case 6: a count outside its normative range.
        let mut c = minimal_contract();
        c.requirement_refs = (0..=MAX_REQUIREMENT_REFS)
            .map(|i| format!("RF-{i}"))
            .collect();
        produced.extend(validate_contract(&c, "task-001", &ids));

        // Case 7: a follow-up handoff with no open items.
        produced.extend(validate_handoff_record(&sample_record(
            HandoffKind::FollowUpHandoff,
        )));

        let expected = [
            ContractViolation::TaskIdUnsafe,
            ContractViolation::TaskIdNotInDag,
            ContractViolation::TaskIdFileMismatch,
            ContractViolation::GoalEmpty,
            ContractViolation::GoalTooLong,
            ContractViolation::AllowedPathsEmpty,
            ContractViolation::AllowedPathsTooMany,
            ContractViolation::AllowedPathsTooBroad,
            ContractViolation::GlobInvalid {
                index: 0,
                message: String::new(),
            },
            ContractViolation::PathNotRelative { index: 0 },
            ContractViolation::AcceptanceEmpty,
            ContractViolation::AcceptanceIdDuplicate { id: String::new() },
            ContractViolation::AcceptanceIdMalformed { id: String::new() },
            ContractViolation::AcceptanceAssertionEmpty { index: 0 },
            ContractViolation::RequiredGatesEmpty,
            ContractViolation::RequiredGatesDuplicate {
                aspect: String::new(),
            },
            ContractViolation::RequiredGatesUnknown {
                aspect: String::new(),
            },
            ContractViolation::RepoAliasMalformed {
                alias: String::new(),
            },
            ContractViolation::StartingContextSourceUnknown {
                source: String::new(),
            },
            ContractViolation::FieldOutOfRange {
                field: "x",
                min: 0,
                max: 0,
                actual: 0,
            },
            ContractViolation::OpenItemsRequiredForFollowUp,
        ];
        assert_eq!(expected.len(), 21);
        for want in &expected {
            assert!(
                produced
                    .iter()
                    .any(|got| discriminant(got) == discriminant(want)),
                "no case produced {want:?}"
            );
        }
        // Every produced violation must render a non-empty message.
        for violation in &produced {
            assert!(!violation.to_string().is_empty());
        }
    }

    #[test]
    fn validate_flags_broad_scope_as_warning_only() {
        let mut contract = minimal_contract();
        contract.scope.allowed_paths = vec!["**".to_string()];
        let violations = validate_contract(&contract, "task-001", &dag_ids(&["task-001"]));
        assert_eq!(violations, vec![ContractViolation::AllowedPathsTooBroad]);
        assert_eq!(violations[0].severity(), ViolationSeverity::Warning);
        assert_eq!(
            ContractViolation::GoalEmpty.severity(),
            ViolationSeverity::Error
        );
    }

    #[test]
    fn validate_flags_optional_field_lengths() {
        let mut contract = minimal_contract();
        contract.task_type = Some(String::new());
        contract.owner = Some("o".repeat(MAX_OWNER_CHARS + 1));
        let violations = validate_contract(&contract, "task-001", &dag_ids(&["task-001"]));
        assert_eq!(
            violations,
            vec![
                ContractViolation::FieldOutOfRange {
                    field: "taskType",
                    min: 1,
                    max: MAX_TASK_TYPE_CHARS,
                    actual: 0,
                },
                ContractViolation::FieldOutOfRange {
                    field: "owner",
                    min: 1,
                    max: MAX_OWNER_CHARS,
                    actual: MAX_OWNER_CHARS + 1,
                },
            ]
        );
    }

    #[test]
    fn validate_handoff_record_accepts_valid_records() {
        let record = sample_record(HandoffKind::VerifiedDeliverable);
        assert_eq!(validate_handoff_record(&record), Vec::new());

        let mut follow_up = sample_record(HandoffKind::FollowUpHandoff);
        follow_up.open_items = vec!["clippy still warns on windows".to_string()];
        assert_eq!(validate_handoff_record(&follow_up), Vec::new());
    }

    #[test]
    fn validate_handoff_record_caps_superseded() {
        let mut record = sample_record(HandoffKind::VerifiedDeliverable);
        record.superseded = (0..=MAX_SUPERSEDED)
            .map(|i| SupersededReceipt {
                receipt_digest: format!("{i:064}"),
                created_at: "2026-10-13T12:00:00Z".to_string(),
                reason: "re-run".to_string(),
            })
            .collect();
        assert_eq!(
            validate_handoff_record(&record),
            vec![ContractViolation::FieldOutOfRange {
                field: "superseded",
                min: 0,
                max: MAX_SUPERSEDED,
                actual: MAX_SUPERSEDED + 1,
            }]
        );
    }

    // --- predicates ------------------------------------------------------

    #[test]
    fn acceptance_id_regex_is_enforced() {
        assert!(acceptance_id_is_valid("A-01"));
        assert!(acceptance_id_is_valid("Z-123"));
        assert!(!acceptance_id_is_valid("a-01"));
        assert!(!acceptance_id_is_valid("A-1"));
        assert!(!acceptance_id_is_valid("A-1234"));
        assert!(!acceptance_id_is_valid("A01"));
        assert!(!acceptance_id_is_valid(""));
    }

    #[test]
    fn repo_alias_regex_is_enforced() {
        assert!(repo_alias_is_valid("dare-cli"));
        assert!(repo_alias_is_valid("a"));
        assert!(repo_alias_is_valid(&format!("a{}", "b".repeat(31))));
        assert!(!repo_alias_is_valid(&format!("a{}", "b".repeat(32))));
        assert!(!repo_alias_is_valid("Dare"));
        assert!(!repo_alias_is_valid("-dare"));
        assert!(!repo_alias_is_valid("dare_cli"));
        assert!(!repo_alias_is_valid(""));
    }

    #[test]
    fn relative_glob_predicate_rejects_escapes() {
        assert!(path_is_relative_glob("crates/**/*.rs"));
        assert!(!path_is_relative_glob(""));
        assert!(!path_is_relative_glob("/etc/passwd"));
        assert!(!path_is_relative_glob("\\\\server\\share"));
        assert!(!path_is_relative_glob("C:/windows"));
        assert!(!path_is_relative_glob("crates/../../etc"));
    }

    #[test]
    fn broad_glob_predicate_matches_whole_tree_only() {
        assert!(glob_is_too_broad("**"));
        assert!(glob_is_too_broad("*"));
        assert!(glob_is_too_broad("**/*"));
        assert!(!glob_is_too_broad("crates/**"));
        assert!(!glob_is_too_broad(""));
    }

    // --- paths -----------------------------------------------------------

    #[test]
    fn contract_dir_cycle_dag() {
        let dag = SafeRelativePath::new("DARE/cycles/x/dare-dag.yaml").expect("safe");
        let dir = contract_dir_for_dag(&dag).expect("resolves");
        assert_eq!(dir.as_str(), "DARE/cycles/x/contracts");
    }

    #[test]
    fn contract_dir_root_dag() {
        let dag = SafeRelativePath::new("dare-dag.yaml").expect("safe");
        let dir = contract_dir_for_dag(&dag).expect("resolves");
        assert_eq!(dir.as_str(), CONTRACT_DIR_NAME);
    }

    #[test]
    fn contract_dir_dot_rejected() {
        let dag = SafeRelativePath::new(".").expect("dot is the project root");
        let err = contract_dir_for_dag(&dag).expect_err("dot has no parent");
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    #[test]
    fn contract_and_handoff_rel_are_confined() {
        let dag = SafeRelativePath::new("DARE/cycles/x/dare-dag.yaml").expect("safe");
        let contract = contract_rel(&dag, "task-002").expect("resolves");
        assert_eq!(contract.as_str(), "DARE/cycles/x/contracts/task-002.yaml");
        let handoff = handoff_rel("task-002").expect("resolves");
        assert_eq!(handoff.as_str(), ".dare/handoff/task-002.json");
    }

    #[test]
    fn unsafe_task_ids_rejected() {
        let dag = SafeRelativePath::new("DARE/dare-dag.yaml").expect("safe");
        for bad in ["a/b", "../x", "-bad", ""] {
            let err = contract_rel(&dag, bad).expect_err("contract_rel rejects unsafe id");
            assert!(matches!(err, CoreError::InvalidInput(_)), "id {bad:?}");
            assert!(err.to_string().contains("unsafe handoff task id"));

            let err = handoff_rel(bad).expect_err("handoff_rel rejects unsafe id");
            assert!(matches!(err, CoreError::InvalidInput(_)), "id {bad:?}");
        }
    }

    // --- loading ---------------------------------------------------------

    #[test]
    fn load_contract_missing_is_none() {
        let dir = tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let dag = SafeRelativePath::new("DARE/dare-dag.yaml").expect("safe");
        assert_eq!(load_contract(&root, &dag, "task-001").expect("ok"), None);
    }

    #[test]
    fn load_contract_reads_existing_file() {
        let dir = tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let contracts = dir.path().join("DARE").join("contracts");
        std::fs::create_dir_all(&contracts).expect("mkdir");
        std::fs::write(contracts.join("task-001.yaml"), MINIMAL_YAML).expect("write");

        let dag = SafeRelativePath::new("DARE/dare-dag.yaml").expect("safe");
        let loaded = load_contract(&root, &dag, "task-001")
            .expect("ok")
            .expect("some");
        assert_eq!(loaded.task_id, "task-001");
        assert_eq!(loaded.exit_criteria.kind, HandoffKind::VerifiedDeliverable);
    }

    #[test]
    fn load_contract_oversize_rejected() {
        let dir = tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let contracts = dir.path().join("DARE").join("contracts");
        std::fs::create_dir_all(&contracts).expect("mkdir");

        let mut file = std::fs::File::create(contracts.join("task-001.yaml")).expect("create");
        {
            use std::io::Write;
            let chunk = vec![b'x'; 64 * 1024];
            for _ in 0..(crate::MAX_CONTRACT_BYTES / chunk.len() as u64 + 2) {
                file.write_all(&chunk).expect("write");
            }
        }
        drop(file);

        let dag = SafeRelativePath::new("DARE/dare-dag.yaml").expect("safe");
        let err = load_contract(&root, &dag, "task-001").expect_err("oversize is rejected");
        assert!(matches!(err, CoreError::Config(_)));
        assert!(err.to_string().contains("size limit"));
    }

    // --- enums -----------------------------------------------------------

    #[test]
    fn refusal_code_roundtrip() {
        assert_eq!(RefusalCode::ALL.len(), 13);
        let mut seen: BTreeSet<&'static str> = BTreeSet::new();
        for code in RefusalCode::ALL {
            let wire = code.as_str();
            assert!(wire.starts_with("HANDOFF_"), "{wire}");
            assert!(seen.insert(wire), "duplicate wire string {wire}");
            assert_eq!(RefusalCode::parse(wire), Some(code));
            assert_eq!(code.to_string(), wire);
        }
        assert_eq!(seen.len(), 13);
        assert_eq!(
            RefusalCode::ReceiptMissing.as_str(),
            "HANDOFF_RECEIPT_MISSING"
        );
        assert_eq!(RefusalCode::parse("HANDOFF_NOPE"), None);
        assert_eq!(RefusalCode::parse(""), None);
    }

    #[test]
    fn evidence_state_variants_are_distinct() {
        let all = [
            EvidenceState::Valid,
            EvidenceState::Missing,
            EvidenceState::Invalid,
            EvidenceState::Stale,
            EvidenceState::NotApplicable,
            EvidenceState::Insufficient,
        ];
        assert_eq!(all.len(), 6);
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                assert_eq!(i == j, a == b, "{a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn violation_severity_variants_are_distinct() {
        assert_ne!(ViolationSeverity::Error, ViolationSeverity::Warning);
        assert_eq!(ViolationSeverity::Error.as_str(), "error");
        assert_eq!(ViolationSeverity::Warning.as_str(), "warning");
        assert_eq!(ViolationSeverity::Warning.to_string(), "warning");
    }

    #[test]
    fn handoff_kind_wire_strings_are_stable() {
        for (kind, wire) in [
            (HandoffKind::VerifiedDeliverable, "verified-deliverable"),
            (HandoffKind::FollowUpHandoff, "follow-up-handoff"),
        ] {
            assert_eq!(kind.as_str(), wire);
            assert_eq!(kind.to_string(), wire);
            assert_eq!(
                serde_json::to_string(&kind).expect("serializes"),
                format!("\"{wire}\"")
            );
            let back: HandoffKind =
                serde_json::from_str(&format!("\"{wire}\"")).expect("deserializes");
            assert_eq!(back, kind);
        }
    }

    #[test]
    fn verdict_and_revision_state_wire_strings_are_stable() {
        for (verdict, wire) in [
            (Verdict::Passed, "passed"),
            (Verdict::Failed, "failed"),
            (Verdict::Insufficient, "insufficient"),
        ] {
            assert_eq!(verdict.as_str(), wire);
            assert_eq!(
                serde_json::to_string(&verdict).expect("serializes"),
                format!("\"{wire}\"")
            );
        }
        for (state, wire) in [
            (RevisionState::Clean, "clean"),
            (RevisionState::Dirty, "dirty"),
            (RevisionState::Unattested, "unattested"),
        ] {
            assert_eq!(state.as_str(), wire);
            let back: RevisionState =
                serde_json::from_str(&format!("\"{wire}\"")).expect("deserializes");
            assert_eq!(back, state);
        }
    }

    #[test]
    fn handoff_decision_invariant_holds_for_both_constructors() {
        let valid = HandoffDecision::valid("task-001", "deliverable text", Some("ab".to_string()));
        assert!(valid.invariant_holds());
        assert_eq!(valid.state, EvidenceState::Valid);
        assert_eq!(valid.deliverable.as_deref(), Some("deliverable text"));
        assert_eq!(valid.refusal, None);

        for state in [
            EvidenceState::Missing,
            EvidenceState::Invalid,
            EvidenceState::Stale,
            EvidenceState::NotApplicable,
            EvidenceState::Insufficient,
        ] {
            let refused = HandoffDecision::refused(
                "task-001",
                state,
                RefusalCode::ReceiptMissing,
                "no receipt on disk",
            );
            assert!(refused.invariant_holds());
            assert_eq!(refused.state, state);
            assert_eq!(refused.refusal, Some(RefusalCode::ReceiptMissing));
            assert_eq!(refused.deliverable, None);
        }

        // A refusal can never be Valid, even if the caller asks for it.
        let coerced = HandoffDecision::refused(
            "task-001",
            EvidenceState::Valid,
            RefusalCode::NotFinal,
            "parent still running",
        );
        assert_eq!(coerced.state, EvidenceState::Invalid);
        assert!(coerced.invariant_holds());

        // A hand-built decision that breaks the invariant is detected.
        let mut broken = HandoffDecision::valid("task-001", "x", None);
        broken.deliverable = None;
        assert!(!broken.invariant_holds());
    }

    #[test]
    fn handoff_policy_default_is_permissive() {
        let policy = HandoffPolicy::default();
        assert!(!policy.strict);
        assert!(!policy.require_attested_revision);
        assert!(!policy.require_contracts);
        assert_eq!(policy.required_kind, HandoffKind::VerifiedDeliverable);
    }

    #[test]
    fn constants_match_the_blueprint() {
        assert_eq!(CONTRACT_SCHEMA_VERSION, 1);
        assert_eq!(HANDOFF_SCHEMA_VERSION, 1);
        assert_eq!(HANDOFF_DIR_REL, ".dare/handoff");
        assert_eq!(CONTRACT_DIR_NAME, "contracts");
        assert_eq!(MAX_ARTIFACT_REFS, 512);
        assert_eq!(MAX_TREE_DIGEST_BYTES, 64 * 1024 * 1024);
        assert_eq!(MAX_SUPERSEDED, 16);
    }
}
