//! `dare execute` — status / next / watch (028) + complete / fail / reset (029).

use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use dare_contracts::{
    handoff_rel, load_contract, load_dag, load_dare_config, load_runtime_state, material_digest,
    save_runtime_state, DagDocument, DareConfig, EvidenceState, HandoffDecision, HandoffPolicy,
    HandoffRecord, RefusalCode, RepoRevision, RevisionState, RuntimeStateV1, SourceRevision,
    TaskContract, HANDOFF_DIR_REL, HANDOFF_SCHEMA_VERSION,
};
use dare_core::{
    redact, truncate_chars, CoreError, CoreResult, MockProcessRunner, ProcessOutput, ProcessRunner,
    ProjectRoot, SafeRelativePath, SystemProcessRunner,
};
use dare_dag::{
    build_next_report, build_status_snapshot, compose_task_prompt, compose_task_prompt_verified,
    compute_ranks, ensure_state, iter_task_views, parent_context_limit, transition,
    write as write_canvas, Clock, ExecuteOutcome, NextReport, ParentEvidence, RefreshCanvas,
    StatusSnapshot, SystemClock, TaskStatus, Transition, CANVAS_REL, DEFAULT_DAG_REL, MSG_BLOCKED,
    MSG_EMPTY, MSG_RESOLVED, STATE_REL,
};
use dare_project::find_project_root;
use dare_verify::{
    attach_best_of, evaluate_parent, formal_enabled_from_cfg, load_handoff, metrics_from_verdict,
    receipt_from_verification, resolve_stack, run_advanced_verify, run_ralph, supersede_handoff,
    task_id_is_path_safe, validate_best_of, verification_from_ralph, verify_enabled_from_cfg,
    write_advanced_verdict, write_handoff, write_verification, AdvancedVerifyRequest,
    FormalBackend, LoopVerdict, RalphReport, ReceiptRequest, VERIFICATION_DIR_REL,
};
use serde_json::{json, Map, Value};

use crate::commands::handoff_policy::{handoff_policy_from_cfg, HandoffFlags};
use crate::commands::handoff_preflight::run_handoff_preflight;
use crate::commands::path_resolve::resolve_project_rel;
use crate::output::OutputRenderer;

/// Blueprint §0.2 human / default strings.
const MSG_OUTPUT_DEFAULT: &str = "Task completed.";
const MSG_REASON_DEFAULT: &str = "Task failed.";
const MSG_COMPLETE_OK_TMPL: &str = "✅ Task {id} marked DONE (Ralph passed).";
const MSG_COMPLETE_GATE_FAIL_TMPL: &str = "Ralph failed — task {id} left RUNNING (not DONE).";
const MSG_ADVANCED_GATE_FAIL_TMPL: &str =
    "Advanced verify failed — task {id} left RUNNING (not DONE).";
const MSG_FAIL_OK_TMPL: &str = "❌ Task {id} marked FAILED.";
const MSG_RESET_OK_TMPL: &str = "🔄 Task {id} reset to PENDING.";
const CONFIG_REL: &str = "dare.config.json";

/// Flags for advanced verify on `--complete` (Blueprint-049 §5.2).
#[derive(Debug, Clone)]
pub struct CompleteVerifyOpts {
    /// Explicit `--verify` (None = use config / default true).
    pub verify: Option<bool>,
    pub full_mutation: bool,
    /// Explicit `--formal` / `--no-formal` (None = use config / default false).
    pub formal: Option<bool>,
    pub formal_backend: FormalBackend,
    pub verdict_json: bool,
    /// `--best-of N` (1..=8); None = single path.
    pub best_of: Option<u32>,
    /// Soft prerank (no-op without best-of).
    pub prerank: bool,
}

impl Default for CompleteVerifyOpts {
    fn default() -> Self {
        Self {
            verify: None,
            full_mutation: false,
            formal: None,
            formal_backend: FormalBackend::Dafny,
            verdict_json: false,
            best_of: None,
            prerank: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExecuteAction {
    Status,
    Next,
    Watch {
        interval_secs: u64,
        max_ticks: Option<u64>,
    },
    Complete {
        id: String,
        output: Option<String>,
        verify_opts: CompleteVerifyOpts,
    },
    Fail {
        id: String,
        reason: Option<String>,
    },
    Reset {
        id: String,
    },
    Agent {
        driver: String,
        task: Option<String>,
        budget_tokens: u64,
        policy: String,
        best_of: Option<u32>,
        prerank: bool,
    },
    CleanupWorktrees,
}

pub fn run_execute(
    dag: Option<PathBuf>,
    action: ExecuteAction,
    handoff: HandoffFlags,
    renderer: &OutputRenderer<'_>,
) -> ExitCode {
    match action {
        ExecuteAction::Complete {
            id,
            output,
            verify_opts,
        } => run_complete(dag, id, output, verify_opts, renderer),
        ExecuteAction::Fail { id, reason } => run_fail(dag, id, reason, renderer),
        ExecuteAction::Reset { id } => run_reset(dag, id, renderer),
        ExecuteAction::Agent {
            driver,
            task,
            budget_tokens,
            policy,
            best_of,
            prerank,
        } => crate::commands::execute_agent::run_agent(
            dag,
            crate::commands::execute_agent::AgentOpts {
                driver,
                task,
                budget_tokens,
                policy,
                best_of,
                prerank,
            },
            handoff,
            renderer,
        ),
        ExecuteAction::CleanupWorktrees => {
            crate::commands::execute_agent::run_cleanup_worktrees(renderer)
        }
        other => match run_execute_inner(dag, other, handoff, renderer) {
            Ok(Some((human, data))) => {
                if let Err(e) = renderer.write_success(&human, data) {
                    return ExitCode::from(renderer.write_error(&e) as u8);
                }
                ExitCode::SUCCESS
            }
            Ok(None) => ExitCode::SUCCESS, // watch already streamed ticks
            Err(e) => ExitCode::from(renderer.write_error(&e) as u8),
        },
    }
}

fn run_execute_inner(
    dag: Option<PathBuf>,
    action: ExecuteAction,
    handoff: HandoffFlags,
    renderer: &OutputRenderer<'_>,
) -> CoreResult<Option<(String, Value)>> {
    let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
    let Some(root_path) = find_project_root(&cwd) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&root_path)?;
    let dag_rel = resolve_project_rel(&root, dag.as_deref(), DEFAULT_DAG_REL, true)?;
    let dag_display = dag_rel.as_str().to_string();
    let clock = SystemClock;

    match action {
        ExecuteAction::Status => {
            let doc = load_dag(&root, &dag_rel)?;
            let state = ensure_state(&root, &doc, &clock)?;
            let ranks = compute_ranks(&doc)?;
            write_canvas(&root, &doc, &state, Some(&ranks), &clock)?;
            let mut snap = build_status_snapshot(&doc, &state, &ranks);
            snap.dag_rel = dag_display.clone();
            snap.canvas_path = CANVAS_REL.to_string();
            let human = format_status_human(&snap);
            let data = status_json(&snap, "status");
            Ok(Some((human, data)))
        }
        ExecuteAction::Next => {
            let doc = load_dag(&root, &dag_rel)?;
            let state = ensure_state(&root, &doc, &clock)?;
            let ranks = compute_ranks(&doc)?;
            write_canvas(&root, &doc, &state, Some(&ranks), &clock)?;
            let mut report = build_next_report(&doc, &state, &ranks).map_err(CoreError::from)?;
            let cfg = load_config_soft(&root);
            let policy = handoff_policy_from_cfg(&cfg, &handoff);
            let (warnings, handoff_blocks) =
                apply_handoff_gate(&root, &dag_rel, &doc, &state, &mut report, &policy)?;
            emit_handoff_warnings(&warnings);
            let human = format_next_human(&report);
            let data = next_json(&report, &dag_display, &handoff_blocks);
            Ok(Some((human, data)))
        }
        ExecuteAction::Watch {
            interval_secs,
            max_ticks,
        } => {
            run_watch(
                &root,
                &dag_rel,
                &dag_display,
                interval_secs,
                max_ticks,
                renderer,
            )?;
            Ok(None)
        }
        ExecuteAction::Complete { .. }
        | ExecuteAction::Fail { .. }
        | ExecuteAction::Reset { .. }
        | ExecuteAction::Agent { .. }
        | ExecuteAction::CleanupWorktrees => {
            unreachable!("mutations/agent handled in run_execute")
        }
    }
}

/// Apêndice C — `--complete` with Ralph gates + advanced verify (§5.3).
fn run_complete(
    dag: Option<PathBuf>,
    task_id: String,
    output: Option<String>,
    verify_opts: CompleteVerifyOpts,
    renderer: &OutputRenderer<'_>,
) -> ExitCode {
    match run_complete_inner(dag, &task_id, output.as_deref(), &verify_opts) {
        Ok((human, data, verdict_json_line)) => {
            if let Some(line) = verdict_json_line {
                if let Err(e) = writeln_stdout(&line) {
                    return ExitCode::from(renderer.write_error(&e) as u8);
                }
            }
            if let Err(e) = renderer.write_success(&human, data) {
                return ExitCode::from(renderer.write_error(&e) as u8);
            }
            ExitCode::SUCCESS
        }
        Err(CompleteOutcome::TimedOut(e)) => {
            let _ = renderer.write_error(&e);
            ExitCode::from(124)
        }
        Err(CompleteOutcome::Other(e)) => ExitCode::from(renderer.write_error(&e) as u8),
    }
}

fn writeln_stdout(line: &str) -> CoreResult<()> {
    use std::io::Write;
    writeln!(std::io::stdout(), "{line}").map_err(|e| CoreError::io(e.to_string()))
}

enum CompleteOutcome {
    TimedOut(CoreError),
    Other(CoreError),
}

fn run_complete_inner(
    dag: Option<PathBuf>,
    task_id: &str,
    output: Option<&str>,
    verify_opts: &CompleteVerifyOpts,
) -> Result<(String, Value, Option<String>), CompleteOutcome> {
    let ctx = prepare_mutation_ctx(dag).map_err(CompleteOutcome::Other)?;
    let PrepareCtx {
        root,
        dag_rel,
        doc,
        clock,
        mut state,
    } = ctx;

    if !dag_has_task(&doc, task_id) {
        return Err(CompleteOutcome::Other(CoreError::not_found(format!(
            "task not found in DAG: {task_id}"
        ))));
    }
    if !task_id_is_path_safe(task_id) {
        return Err(CompleteOutcome::Other(CoreError::invalid_input(format!(
            "unsafe verification task id: {task_id}"
        ))));
    }

    let stack = resolve_stack(&root).map_err(CompleteOutcome::Other)?;
    // Validate stack is implemented before mutating status.
    dare_verify::gate_commands(&stack).map_err(CompleteOutcome::Other)?;

    ensure_running(&root, &dag_rel, &doc, &state, task_id, &clock)
        .map_err(CompleteOutcome::Other)?;
    // Refresh state after possible Start.
    state = load_state_soft(&root);
    let status = task_status(&state, task_id).map_err(CompleteOutcome::Other)?;
    if status != TaskStatus::Running {
        return Err(CompleteOutcome::Other(CoreError::invalid_input(format!(
            "invalid transition Complete from {}",
            status.as_str()
        ))));
    }

    let cap = task_output_limit(&doc);
    let runner = ralph_runner_from_env();
    let ralph = run_ralph(&root, &stack, runner.as_ref(), cap).map_err(CompleteOutcome::Other)?;

    let now = clock.now_rfc3339();
    let verif = verification_from_ralph(task_id, &ralph, &now);
    write_verification(&root, &verif).map_err(CompleteOutcome::Other)?;

    if ralph.timed_out {
        let msg = format_complete_gate_fail(task_id, &ralph);
        return Err(CompleteOutcome::TimedOut(CoreError::internal(msg)));
    }
    if !ralph.ok {
        let msg = format_complete_gate_fail(task_id, &ralph);
        return Err(CompleteOutcome::Other(CoreError::internal(msg)));
    }

    // Advanced verify after Ralph ok (separate runner so DARE_RALPH_MOCK queue stays Ralph-only).
    let cfg = load_config_soft(&root);
    let adv_req = build_advanced_request(task_id, verify_opts, &cfg);
    let adv_runner = SystemProcessRunner;
    let mut verdict =
        run_advanced_verify(&root, &cfg, &adv_req, &adv_runner).map_err(CompleteOutcome::Other)?;
    if let Some(n) = verify_opts.best_of {
        validate_best_of(n).map_err(CompleteOutcome::Other)?;
        let _ = verify_opts.prerank; // soft: order only; DONE still gated by verdict.ok
        let cands: Vec<_> = (1..=n)
            .map(|id| metrics_from_verdict(id, &verdict))
            .collect();
        attach_best_of(&mut verdict, n, &cands);
    }
    if adv_req.verify {
        write_advanced_verdict(&root, &verdict).map_err(CompleteOutcome::Other)?;
    }
    if !verdict.ok {
        let msg = format_advanced_gate_fail(task_id, &verdict);
        return Err(CompleteOutcome::Other(CoreError::internal(msg)));
    }

    let raw_output = output.unwrap_or(MSG_OUTPUT_DEFAULT);
    let (truncated, _) = truncate_chars(raw_output.to_string(), cap);
    let truncated_for_handoff = truncated.clone();
    transition(
        &root,
        &doc,
        task_id,
        Transition::Complete { output: truncated },
        &clock,
        RefreshCanvas::Yes,
    )
    .map_err(CompleteOutcome::Other)?;

    let final_verif = verification_from_ralph(task_id, &ralph, &clock.now_rfc3339());
    // ok already true; overwrite for idempotent final artifact.
    write_verification(&root, &final_verif).map_err(CompleteOutcome::Other)?;

    // The task is DONE from here on. A failure below must report honestly and
    // leave the status alone: reverting a legitimate DONE because a sidecar
    // write failed would be worse than the failure.
    let emitted = emit_handoff_on_complete(
        &root,
        &dag_rel,
        &doc,
        task_id,
        &truncated_for_handoff,
        &clock,
    )
    .map_err(|e| CompleteOutcome::Other(receipt_failure_error(task_id, &e)))?;

    let human = MSG_COMPLETE_OK_TMPL.replace("{id}", task_id);
    let data = complete_json(task_id, &ralph, &verdict, emitted.as_ref());
    let verdict_line = if verify_opts.verdict_json {
        Some(
            serde_json::to_string(&verdict)
                .map_err(|e| CompleteOutcome::Other(CoreError::internal(e.to_string())))?,
        )
    } else {
        None
    };
    Ok((human, data, verdict_line))
}

fn load_config_soft(root: &ProjectRoot) -> DareConfig {
    let Ok(rel) = SafeRelativePath::new(CONFIG_REL) else {
        return DareConfig::default();
    };
    load_dare_config(root, &rel).unwrap_or_default()
}

fn build_advanced_request(
    task_id: &str,
    opts: &CompleteVerifyOpts,
    cfg: &DareConfig,
) -> AdvancedVerifyRequest {
    let verify = opts.verify.unwrap_or_else(|| verify_enabled_from_cfg(cfg));
    let formal = opts.formal.unwrap_or_else(|| formal_enabled_from_cfg(cfg));
    AdvancedVerifyRequest {
        task_id: task_id.to_string(),
        full_mutation: opts.full_mutation,
        formal,
        formal_backend: opts.formal_backend,
        verify,
    }
}

fn run_fail(
    dag: Option<PathBuf>,
    task_id: String,
    reason: Option<String>,
    renderer: &OutputRenderer<'_>,
) -> ExitCode {
    match run_fail_inner(dag, &task_id, reason.as_deref()) {
        Ok((human, data)) => {
            if let Err(e) = renderer.write_success(&human, data) {
                return ExitCode::from(renderer.write_error(&e) as u8);
            }
            ExitCode::SUCCESS
        }
        Err(e) => ExitCode::from(renderer.write_error(&e) as u8),
    }
}

fn run_fail_inner(
    dag: Option<PathBuf>,
    task_id: &str,
    reason: Option<&str>,
) -> CoreResult<(String, Value)> {
    let PrepareCtx {
        root,
        dag_rel,
        doc,
        clock,
        state,
    } = prepare_mutation_ctx(dag)?;

    if !dag_has_task(&doc, task_id) {
        return Err(CoreError::not_found(format!(
            "task not found in DAG: {task_id}"
        )));
    }

    ensure_running(&root, &dag_rel, &doc, &state, task_id, &clock)?;

    let cap = task_output_limit(&doc);
    let raw = reason.unwrap_or(MSG_REASON_DEFAULT);
    let (truncated, _) = truncate_chars(raw.to_string(), cap);
    transition(
        &root,
        &doc,
        task_id,
        Transition::Fail {
            error: truncated.clone(),
        },
        &clock,
        RefreshCanvas::Yes,
    )?;

    let human = MSG_FAIL_OK_TMPL.replace("{id}", task_id);
    let data = json!({
        "action": "fail",
        "taskId": task_id,
        "status": "FAILED",
        "reason": truncated,
    });
    Ok((human, data))
}

fn run_reset(dag: Option<PathBuf>, task_id: String, renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_reset_inner(dag, &task_id) {
        Ok((human, data)) => {
            if let Err(e) = renderer.write_success(&human, data) {
                return ExitCode::from(renderer.write_error(&e) as u8);
            }
            ExitCode::SUCCESS
        }
        Err(e) => ExitCode::from(renderer.write_error(&e) as u8),
    }
}

fn run_reset_inner(dag: Option<PathBuf>, task_id: &str) -> CoreResult<(String, Value)> {
    let PrepareCtx {
        root,
        doc,
        clock,
        state,
        ..
    } = prepare_mutation_ctx(dag)?;

    if !dag_has_task(&doc, task_id) {
        return Err(CoreError::not_found(format!(
            "task not found in DAG: {task_id}"
        )));
    }

    let attempts_before = state
        .tasks
        .get(task_id)
        .map(|t| t.attempts.len())
        .unwrap_or(0);

    transition(
        &root,
        &doc,
        task_id,
        Transition::Reset,
        &clock,
        RefreshCanvas::Yes,
    )?;

    // Unconditional: idempotent when no sidecar exists, and the point is that
    // a reset never discards the previous receipt without recording it.
    supersede_handoff(&root, task_id, "task reset", &clock.now_rfc3339())?;

    let human = MSG_RESET_OK_TMPL.replace("{id}", task_id);
    let data = json!({
        "action": "reset",
        "taskId": task_id,
        "status": "PENDING",
        "attemptsPreserved": true,
        "attemptsBefore": attempts_before,
    });
    Ok((human, data))
}

struct PrepareCtx {
    root: ProjectRoot,
    dag_rel: SafeRelativePath,
    doc: DagDocument,
    clock: SystemClock,
    state: RuntimeStateV1,
}

fn prepare_mutation_ctx(dag: Option<PathBuf>) -> CoreResult<PrepareCtx> {
    let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
    let Some(root_path) = find_project_root(&cwd) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&root_path)?;
    let dag_rel = resolve_project_rel(&root, dag.as_deref(), DEFAULT_DAG_REL, true)?;
    let clock = SystemClock;
    let doc = load_dag(&root, &dag_rel)?;
    let state = ensure_state(&root, &doc, &clock)?;
    Ok(PrepareCtx {
        root,
        dag_rel,
        doc,
        clock,
        state,
    })
}

fn ensure_running(
    root: &ProjectRoot,
    dag_rel: &SafeRelativePath,
    doc: &DagDocument,
    state: &RuntimeStateV1,
    task_id: &str,
    clock: &dyn Clock,
) -> CoreResult<()> {
    let status = task_status(state, task_id)?;
    match status {
        TaskStatus::Pending => {
            transition(
                root,
                doc,
                task_id,
                Transition::Start,
                clock,
                RefreshCanvas::No,
            )?;
            // The digest of the contract the work starts against, so a material
            // change before `--complete` can be detected rather than adopted.
            record_start_provenance(root, dag_rel, task_id, clock)?;
            Ok(())
        }
        TaskStatus::Running => Ok(()),
        other => Err(CoreError::invalid_input(format!(
            "invalid transition: task is {}",
            other.as_str()
        ))),
    }
}

fn task_status(state: &RuntimeStateV1, task_id: &str) -> CoreResult<TaskStatus> {
    let task = state
        .tasks
        .get(task_id)
        .ok_or_else(|| CoreError::not_found(format!("task not found in DAG: {task_id}")))?;
    TaskStatus::parse(&task.status)
}

fn dag_has_task(doc: &DagDocument, task_id: &str) -> bool {
    iter_task_views(doc).iter().any(|v| v.id == task_id)
}

fn task_output_limit(doc: &DagDocument) -> usize {
    match doc {
        DagDocument::V21(d) => d.limits.task_output_chars as usize,
        DagDocument::Legacy(_) => 4000,
    }
}

/// Apêndice E — `DARE_RALPH_MOCK` test harness.
fn ralph_runner_from_env() -> Box<dyn ProcessRunner> {
    match std::env::var("DARE_RALPH_MOCK")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "" => Box::new(SystemProcessRunner),
        "1" | "pass" => {
            let mock = MockProcessRunner::new();
            mock.push(mock_ok());
            mock.push(mock_ok());
            mock.push(mock_ok());
            Box::new(mock)
        }
        "fail" => {
            let mock = MockProcessRunner::new();
            mock.push(mock_fail(1));
            Box::new(mock)
        }
        "timeout" => {
            let mock = MockProcessRunner::new();
            mock.push(mock_timeout());
            Box::new(mock)
        }
        _ => Box::new(SystemProcessRunner),
    }
}

fn mock_ok() -> ProcessOutput {
    ProcessOutput {
        exit_code: 0,
        stdout: String::new(),
        stderr: String::new(),
        stdout_truncated: false,
        stderr_truncated: false,
        timed_out: false,
        cancelled: false,
    }
}

fn mock_fail(code: i32) -> ProcessOutput {
    ProcessOutput {
        exit_code: code,
        stdout: String::new(),
        stderr: "mock fail".into(),
        stdout_truncated: false,
        stderr_truncated: false,
        timed_out: false,
        cancelled: false,
    }
}

fn mock_timeout() -> ProcessOutput {
    ProcessOutput {
        exit_code: 124,
        stdout: String::new(),
        stderr: "mock timeout".into(),
        stdout_truncated: false,
        stderr_truncated: false,
        timed_out: true,
        cancelled: false,
    }
}

/// Used by `--agent` Done path to run Ralph/complete (029) without printing.
pub(crate) fn complete_task_after_agent_silent(
    dag: Option<PathBuf>,
    task_id: &str,
    output: &str,
) -> Result<(), CompleteExit> {
    match run_complete_inner(dag, task_id, Some(output), &CompleteVerifyOpts::default()) {
        Ok(_) => Ok(()),
        Err(CompleteOutcome::TimedOut(_)) => Err(CompleteExit::Timeout),
        Err(CompleteOutcome::Other(e)) => Err(CompleteExit::Err(e)),
    }
}

#[derive(Debug)]
pub(crate) enum CompleteExit {
    Timeout,
    Err(CoreError),
}

/// Used by `--agent` Stop path to stamp Fail without printing success.
pub(crate) fn fail_task_after_agent(
    dag: Option<PathBuf>,
    task_id: &str,
    reason: &str,
) -> CoreResult<()> {
    run_fail_inner(dag, task_id, Some(reason)).map(|_| ())
}

fn format_complete_gate_fail(id: &str, ralph: &RalphReport) -> String {
    let mut msg = MSG_COMPLETE_GATE_FAIL_TMPL.replace("{id}", id);
    if let Some(step) = ralph.steps.iter().find(|s| s.timed_out || s.exit_code != 0) {
        msg.push_str(&format!(
            " ({} exit={})",
            step.aspect.as_str(),
            step.exit_code
        ));
    }
    msg
}

fn format_advanced_gate_fail(id: &str, verdict: &LoopVerdict) -> String {
    let mut msg = MSG_ADVANCED_GATE_FAIL_TMPL.replace("{id}", id);
    if let Some(step) = verdict
        .aspects
        .iter()
        .find(|a| a.status == dare_verify::AspectStatus::Fail)
    {
        let reason = step.reason.as_deref().unwrap_or("fail");
        msg.push_str(&format!(" ({}: {reason})", step.aspect.as_str()));
    }
    msg
}

fn complete_json(
    task_id: &str,
    ralph: &RalphReport,
    verdict: &LoopVerdict,
    emitted: Option<&(String, Value)>,
) -> Value {
    let verification_path = format!("{VERIFICATION_DIR_REL}/{task_id}.json");
    let advanced_path = format!("{VERIFICATION_DIR_REL}/{task_id}.advanced.json");
    let ralph_val = serde_json::to_value(ralph).unwrap_or_else(|_| json!({}));
    let verdict_val = serde_json::to_value(verdict).unwrap_or_else(|_| json!({}));
    let mut data = json!({
        "action": "complete",
        "taskId": task_id,
        "status": "DONE",
        "verificationPath": verification_path,
        "advancedVerificationPath": advanced_path,
        "ralph": ralph_val,
        "verdict": verdict_val,
    });
    // Additive, and omitted entirely rather than emitted as null, so a run
    // without a contract stays byte-identical to what it produced before.
    if let (Some(obj), Some((path, receipt))) = (data.as_object_mut(), emitted) {
        obj.insert("handoffPath".to_string(), json!(path));
        obj.insert("receipt".to_string(), receipt.clone());
    }
    data
}

fn run_watch(
    root: &ProjectRoot,
    dag_rel: &dare_core::SafeRelativePath,
    dag_display: &str,
    interval_secs: u64,
    max_ticks: Option<u64>,
    renderer: &OutputRenderer<'_>,
) -> CoreResult<()> {
    let mut tick: u64 = 0;
    loop {
        tick += 1;
        let doc = load_dag(root, dag_rel)?;
        let state = load_state_soft(root);
        let ranks = compute_ranks(&doc)?;
        let mut snap = build_status_snapshot(&doc, &state, &ranks);
        snap.dag_rel = dag_display.to_string();
        snap.canvas_path = CANVAS_REL.to_string();
        let human = format_status_human(&snap);
        let data = status_json(&snap, "watch");
        renderer.write_success(&human, data)?;

        if max_ticks.is_some_and(|m| tick >= m) {
            break;
        }
        if interval_secs > 0 {
            thread::sleep(Duration::from_secs(interval_secs));
        }
    }
    Ok(())
}

fn load_state_soft(root: &ProjectRoot) -> RuntimeStateV1 {
    match dare_core::SafeRelativePath::new(STATE_REL) {
        Ok(rel) => load_runtime_state(root, &rel).unwrap_or_else(|_| empty_state()),
        Err(_) => empty_state(),
    }
}

fn empty_state() -> RuntimeStateV1 {
    RuntimeStateV1 {
        version: 1,
        updated_at: String::new(),
        tasks: Default::default(),
        extra: Map::new(),
    }
}

fn format_status_human(snap: &StatusSnapshot) -> String {
    if snap.outcome == ExecuteOutcome::Empty {
        return MSG_EMPTY.to_string();
    }
    let c = &snap.counts;
    format!(
        "📊 {}\n\n  ✅ DONE     : {}\n  🔄 RUNNING  : {}\n  ⏳ PENDING  : {}\n  ❌ FAILED   : {}\n  ⏭️  SKIPPED  : {}\n\n  📄 Canvas: {}\n",
        snap.title, c.done, c.running, c.pending, c.failed, c.skipped, snap.canvas_path
    )
}

fn format_next_human(report: &NextReport) -> String {
    match report.outcome {
        ExecuteOutcome::Empty => MSG_EMPTY.to_string(),
        ExecuteOutcome::Resolved => MSG_RESOLVED.to_string(),
        ExecuteOutcome::Blocked => format!("{MSG_BLOCKED} (PENDING remain with unmet deps)."),
        ExecuteOutcome::Waiting => "No executable tasks (work in progress).".to_string(),
        ExecuteOutcome::Status => MSG_EMPTY.to_string(),
        ExecuteOutcome::NextReady => {
            let rank = report.rank.unwrap_or(0);
            let mut out = format!(
                "📦 Rank {rank} — {} task(s) ready in parallel\n\n",
                report.ready.len()
            );
            for t in &report.ready {
                out.push_str(&format!("▸ {} — {}\n", t.id, t.title));
                out.push_str(&format!("  complexity: {}\n", t.complexity));
                if !t.spec_file.is_empty() {
                    out.push_str(&format!("  spec_file:  {}\n", t.spec_file));
                }
                out.push_str("  prompt:\n");
                for line in t.prompt.lines() {
                    out.push_str(&format!("    {line}\n"));
                }
                out.push('\n');
            }
            out.push_str(
                "Next steps for the IDE agent:\n  1. Execute each task above.\n  2. After each task: `dare execute --complete <id> --output \"<summary>\"`\n",
            );
            out
        }
    }
}

fn status_json(snap: &StatusSnapshot, action: &str) -> Value {
    let tasks: Vec<Value> = snap
        .tasks
        .iter()
        .map(|t| {
            json!({
                "complexity": t.complexity,
                "id": t.id,
                "rank": t.rank,
                "status": t.status,
                "title": t.title,
            })
        })
        .collect();
    let c = &snap.counts;
    json!({
        "action": action,
        "canvasPath": snap.canvas_path,
        "counts": {
            "done": c.done,
            "failed": c.failed,
            "pending": c.pending,
            "running": c.running,
            "skipped": c.skipped,
            "total": c.total,
        },
        "dag": snap.dag_rel,
        "outcome": snap.outcome.as_str(),
        "tasks": tasks,
    })
}

fn next_json(report: &NextReport, dag: &str, handoff_blocks: &BTreeMap<String, Value>) -> Value {
    let ready: Vec<Value> = report
        .ready
        .iter()
        .map(|t| {
            // Additive: every key already emitted keeps its name and place.
            let mut item = json!({
                "complexity": t.complexity,
                "id": t.id,
                "prompt": t.prompt,
                "rank": t.rank,
                "specFile": t.spec_file,
                "title": t.title,
            });
            if let (Some(obj), Some(block)) = (item.as_object_mut(), handoff_blocks.get(&t.id)) {
                obj.insert("handoff".to_string(), block.clone());
            }
            item
        })
        .collect();
    json!({
        "action": "next",
        "blocked": report.outcome == ExecuteOutcome::Blocked,
        "dag": dag,
        "outcome": report.outcome.as_str(),
        "rank": report.rank,
        "ready": ready,
        "resolved": report.outcome == ExecuteOutcome::Resolved,
    })
}

/// Writes compatibility-mode warnings to stderr, redacted.
///
/// stdout is deliberately untouched: the golden baseline of task-001 asserts it
/// byte for byte, and a warning must never change what a pipe receives.
pub fn emit_handoff_warnings(warnings: &[String]) {
    use std::io::Write as _;
    for w in warnings {
        let _ = writeln!(std::io::stderr(), "{}", redact(w));
    }
}

// ---------------------------------------------------------------------------
// Handoff emission (task-010)
// ---------------------------------------------------------------------------

/// Key under a task's `extra` in `.dare/state.json` holding the material digest
/// recorded when the task started.
///
/// It lives in `extra` and not as a struct field so `.dare/state.json` gains no
/// required key (RF-10). The npm runtime rebuilds that file from the fields it
/// knows and drops the rest, so after a round trip through npm the digest is
/// simply absent — which is read as "nothing was recorded", never as a
/// divergence.
const STATE_KEY_CONTRACT_DIGEST: &str = "handoffContractDigest";

/// Alias used for the repository when the contract names none.
const SELF_REPO_ALIAS: &str = "self";

/// Refusal shown when the material fields of a contract moved mid-flight.
///
/// It names the recovery command, because a digest mismatch without a way
/// forward is a dead end for the operator.
const MSG_CONTRACT_CHANGED_TMPL: &str = concat!(
    "contract changed after task start: {id} (HANDOFF_CONTRACT_DIGEST_MISMATCH)\n",
    "       material fields (goal/scope/acceptance/exitCriteria) differ from the recorded digest\n",
    "       to adopt the new contract: dare execute --reset {id}"
);

/// Reported when the task legitimately reached DONE but its sidecar could not
/// be written. The status is deliberately left alone.
const MSG_RECEIPT_NOT_WRITTEN_TMPL: &str = concat!(
    "task {id} is DONE but the handoff receipt was not written: {why}\n",
    "       the task status is correct and was left untouched; ",
    "re-run `dare execute --complete {id}` to emit the receipt"
);

/// Key under a task's `extra` holding the HEAD sha the task started from.
///
/// Optional for the same reason as [`STATE_KEY_CONTRACT_DIGEST`]: absent means
/// "not recorded", and the changed-file set is then simply empty.
const STATE_KEY_START_SHA: &str = "handoffStartSha";

/// Records the contract digest and the starting revision on the task's entry.
///
/// Called on the `Start` transition so the complete path has something to
/// compare against. No contract means nothing to record at all.
fn record_start_provenance(
    root: &ProjectRoot,
    dag_rel: &SafeRelativePath,
    task_id: &str,
    clock: &dyn Clock,
) -> CoreResult<()> {
    let Some(contract) = load_contract(root, dag_rel, task_id)? else {
        return Ok(());
    };
    let digest = material_digest(&contract)?;
    let head = dare_project::git::head_sha(root);
    let mut state = load_state_soft(root);
    let Some(entry) = state.tasks.get_mut(task_id) else {
        return Ok(());
    };
    entry
        .extra
        .insert(STATE_KEY_CONTRACT_DIGEST.to_string(), json!(digest));
    if let Some(sha) = head {
        entry
            .extra
            .insert(STATE_KEY_START_SHA.to_string(), json!(sha));
    }
    state.updated_at = clock.now_rfc3339();
    let rel = SafeRelativePath::new(STATE_REL)?;
    save_runtime_state(root, &rel, &state)
}

/// The digest recorded at `Start`, if any survived in the state file.
fn recorded_start_digest(state: &RuntimeStateV1, task_id: &str) -> Option<String> {
    state
        .tasks
        .get(task_id)?
        .extra
        .get(STATE_KEY_CONTRACT_DIGEST)?
        .as_str()
        .map(str::to_string)
}

/// Builds the source revision from the git provenance primitives.
///
/// Every field is an `Option` at the source, so a project without git — or one
/// where `git` is not on PATH — becomes [`RevisionState::Unattested`] rather
/// than carrying a fabricated SHA.
fn source_revision_for(
    root: &ProjectRoot,
    contract: &TaskContract,
    start_sha: Option<&str>,
) -> (SourceRevision, Vec<String>) {
    let head = dare_project::git::head_sha(root);
    let dirty = dare_project::git::worktree_is_dirty(root);

    let state = match (&head, dirty) {
        (Some(_), Some(false)) => RevisionState::Clean,
        (Some(_), Some(true)) => RevisionState::Dirty,
        _ => RevisionState::Unattested,
    };

    let changed = start_sha
        .and_then(|from| dare_project::git::changed_files(root, from, None))
        .unwrap_or_default();

    let aliases: Vec<String> = if contract.starting_context.repositories.is_empty() {
        vec![SELF_REPO_ALIAS.to_string()]
    } else {
        contract
            .starting_context
            .repositories
            .iter()
            .map(|r| r.alias.clone())
            .collect()
    };

    let repositories = aliases
        .into_iter()
        .map(|alias| RepoRevision {
            alias,
            commit_sha: head.clone(),
            worktree_digest: None,
        })
        .collect();

    (
        SourceRevision {
            state,
            repositories,
        },
        changed,
    )
}

/// Emits the receipt and handoff sidecar for a task that reached DONE.
///
/// Returns `None` when the task has no contract: contracts are optional
/// (D-06), and their absence must leave the run identical to today, with no
/// sidecar and no new JSON field.
///
/// Refuses when the contract's material fields moved since `Start`, because
/// adopting a contract the work was not done against would make the receipt a
/// claim about the wrong thing.
fn emit_handoff_on_complete(
    root: &ProjectRoot,
    dag_rel: &SafeRelativePath,
    doc: &DagDocument,
    task_id: &str,
    deliverable: &str,
    clock: &dyn Clock,
) -> CoreResult<Option<(String, Value)>> {
    let Some(contract) = load_contract(root, dag_rel, task_id)? else {
        return Ok(None);
    };

    let state = load_state_soft(root);
    let recorded = recorded_start_digest(&state, task_id);
    let current = material_digest(&contract)?;
    if let Some(recorded) = &recorded {
        if recorded != &current {
            return Err(CoreError::config(
                MSG_CONTRACT_CHANGED_TMPL.replace("{id}", task_id),
            ));
        }
    }

    let start_sha = state
        .tasks
        .get(task_id)
        .and_then(|t| t.extra.get(STATE_KEY_START_SHA))
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let (revision, changed) = source_revision_for(root, &contract, start_sha.as_deref());

    // A re-run over an existing sidecar keeps the previous receipt's digest.
    supersede_handoff(root, task_id, "re-executed", &clock.now_rfc3339())?;

    let req = ReceiptRequest {
        task_id,
        contract: &contract,
        revision: &revision,
        created_at: &clock.now_rfc3339(),
        driver: "cli",
        changed_files: &changed,
    };
    let receipt = receipt_from_verification(root, &req)?;
    let receipt_value =
        serde_json::to_value(&receipt).map_err(|e| CoreError::internal(e.to_string()))?;

    let existing = load_handoff(root, task_id).ok();
    let record = HandoffRecord {
        schema_version: HANDOFF_SCHEMA_VERSION,
        task_id: task_id.to_string(),
        kind: contract.exit_criteria.kind,
        receipt,
        deliverable: deliverable.to_string(),
        context_bounds: u32::try_from(parent_context_limit(doc)).unwrap_or(u32::MAX),
        next_steps: Vec::new(),
        open_items: Vec::new(),
        superseded: existing.map(|r| r.superseded).unwrap_or_default(),
        extra: Map::new(),
    };
    write_handoff(root, &record)?;

    let rel = handoff_rel(task_id)?;
    Ok(Some((rel.as_str().to_string(), receipt_value)))
}

/// Wraps an emission failure so the operator learns the status is still right.
///
/// Reverting a legitimate DONE because a sidecar write failed would be worse
/// than the failure itself, so the status is never touched here.
fn receipt_failure_error(task_id: &str, err: &CoreError) -> CoreError {
    // A contract that moved is already a precise message; do not bury it.
    if err.message().contains("HANDOFF_CONTRACT_DIGEST_MISMATCH") {
        return CoreError::config(err.message().to_string());
    }
    CoreError::internal(
        MSG_RECEIPT_NOT_WRITTEN_TMPL
            .replace("{id}", task_id)
            .replace("{why}", err.message()),
    )
}

/// The `handoff` block added to each `ready` item of `--next --json`.
fn handoff_json_block(policy: &HandoffPolicy, outcome: &GateOutcome) -> Value {
    let refused: Vec<Value> = outcome
        .refused
        .iter()
        .map(|d| {
            json!({
                "parentId": d.parent_id,
                "state": evidence_state_str(d.state),
                "code": d.refusal.unwrap_or(RefusalCode::ReceiptInvalid).as_str(),
                "detail": d.detail,
            })
        })
        .collect();
    json!({
        "strict": policy.strict,
        "imported": outcome.imported,
        "refused": refused,
    })
}

/// Wire string of an evidence state.
fn evidence_state_str(state: EvidenceState) -> &'static str {
    match state {
        EvidenceState::Valid => "valid",
        EvidenceState::Missing => "missing",
        EvidenceState::Invalid => "invalid",
        EvidenceState::Stale => "stale",
        EvidenceState::NotApplicable => "notApplicable",
        EvidenceState::Insufficient => "insufficient",
    }
}

// ---------------------------------------------------------------------------
// Handoff gate (task-009)
// ---------------------------------------------------------------------------

/// Warning printed in compatibility mode, where a refusal informs but never
/// blocks. It states plainly that it is not verification, so nobody reads a
/// successful run as a verified one.
const MSG_COMPAT_WARN_TMPL: &str = concat!(
    "warning: handoff from {id} not verified ({code}); imported anyway\n",
    "         (compatibility mode — this is not verification; ",
    "enable handoff.strict to block)"
);

/// Prefix of a strict refusal, which is a hard error (exit 6).
const MSG_STRICT_REFUSED_TMPL: &str = "handoff refused: {id} ({code})";

/// [`ParentEvidence`] backed by the evidence on disk.
///
/// Delegates straight to [`evaluate_parent`] and re-implements no check of its
/// own, so the twelve ordered checks have exactly one home.
pub struct CliParentEvidence<'a> {
    root: &'a ProjectRoot,
    dag_rel: &'a SafeRelativePath,
    policy: &'a HandoffPolicy,
    calls: Cell<u32>,
}

impl<'a> CliParentEvidence<'a> {
    pub fn new(
        root: &'a ProjectRoot,
        dag_rel: &'a SafeRelativePath,
        policy: &'a HandoffPolicy,
    ) -> Self {
        Self {
            root,
            dag_rel,
            policy,
            calls: Cell::new(0),
        }
    }

    /// How many times evidence was consulted.
    ///
    /// Only the zero-cost assertions need this, so it does not exist in a
    /// release build.
    #[cfg(test)]
    pub fn calls(&self) -> u32 {
        self.calls.get()
    }
}

impl ParentEvidence for CliParentEvidence<'_> {
    fn decide(&self, parent_id: &str) -> HandoffDecision {
        self.calls.set(self.calls.get().saturating_add(1));
        evaluate_parent(self.root, self.dag_rel, parent_id, self.policy)
    }
}

/// Whether this project produces handoffs at all.
///
/// A project that has never written a sidecar gets no evaluation and no
/// warning: the default path must not pay for a feature it does not use. Once
/// any sidecar exists the project has opted in, and from then on a parent
/// *without* one is exactly what compatibility mode should report — so the
/// question is about the directory, not about any one task.
fn handoff_in_use(root: &ProjectRoot) -> bool {
    let Ok(rel) = SafeRelativePath::new(HANDOFF_DIR_REL) else {
        return false;
    };
    let Ok(abs) = root.resolve(&rel) else {
        return false;
    };
    let Ok(entries) = std::fs::read_dir(abs.as_path().as_std_path()) else {
        return false;
    };
    entries.flatten().any(|e| {
        e.path()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    })
}

/// Composes one task's prompt under the resolved policy.
///
/// Strict refuses the whole run with [`CoreError::GuardFail`] — exit 6 — on the
/// first parent whose evidence does not hold, because importing unverified
/// context is the thing strict mode exists to prevent.
///
/// Compatibility mode composes exactly as today and routes any refusal to
/// `warn`, which the caller sends to stderr. stdout is untouched, which is what
/// the golden baseline asserts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateOutcome {
    pub prompt: String,
    /// Parents whose deliverable reached the prompt.
    pub imported: Vec<String>,
    /// Decisions that kept a parent out, or that only warned in compat mode.
    pub refused: Vec<HandoffDecision>,
}

pub fn gated_prompt(
    root: &ProjectRoot,
    doc: &DagDocument,
    state: &RuntimeStateV1,
    task_id: &str,
    policy: &HandoffPolicy,
    ev: &dyn ParentEvidence,
    warn: &mut dyn FnMut(String),
) -> CoreResult<GateOutcome> {
    if policy.strict {
        let composed = compose_task_prompt_verified(doc, state, task_id, ev)?;
        if let Some(refusal) = composed.refused.first() {
            return Err(CoreError::guard_fail(
                MSG_STRICT_REFUSED_TMPL
                    .replace("{id}", &refusal.parent_id)
                    .replace("{code}", &refusal_code_str(refusal)),
            ));
        }
        return Ok(GateOutcome {
            prompt: composed.prompt,
            imported: composed.imported,
            refused: composed.refused,
        });
    }

    // Default path: today's composition, byte for byte.
    let prompt = compose_task_prompt(doc, state, task_id)?;
    let mut imported = Vec::new();
    let mut refused = Vec::new();
    if handoff_in_use(root) {
        let composed = compose_task_prompt_verified(doc, state, task_id, ev)?;
        for refusal in &composed.refused {
            warn(
                MSG_COMPAT_WARN_TMPL
                    .replace("{id}", &refusal.parent_id)
                    .replace("{code}", &refusal_code_str(refusal)),
            );
        }
        imported = composed.imported;
        refused = composed.refused;
    }
    Ok(GateOutcome {
        prompt,
        imported,
        refused,
    })
}

/// Stable wire code of a refusal, or `ReceiptInvalid` for the impossible case
/// of a refusal without one — never a silent empty string.
fn refusal_code_str(decision: &HandoffDecision) -> String {
    decision
        .refusal
        .unwrap_or(RefusalCode::ReceiptInvalid)
        .as_str()
        .to_string()
}

/// Rewrites a next-report's prompts under the policy, after the preflight.
///
/// The report arrives with prompts already composed the legacy way; strict mode
/// replaces them with verified ones. Returns the warnings for stderr.
pub fn apply_handoff_gate(
    root: &ProjectRoot,
    dag_rel: &SafeRelativePath,
    doc: &DagDocument,
    state: &RuntimeStateV1,
    report: &mut NextReport,
    policy: &HandoffPolicy,
) -> CoreResult<(Vec<String>, BTreeMap<String, Value>)> {
    if policy.strict {
        run_handoff_preflight(root, dag_rel, doc, policy)?;
    }
    let ev = CliParentEvidence::new(root, dag_rel, policy);
    // The block is additive, and "additive with a safe default" means absent
    // while the feature is dormant: a project that neither enables strict nor
    // produces sidecars must emit the JSON it emitted before this existed.
    let report_handoff = policy.strict || handoff_in_use(root);
    let mut warnings = Vec::new();
    let mut blocks = BTreeMap::new();
    for task in &mut report.ready {
        let mut sink = |w: String| warnings.push(w);
        let outcome = gated_prompt(root, doc, state, &task.id, policy, &ev, &mut sink)?;
        if report_handoff {
            blocks.insert(task.id.clone(), handoff_json_block(policy, &outcome));
        }
        task.prompt = outcome.prompt;
    }
    Ok((warnings, blocks))
}

#[cfg(test)]
mod handoff_gate_tests {
    use super::*;
    use dare_contracts::{
        parse_dag_yaml, EvidenceState, HandoffKind, HandoffRecord, Producer, RepoRevision,
        RevisionState, SourceRevision, TaskRuntimeState, Verdict, VerificationReceipt,
        HANDOFF_SCHEMA_VERSION,
    };
    use dare_verify::write_handoff;
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    const DAG: &str = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-a
    title: Alpha
    depends_on: []
    complexity: LOW
    subtask_prompt: do-a
  - id: task-b
    title: Beta
    depends_on: [task-a]
    complexity: LOW
    subtask_prompt: do-b
"#;

    /// Counts how often evidence is consulted and always refuses, so a single
    /// call is visible both in the counter and in the output.
    struct CountingEvidence {
        calls: Cell<u32>,
    }

    impl CountingEvidence {
        fn new() -> Self {
            Self {
                calls: Cell::new(0),
            }
        }
    }

    impl ParentEvidence for CountingEvidence {
        fn decide(&self, parent_id: &str) -> HandoffDecision {
            self.calls.set(self.calls.get() + 1);
            HandoffDecision::refused(
                parent_id,
                EvidenceState::Missing,
                RefusalCode::ReceiptMissing,
                "no sidecar",
            )
        }
    }

    fn temp_root() -> (tempfile::TempDir, ProjectRoot) {
        let dir = tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("project root");
        (dir, root)
    }

    fn state_with_done_parent() -> RuntimeStateV1 {
        let mut tasks = BTreeMap::new();
        tasks.insert(
            "task-a".to_string(),
            TaskRuntimeState {
                status: TaskStatus::Done.as_str().to_string(),
                output: "alpha output".to_string(),
                error: String::new(),
                tokens: None,
                duration: None,
                attempts: Vec::new(),
                parent_id: None,
                depends_on: Vec::new(),
                extra: Map::new(),
            },
        );
        tasks.insert(
            "task-b".to_string(),
            TaskRuntimeState {
                status: TaskStatus::Pending.as_str().to_string(),
                output: String::new(),
                error: String::new(),
                tokens: None,
                duration: None,
                attempts: Vec::new(),
                parent_id: None,
                depends_on: vec!["task-a".to_string()],
                extra: Map::new(),
            },
        );
        RuntimeStateV1 {
            version: 1,
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            tasks,
            extra: Map::new(),
        }
    }

    fn seed_sidecar(root: &ProjectRoot, task_id: &str) {
        let receipt = VerificationReceipt {
            schema_version: HANDOFF_SCHEMA_VERSION,
            task_id: task_id.to_string(),
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
            artifact_refs: Vec::new(),
            artifact_digests: Vec::new(),
            verification_ref: format!(".dare/verification/{task_id}.json"),
            verification_digest: "d".repeat(64),
            verdict: Verdict::Passed,
            created_at: "2026-10-13T12:00:00Z".to_string(),
            producer: Producer {
                cli_version: "4.0.0".to_string(),
                driver: "cli".to_string(),
            },
        };
        let record = HandoffRecord {
            schema_version: HANDOFF_SCHEMA_VERSION,
            task_id: task_id.to_string(),
            kind: HandoffKind::VerifiedDeliverable,
            receipt,
            deliverable: "seeded".to_string(),
            context_bounds: 2000,
            next_steps: Vec::new(),
            open_items: Vec::new(),
            superseded: Vec::new(),
            extra: Map::new(),
        };
        write_handoff(root, &record).expect("sidecar written");
    }

    #[test]
    fn preflight_not_called_when_not_strict() {
        let (_dir, root) = temp_root();
        let doc = parse_dag_yaml(DAG).expect("dag parses");
        let state = state_with_done_parent();
        let policy = HandoffPolicy::default();
        assert!(!policy.strict);

        let ev = CountingEvidence::new();
        let mut warnings = Vec::new();
        let prompt = gated_prompt(&root, &doc, &state, "task-b", &policy, &ev, &mut |w| {
            warnings.push(w)
        })
        .expect("compat composes")
        .prompt;

        assert_eq!(
            ev.calls.get(),
            0,
            "a project with no sidecar must not consult evidence at all"
        );
        assert!(warnings.is_empty(), "nothing to warn about: {warnings:?}");
        assert_eq!(
            prompt,
            compose_task_prompt(&doc, &state, "task-b").expect("legacy composes"),
            "the default path composes exactly as today"
        );
    }

    #[test]
    fn cli_evidence_not_constructed_into_calls_when_not_strict() {
        // The real evidence type, not the fake: proves `evaluate_parent` is
        // never reached on the default path.
        let (_dir, root) = temp_root();
        let doc = parse_dag_yaml(DAG).expect("dag parses");
        let state = state_with_done_parent();
        let policy = HandoffPolicy::default();
        let dag_rel = SafeRelativePath::new("DARE/dare-dag.yaml").expect("rel");
        let ev = CliParentEvidence::new(&root, &dag_rel, &policy);

        let mut warnings = Vec::new();
        gated_prompt(&root, &doc, &state, "task-b", &policy, &ev, &mut |w| {
            warnings.push(w)
        })
        .expect("compat composes");
        assert_eq!(
            ev.calls(),
            0,
            "evaluate_parent was reached on the default path"
        );
    }

    #[test]
    fn compat_warns_once_a_sidecar_exists() {
        let (_dir, root) = temp_root();
        let doc = parse_dag_yaml(DAG).expect("dag parses");
        let state = state_with_done_parent();
        let policy = HandoffPolicy::default();
        // The project has opted into handoffs by producing one.
        seed_sidecar(&root, "task-a");

        let ev = CountingEvidence::new();
        let mut warnings = Vec::new();
        let prompt = gated_prompt(&root, &doc, &state, "task-b", &policy, &ev, &mut |w| {
            warnings.push(w)
        })
        .expect("compat never blocks")
        .prompt;

        assert_eq!(ev.calls.get(), 1, "the DONE parent was evaluated");
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        let w = &warnings[0];
        assert!(w.contains("handoff from task-a not verified"), "{w}");
        assert!(w.contains("HANDOFF_RECEIPT_MISSING"), "{w}");
        assert!(w.contains("imported anyway"), "{w}");
        assert!(
            w.contains("this is not verification"),
            "the warning must never read as a verification: {w}"
        );
        assert_eq!(
            prompt,
            compose_task_prompt(&doc, &state, "task-b").expect("legacy composes"),
            "compat mode imports anyway, so the prompt is unchanged"
        );
    }

    #[test]
    fn strict_refusal_is_guard_fail_exit_6() {
        let (_dir, root) = temp_root();
        let doc = parse_dag_yaml(DAG).expect("dag parses");
        let state = state_with_done_parent();
        let policy = HandoffPolicy {
            strict: true,
            ..HandoffPolicy::default()
        };

        let ev = CountingEvidence::new();
        let mut warnings = Vec::new();
        let err = gated_prompt(&root, &doc, &state, "task-b", &policy, &ev, &mut |w| {
            warnings.push(w)
        })
        .expect_err("strict blocks an unverified parent");

        assert_eq!(err.exit_code(), 6, "got {err:?}");
        assert!(matches!(err, CoreError::GuardFail(_)), "got {err:?}");
        assert!(
            err.message().contains("handoff refused: task-a"),
            "got {}",
            err.message()
        );
        assert!(
            err.message().contains("HANDOFF_RECEIPT_MISSING"),
            "the stable code belongs in the message: {}",
            err.message()
        );
        assert!(warnings.is_empty(), "strict refuses instead of warning");
    }

    #[test]
    fn strict_imports_a_valid_parent() {
        struct AcceptAll;
        impl ParentEvidence for AcceptAll {
            fn decide(&self, parent_id: &str) -> HandoffDecision {
                HandoffDecision::valid(parent_id, "alpha output", None)
            }
        }

        let (_dir, root) = temp_root();
        let doc = parse_dag_yaml(DAG).expect("dag parses");
        let state = state_with_done_parent();
        let policy = HandoffPolicy {
            strict: true,
            ..HandoffPolicy::default()
        };

        let mut warnings = Vec::new();
        let prompt = gated_prompt(
            &root,
            &doc,
            &state,
            "task-b",
            &policy,
            &AcceptAll,
            &mut |w| warnings.push(w),
        )
        .expect("a valid parent imports")
        .prompt;

        assert_eq!(
            prompt,
            compose_task_prompt(&doc, &state, "task-b").expect("legacy composes"),
            "verified evidence carrying the recorded output is byte-identical"
        );
        assert!(warnings.is_empty());
    }
}
