//! Best-of-N candidate worktrees + Pareto selection (Blueprint-049 §0.6).

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use dare_core::{CoreError, CoreResult, ProcessRunner, ProjectRoot, SafeCommand, SafeRelativePath};

use crate::report::{AspectStatus, BestOfCandidate, BestOfSummary, LoopVerdict};

/// Jail root for best-of candidate worktrees.
pub const WORKTREES_REL: &str = ".dare/worktrees";

/// Inclusive maximum for `--best-of`.
pub const BEST_OF_MAX: u32 = 8;

/// Inclusive minimum for `--best-of`.
pub const BEST_OF_MIN: u32 = 1;

/// Usage / invalid-input message when N is outside 1..=8.
pub const MSG_BEST_OF_RANGE: &str = "--best-of must be between 1 and 8";

/// Metrics used by [`pareto_select`] (deterministic; no RNG).
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateMetrics {
    pub id: u32,
    pub aspects_passed: u32,
    pub mutation_score: f64,
    pub duration_ms: u64,
}

/// Spec for a candidate git worktree under [`WORKTREES_REL`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BestOfWorktreeSpec {
    pub candidate_id: u32,
    pub branch: String,
    pub rel_path: String,
}

/// Git worktree manager for `.dare/worktrees/cand-{n}/` (SafeCommand argv only).
pub struct BestOfWorktreeManager {
    root: ProjectRoot,
    runner: Arc<dyn ProcessRunner>,
}

impl BestOfWorktreeManager {
    pub fn new(root: ProjectRoot, runner: Arc<dyn ProcessRunner>) -> Self {
        Self { root, runner }
    }

    /// Create worktree at `.dare/worktrees/cand-{id}/` (id in 1..=[`BEST_OF_MAX`]).
    pub fn create(&self, candidate_id: u32) -> CoreResult<BestOfWorktreeSpec> {
        validate_best_of(candidate_id)?;
        if !self.root.as_path().join(".git").exists() {
            return Err(CoreError::invalid_input(
                "best-of worktrees require a git repository (.git missing)",
            ));
        }

        let branch = format!("dare/bestof-cand-{candidate_id}");
        let rel_path = format!("{WORKTREES_REL}/cand-{candidate_id}");
        let rel = SafeRelativePath::new(&rel_path)?;

        let parent = self.root.as_path().join(WORKTREES_REL);
        std::fs::create_dir_all(&parent)
            .map_err(|e| CoreError::internal(format!("create best-of worktrees dir: {e}")))?;

        let cwd_rel = SafeRelativePath::new(".")?;
        let cmd = SafeCommand::new("git")
            .args([
                "worktree".into(),
                "add".into(),
                "-b".into(),
                branch.clone(),
                rel.as_str().to_string(),
                "HEAD".into(),
            ])
            .cwd(self.root.clone(), cwd_rel)
            .timeout(Duration::from_secs(120));

        let out = self.runner.run(&cmd)?;
        if out.exit_code != 0 {
            return Err(CoreError::internal(format!(
                "git worktree add failed (exit {}): {}",
                out.exit_code,
                out.stderr.trim()
            )));
        }

        Ok(BestOfWorktreeSpec {
            candidate_id,
            branch,
            rel_path,
        })
    }

    pub fn remove(&self, spec: &BestOfWorktreeSpec) -> CoreResult<()> {
        let rel = SafeRelativePath::new(&spec.rel_path)?;
        // Path jail: must stay under WORKTREES_REL/cand-*
        if !spec.rel_path.starts_with(&format!("{WORKTREES_REL}/cand-")) {
            return Err(CoreError::invalid_input(
                "best-of worktree path escapes jail",
            ));
        }
        let cwd_rel = SafeRelativePath::new(".")?;
        let cmd = SafeCommand::new("git")
            .args([
                "worktree".into(),
                "remove".into(),
                "--force".into(),
                rel.as_str().to_string(),
            ])
            .cwd(self.root.clone(), cwd_rel)
            .timeout(Duration::from_secs(120));

        let out = self.runner.run(&cmd)?;
        if out.exit_code != 0 {
            return Err(CoreError::internal(format!(
                "git worktree remove failed (exit {}): {}",
                out.exit_code,
                out.stderr.trim()
            )));
        }
        Ok(())
    }
}

/// Reject N outside [`BEST_OF_MIN`]..=[`BEST_OF_MAX`].
pub fn validate_best_of(n: u32) -> CoreResult<()> {
    if !(BEST_OF_MIN..=BEST_OF_MAX).contains(&n) {
        return Err(CoreError::invalid_input(MSG_BEST_OF_RANGE));
    }
    // Jail probe for cand path
    let probe = format!("{WORKTREES_REL}/cand-{n}");
    SafeRelativePath::new(&probe)?;
    if Path::new(&probe).components().count() != 3 {
        return Err(CoreError::invalid_input(MSG_BEST_OF_RANGE));
    }
    Ok(())
}

/// Whether candidate `c` Pareto-dominates `d` (§0.6).
pub fn dominates(c: &CandidateMetrics, d: &CandidateMetrics) -> bool {
    let ge_ap = c.aspects_passed >= d.aspects_passed;
    let ge_ms = c.mutation_score >= d.mutation_score;
    let le_dur = c.duration_ms <= d.duration_ms;
    let strict = c.aspects_passed > d.aspects_passed
        || c.mutation_score > d.mutation_score
        || c.duration_ms < d.duration_ms;
    ge_ap && ge_ms && le_dur && strict
}

/// Non-dominated candidate ids (sorted ASC).
pub fn pareto_front_ids(cands: &[CandidateMetrics]) -> Vec<u32> {
    let mut ids: Vec<u32> = cands
        .iter()
        .filter(|c| !cands.iter().any(|o| o.id != c.id && dominates(o, c)))
        .map(|c| c.id)
        .collect();
    ids.sort_unstable();
    ids
}

/// Winner id: Pareto front sorted by aspectsPassed↓, mutationScore↓, durationMs↑, id↑.
pub fn pareto_select(cands: &[CandidateMetrics]) -> u32 {
    assert!(!cands.is_empty(), "pareto_select requires at least one candidate");
    let front = pareto_front_ids(cands);
    let mut front_cands: Vec<&CandidateMetrics> = cands
        .iter()
        .filter(|c| front.contains(&c.id))
        .collect();
    front_cands.sort_by(|a, b| {
        b.aspects_passed
            .cmp(&a.aspects_passed)
            .then_with(|| {
                b.mutation_score
                    .partial_cmp(&a.mutation_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.duration_ms.cmp(&b.duration_ms))
            .then_with(|| a.id.cmp(&b.id))
    });
    front_cands[0].id
}

/// Build [`CandidateMetrics`] from a [`LoopVerdict`] aspects row set.
pub fn metrics_from_verdict(id: u32, verdict: &LoopVerdict) -> CandidateMetrics {
    let aspects_passed = verdict
        .aspects
        .iter()
        .filter(|a| a.status == AspectStatus::Pass)
        .count() as u32;
    let mutation_score = verdict
        .aspects
        .iter()
        .find(|a| {
            matches!(
                a.aspect,
                crate::report::AdvancedAspect::Mutation
            )
        })
        .and_then(|a| a.score)
        .unwrap_or(0.0);
    let duration_ms = verdict.aspects.iter().map(|a| a.duration_ms).sum();
    CandidateMetrics {
        id,
        aspects_passed,
        mutation_score,
        duration_ms,
    }
}

/// Attach a [`BestOfSummary`] to `verdict` from `cands` (caller supplies metrics).
pub fn attach_best_of(verdict: &mut LoopVerdict, n: u32, cands: &[CandidateMetrics]) {
    if cands.is_empty() {
        return;
    }
    let winner_id = pareto_select(cands);
    let pareto_ids = pareto_front_ids(cands);
    let candidates: Vec<BestOfCandidate> = cands
        .iter()
        .map(|c| BestOfCandidate {
            id: c.id,
            aspects_passed: c.aspects_passed,
            mutation_score: c.mutation_score,
            duration_ms: c.duration_ms,
            ok: c.aspects_passed > 0 || verdict.ok,
        })
        .collect();
    verdict.best_of = Some(BestOfSummary {
        n,
        candidates,
        pareto_ids,
        winner_id: Some(winner_id),
    });
    verdict.winner_id = Some(winner_id);
}

/// Create cand-1..N worktrees, synthesize identical metrics from `verdict`, select Pareto, cleanup.
///
/// Used by `--complete --best-of N` when candidates share the same tree state.
pub fn fill_best_of_with_worktrees(
    root: &ProjectRoot,
    n: u32,
    verdict: &mut LoopVerdict,
    runner: Arc<dyn ProcessRunner>,
) -> CoreResult<()> {
    validate_best_of(n)?;
    let mgr = BestOfWorktreeManager::new(root.clone(), runner);
    let mut specs = Vec::with_capacity(n as usize);
    for id in 1..=n {
        specs.push(mgr.create(id)?);
    }

    let base = metrics_from_verdict(0, verdict);
    let cands: Vec<CandidateMetrics> = (1..=n)
        .map(|id| CandidateMetrics {
            id,
            aspects_passed: base.aspects_passed,
            mutation_score: base.mutation_score,
            duration_ms: base.duration_ms,
        })
        .collect();
    attach_best_of(verdict, n, &cands);

    for spec in &specs {
        mgr.remove(spec)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use dare_core::{MockProcessRunner, ProcessOutput, SafeCommand};

    use super::*;
    use crate::report::{AdvancedAspect, AspectResult, AspectStatus, LOOP_VERDICT_SCHEMA};

    fn cand(id: u32, ap: u32, ms: f64, dur: u64) -> CandidateMetrics {
        CandidateMetrics {
            id,
            aspects_passed: ap,
            mutation_score: ms,
            duration_ms: dur,
        }
    }

    #[test]
    fn validate_best_of_range() {
        assert!(validate_best_of(1).is_ok());
        assert!(validate_best_of(8).is_ok());
        assert!(validate_best_of(0).is_err());
        assert!(validate_best_of(9).is_err());
        let err = validate_best_of(0).unwrap_err();
        assert!(err.to_string().contains(MSG_BEST_OF_RANGE));
    }

    #[test]
    fn pareto_domination_prefers_better_aspects() {
        let cands = vec![
            cand(1, 1, 0.5, 100),
            cand(2, 3, 0.5, 100),
            cand(3, 2, 0.5, 100),
        ];
        // 2 dominates 1 and 3 (more aspects, same mutation/duration)
        assert!(dominates(&cands[1], &cands[0]));
        assert!(dominates(&cands[1], &cands[2]));
        assert_eq!(pareto_front_ids(&cands), vec![2]);
        assert_eq!(pareto_select(&cands), 2);
    }

    #[test]
    fn pareto_duration_and_id_tiebreak() {
        // Neither dominates: same aspects/mutation; lower duration wins; then id ASC
        let cands = vec![
            cand(2, 2, 0.8, 50),
            cand(1, 2, 0.8, 50),
            cand(3, 2, 0.8, 90),
        ];
        // 3 is dominated by 1 and 2 (worse duration)
        assert!(dominates(&cands[0], &cands[2]));
        assert_eq!(pareto_front_ids(&cands), vec![1, 2]);
        // same keys → id ASC → 1
        assert_eq!(pareto_select(&cands), 1);
    }

    #[test]
    fn pareto_mutation_score_breaks_tie() {
        let cands = vec![cand(1, 2, 0.5, 10), cand(2, 2, 0.9, 10)];
        assert!(dominates(&cands[1], &cands[0]));
        assert_eq!(pareto_select(&cands), 2);
    }

    #[test]
    fn attach_best_of_fills_summary() {
        let mut v = LoopVerdict {
            schema_version: LOOP_VERDICT_SCHEMA,
            task_id: "t1".into(),
            ok: true,
            ralph_ok: true,
            policy: "fixed".into(),
            decay_action: "done".into(),
            best_of: None,
            winner_id: None,
            aspects: vec![AspectResult {
                aspect: AdvancedAspect::Mutation,
                status: AspectStatus::Pass,
                score: Some(0.8),
                reason: None,
                exit_code: Some(0),
                duration_ms: 12,
                stdout_tail: String::new(),
                stderr_tail: String::new(),
            }],
            failure_signature: None,
        };
        let cands = vec![
            metrics_from_verdict(1, &v),
            CandidateMetrics {
                id: 2,
                aspects_passed: 0,
                mutation_score: 0.1,
                duration_ms: 99,
            },
        ];
        attach_best_of(&mut v, 2, &cands);
        let summary = v.best_of.unwrap();
        assert_eq!(summary.n, 2);
        assert_eq!(summary.winner_id, Some(1));
        assert_eq!(v.winner_id, Some(1));
        assert!(summary.pareto_ids.contains(&1));
    }

    struct RecordingRunner {
        calls: Mutex<Vec<(String, Vec<String>)>>,
        inner: MockProcessRunner,
    }

    impl RecordingRunner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                inner: MockProcessRunner::new(),
            }
        }

        fn push_ok(&self) {
            self.inner.push(ProcessOutput {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                stdout_truncated: false,
                stderr_truncated: false,
                timed_out: false,
                cancelled: false,
            });
        }
    }

    impl ProcessRunner for RecordingRunner {
        fn run(&self, cmd: &SafeCommand) -> CoreResult<ProcessOutput> {
            self.calls
                .lock()
                .unwrap()
                .push((cmd.program().to_string(), cmd.arg_list().to_vec()));
            self.inner.run(cmd)
        }
    }

    #[test]
    fn worktree_create_argv_jail() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rec = Arc::new(RecordingRunner::new());
        rec.push_ok();
        let mgr = BestOfWorktreeManager::new(root, rec.clone());
        let spec = mgr.create(3).unwrap();
        assert_eq!(spec.rel_path, ".dare/worktrees/cand-3");
        assert_eq!(spec.branch, "dare/bestof-cand-3");
        let calls = rec.calls.lock().unwrap();
        assert_eq!(calls[0].0, "git");
        assert_eq!(
            calls[0].1,
            vec![
                "worktree",
                "add",
                "-b",
                "dare/bestof-cand-3",
                ".dare/worktrees/cand-3",
                "HEAD",
            ]
        );
    }
}
