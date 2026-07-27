//! `dare execute --agent` — mock driver loop, worktrees, budget (microplano 030).

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dare_agent::{
    apply_fixed, failure_signature, resolve_driver, AgentRequest, AgentRunStatus, BudgetTracker,
    FixedDecision, WorktreeManager, MAX_AGENT_ATTEMPTS,
};
use dare_verify::{
    apply_decay, msg_policy_unknown, validate_best_of, BestOfWorktreeManager, DecayAction,
};
use dare_contracts::{load_dag, load_runtime_state, save_runtime_state, DagDocument};
use dare_core::{
    CancelFlag, CoreError, CoreResult, ProjectRoot, SafeRelativePath, SystemProcessRunner,
};
use dare_dag::{
    compose_task_prompt, compute_ranks, ensure_state, ready_at_min_rank, SystemClock,
    DEFAULT_DAG_REL, STATE_REL,
};
use dare_project::find_project_root;
use serde_json::{json, Value};

use crate::commands::execute::{
    complete_task_after_agent_silent, fail_task_after_agent, CompleteExit,
};
use crate::commands::path_resolve::resolve_project_rel;
use crate::output::OutputRenderer;

const MSG_NO_GIT: &str = "agent requires a git repository (.git missing)";
const MSG_AGENT_DONE: &str = "✅ Agent finished task {id} (decision=Done).";
const MSG_AGENT_STOP: &str = "⏹ Agent stopped task {id} (decision=Stop).";
const MSG_AGENT_BUDGET: &str = "Agent stopped — budget exhausted.";
const MSG_AGENT_EMPTY: &str = "No ready tasks for --agent (resolved or blocked).";
const MSG_CLEANUP_OK: &str = "✅ Cleaned up {n} agent worktree(s).";

pub struct AgentOpts {
    pub driver: String,
    pub task: Option<String>,
    pub budget_tokens: u64,
    pub policy: String,
    pub best_of: Option<u32>,
    pub prerank: bool,
}

/// `dare execute --cleanup-worktrees`
pub fn run_cleanup_worktrees(renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_cleanup_inner() {
        Ok((human, data)) => {
            if let Err(e) = renderer.write_success(&human, data) {
                return ExitCode::from(renderer.write_error(&e) as u8);
            }
            ExitCode::SUCCESS
        }
        Err(e) => ExitCode::from(renderer.write_error(&e) as u8),
    }
}

fn run_cleanup_inner() -> CoreResult<(String, Value)> {
    let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
    let Some(root_path) = find_project_root(&cwd) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&root_path)?;
    let mgr = WorktreeManager::new(root, Arc::new(SystemProcessRunner));
    let removed = mgr.cleanup_all()?;
    let human = MSG_CLEANUP_OK.replace("{n}", &removed.to_string());
    let data = json!({
        "action": "cleanup-worktrees",
        "removed": removed,
    });
    Ok((human, data))
}

pub fn run_agent(dag: Option<PathBuf>, opts: AgentOpts, renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_agent_inner(dag.clone(), opts) {
        Ok(AgentOutcome::DoneRalph {
            human,
            data,
            task_id,
            summary,
        }) => match complete_task_after_agent_silent(dag, &task_id, &summary) {
            Ok(()) => {
                if let Err(e) = renderer.write_success(&human, data) {
                    return ExitCode::from(renderer.write_error(&e) as u8);
                }
                ExitCode::SUCCESS
            }
            Err(CompleteExit::Timeout) => {
                let _ = renderer.write_error(&CoreError::internal(format!(
                    "Ralph timed out after agent Done on {task_id}"
                )));
                ExitCode::from(124)
            }
            Err(CompleteExit::Err(e)) => ExitCode::from(renderer.write_error(&e) as u8),
        },
        Ok(AgentOutcome::DoneSkipRalph { human, data }) => {
            if let Err(e) = renderer.write_success(&human, data) {
                return ExitCode::from(renderer.write_error(&e) as u8);
            }
            ExitCode::SUCCESS
        }
        Ok(AgentOutcome::Stopped { human, data }) => {
            let _ = renderer.write_report(&human, data, false);
            ExitCode::from(1)
        }
        Ok(AgentOutcome::Budget { human, data }) => {
            let _ = renderer.write_report(&human, data, false);
            ExitCode::from(1)
        }
        Ok(AgentOutcome::Empty { human, data }) => {
            if let Err(e) = renderer.write_success(&human, data) {
                return ExitCode::from(renderer.write_error(&e) as u8);
            }
            ExitCode::SUCCESS
        }
        Ok(AgentOutcome::Timeout { human }) => {
            let _ = renderer.write_error(&CoreError::internal(human));
            ExitCode::from(124)
        }
        Err(e) => ExitCode::from(renderer.write_error(&e) as u8),
    }
}

enum AgentOutcome {
    DoneRalph {
        human: String,
        data: Value,
        task_id: String,
        summary: String,
    },
    DoneSkipRalph {
        human: String,
        data: Value,
    },
    Stopped {
        human: String,
        data: Value,
    },
    Budget {
        human: String,
        data: Value,
    },
    Empty {
        human: String,
        data: Value,
    },
    Timeout {
        human: String,
    },
}

fn run_agent_inner(dag: Option<PathBuf>, opts: AgentOpts) -> CoreResult<AgentOutcome> {
    let use_decay = match opts.policy.as_str() {
        "fixed" => false,
        "decay" => true,
        other => return Err(CoreError::usage(msg_policy_unknown(other))),
    };

    if let Some(n) = opts.best_of {
        validate_best_of(n)?;
    }
    let _ = opts.prerank;

    let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
    let Some(root_path) = find_project_root(&cwd) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&root_path)?;

    if !root.as_path().join(".git").exists() {
        return Err(CoreError::invalid_input(MSG_NO_GIT));
    }

    let dag_rel = resolve_project_rel(&root, dag.as_deref(), DEFAULT_DAG_REL, true)?;
    let clock = SystemClock;
    let doc = load_dag(&root, &dag_rel)?;
    let state = ensure_state(&root, &doc, &clock)?;
    let ranks = compute_ranks(&doc)?;

    let driver = resolve_driver(&opts.driver)?;
    let _ = driver.doctor()?;
    dare_guard::run_preflight(&root, &dare_guard::PreflightOptions::default())?;

    let task_id = match &opts.task {
        Some(id) => {
            if !dag_has_task(&doc, id) {
                return Err(CoreError::not_found(format!("task not found in DAG: {id}")));
            }
            id.clone()
        }
        None => {
            let ready = ready_at_min_rank(&doc, &state, &ranks);
            match ready.first() {
                Some(id) => id.clone(),
                None => {
                    return Ok(AgentOutcome::Empty {
                        human: MSG_AGENT_EMPTY.into(),
                        data: json!({
                            "action": "agent",
                            "decision": "empty",
                            "readyCount": 0,
                        }),
                    });
                }
            }
        }
    };

    // Best-of-N: materialize cand worktrees under `.dare/worktrees/cand-{n}/`, then clean up.
    let mut best_of_specs = Vec::new();
    if let Some(n) = opts.best_of {
        let bo_mgr = BestOfWorktreeManager::new(root.clone(), Arc::new(SystemProcessRunner));
        for id in 1..=n {
            best_of_specs.push(bo_mgr.create(id)?);
        }
    }

    let mut budget = BudgetTracker::new(opts.budget_tokens);
    let cancel: CancelFlag = Arc::new(AtomicBool::new(false));
    let wt_mgr = WorktreeManager::new(root.clone(), Arc::new(SystemProcessRunner));

    let mut last_summary = String::new();
    let mut last_stderr = String::new();
    let mut last_status = AgentRunStatus::Failure;
    let mut last_worktree = String::new();
    let mut attempts_done = 0u32;
    let mut recent_signatures: Vec<String> = Vec::new();

    let loop_result = (|| -> CoreResult<AgentOutcome> {
        for attempt in 1..=MAX_AGENT_ATTEMPTS {
            if !budget.can_continue() {
                return Ok(AgentOutcome::Budget {
                    human: MSG_AGENT_BUDGET.to_string(),
                    data: agent_json(
                        &task_id,
                        driver.id(),
                        &opts.policy,
                        "budget_exhausted",
                        attempt.saturating_sub(1),
                        &budget,
                        &last_worktree,
                        None,
                        false,
                    ),
                });
            }
            if cancel.load(Ordering::SeqCst) {
                last_status = AgentRunStatus::Cancelled;
                break;
            }

            let spec = wt_mgr.create(&task_id, attempt)?;
            last_worktree = spec.rel_path.clone();
            let wt_abs = root.as_path().join(&spec.rel_path);

            let prompt = compose_task_prompt(&doc, &state, &task_id)?;
            let req = AgentRequest {
                task_id: task_id.clone(),
                prompt,
                cwd: PathBuf::from(wt_abs.as_str()),
                model: None,
                stdout_cap_chars: task_output_limit(&doc),
            };

            let result = driver.run(&req, &cancel)?;
            let _ = wt_mgr.remove(&spec);

            attempts_done = attempt;
            last_summary = result.summary.clone();
            last_stderr = result.stderr.clone();
            last_status = result.status;

            if result.status == AgentRunStatus::Timeout {
                return Ok(AgentOutcome::Timeout {
                    human: format!("Agent timed out on task {task_id}"),
                });
            }

            let _ = budget.consume(result.tokens.unwrap_or(0));

            if use_decay {
                let sig = failure_signature("agent", &result.stderr);
                let action = apply_decay(
                    result.status,
                    attempt,
                    &recent_signatures,
                    &sig,
                );
                if result.status == AgentRunStatus::Failure {
                    recent_signatures.push(sig);
                }
                match action {
                    DecayAction::Continue
                    | DecayAction::FreshStart
                    | DecayAction::Replan
                    | DecayAction::Escalate => continue,
                    DecayAction::Done => {
                        return agent_done_outcome(
                            &task_id,
                            driver.id(),
                            &opts.policy,
                            attempt,
                            &budget,
                            &last_worktree,
                            &result.summary,
                            result.tokens,
                        );
                    }
                    DecayAction::Stop => {
                        return stop_outcome(
                            &root,
                            &task_id,
                            driver.id(),
                            &opts.policy,
                            attempt,
                            &budget,
                            &last_worktree,
                            &last_summary,
                            &last_stderr,
                            last_status,
                        );
                    }
                }
            }

            let decision = apply_fixed(result.status, attempt, MAX_AGENT_ATTEMPTS);
            match decision {
                FixedDecision::Continue => continue,
                FixedDecision::Done => {
                    return agent_done_outcome(
                        &task_id,
                        driver.id(),
                        &opts.policy,
                        attempt,
                        &budget,
                        &last_worktree,
                        &result.summary,
                        result.tokens,
                    );
                }
                FixedDecision::Stop => {
                    return stop_outcome(
                        &root,
                        &task_id,
                        driver.id(),
                        &opts.policy,
                        attempt,
                        &budget,
                        &last_worktree,
                        &last_summary,
                        &last_stderr,
                        last_status,
                    );
                }
            }
        }

        stop_outcome(
            &root,
            &task_id,
            driver.id(),
            &opts.policy,
            attempts_done,
            &budget,
            &last_worktree,
            &last_summary,
            &last_stderr,
            last_status,
        )
    })();

    if !best_of_specs.is_empty() {
        let bo_mgr = BestOfWorktreeManager::new(root.clone(), Arc::new(SystemProcessRunner));
        for spec in &best_of_specs {
            let _ = bo_mgr.remove(spec);
        }
    }

    loop_result
}

#[allow(clippy::too_many_arguments)]
fn agent_done_outcome(
    task_id: &str,
    driver_id: &str,
    policy: &str,
    attempt: u32,
    budget: &BudgetTracker,
    worktree: &str,
    summary: &str,
    tokens: Option<u64>,
) -> CoreResult<AgentOutcome> {
    let ralph_skipped = std::env::var("DARE_AGENT_SKIP_RALPH")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let human = MSG_AGENT_DONE.replace("{id}", task_id);
    let data = agent_json(
        task_id,
        driver_id,
        policy,
        "done",
        attempt,
        budget,
        worktree,
        Some(json!({
            "status": "success",
            "summary": summary,
            "tokens": tokens,
        })),
        ralph_skipped,
    );
    if ralph_skipped {
        return Ok(AgentOutcome::DoneSkipRalph { human, data });
    }
    Ok(AgentOutcome::DoneRalph {
        human,
        data,
        task_id: task_id.to_string(),
        summary: summary.to_string(),
    })
}

#[allow(clippy::too_many_arguments)]
fn stop_outcome(
    root: &ProjectRoot,
    task_id: &str,
    driver_id: &str,
    policy: &str,
    attempts: u32,
    budget: &BudgetTracker,
    worktree: &str,
    summary: &str,
    stderr: &str,
    status: AgentRunStatus,
) -> CoreResult<AgentOutcome> {
    let sig = failure_signature("agent", stderr);
    let _ = fail_task_after_agent(None, task_id, summary);
    let _ = stamp_last_failure_signature(root, task_id, &sig);
    let status_s = match status {
        AgentRunStatus::Success => "success",
        AgentRunStatus::Failure => "failure",
        AgentRunStatus::Timeout => "timeout",
        AgentRunStatus::Cancelled => "cancelled",
    };
    let human = MSG_AGENT_STOP.replace("{id}", task_id);
    let data = agent_json(
        task_id,
        driver_id,
        policy,
        "stop",
        attempts,
        budget,
        worktree,
        Some(json!({
            "status": status_s,
            "summary": summary,
            "failureSignature": sig,
        })),
        false,
    );
    Ok(AgentOutcome::Stopped { human, data })
}

fn dag_has_task(doc: &DagDocument, task_id: &str) -> bool {
    dare_dag::iter_task_views(doc)
        .iter()
        .any(|v| v.id == task_id)
}

fn task_output_limit(doc: &DagDocument) -> usize {
    match doc {
        DagDocument::V21(d) => d.limits.task_output_chars as usize,
        DagDocument::Legacy(_) => 4000,
    }
}

#[allow(clippy::too_many_arguments)]
fn agent_json(
    task_id: &str,
    driver: &str,
    policy: &str,
    decision: &str,
    attempts: u32,
    budget: &BudgetTracker,
    worktree_path: &str,
    result: Option<Value>,
    ralph_skipped: bool,
) -> Value {
    let mut v = json!({
        "action": "agent",
        "taskId": task_id,
        "driver": driver,
        "policy": policy,
        "decision": decision,
        "attempts": attempts,
        "budget": {
            "limit": budget.limit(),
            "used": budget.used(),
        },
        "worktreePath": worktree_path,
        "result": result,
        "ralphSkipped": ralph_skipped,
    });
    if decision == "budget_exhausted" {
        v["reason"] = json!("budget_exhausted");
    }
    v
}

fn stamp_last_failure_signature(root: &ProjectRoot, task_id: &str, sig: &str) -> CoreResult<()> {
    let rel = SafeRelativePath::new(STATE_REL)?;
    let mut state = load_runtime_state(root, &rel)?;
    if let Some(task) = state.tasks.get_mut(task_id) {
        if let Some(last) = task.attempts.last_mut() {
            last.failure_signature = Some(sig.to_string());
            last.failed_aspect = Some("agent".into());
        }
    }
    save_runtime_state(root, &rel, &state)
}
