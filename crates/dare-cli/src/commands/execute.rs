//! `dare execute` — status / next / watch (028) + complete / fail / reset (029).

use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use dare_contracts::{
    load_dag, load_dare_config, load_runtime_state, DagDocument, DareConfig, RuntimeStateV1,
};
use dare_core::{
    truncate_chars, CoreError, CoreResult, MockProcessRunner, ProcessOutput, ProcessRunner,
    ProjectRoot, SafeRelativePath, SystemProcessRunner,
};
use dare_dag::{
    build_next_report, build_status_snapshot, compute_ranks, ensure_state, iter_task_views,
    transition, write as write_canvas, Clock, ExecuteOutcome, NextReport, RefreshCanvas,
    StatusSnapshot, SystemClock, TaskStatus, Transition, CANVAS_REL, DEFAULT_DAG_REL, MSG_BLOCKED,
    MSG_EMPTY, MSG_RESOLVED, STATE_REL,
};
use dare_project::find_project_root;
use dare_verify::{
    formal_enabled_from_cfg, resolve_stack, run_advanced_verify, run_ralph, task_id_is_path_safe,
    verification_from_ralph, verify_enabled_from_cfg, write_advanced_verdict, write_verification,
    AdvancedVerifyRequest, FormalBackend, LoopVerdict, RalphReport, VERIFICATION_DIR_REL,
};
use serde_json::{json, Map, Value};

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
}

impl Default for CompleteVerifyOpts {
    fn default() -> Self {
        Self {
            verify: None,
            full_mutation: false,
            formal: None,
            formal_backend: FormalBackend::Dafny,
            verdict_json: false,
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
    },
    CleanupWorktrees,
}

pub fn run_execute(
    dag: Option<PathBuf>,
    action: ExecuteAction,
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
        } => crate::commands::execute_agent::run_agent(
            dag,
            crate::commands::execute_agent::AgentOpts {
                driver,
                task,
                budget_tokens,
                policy,
            },
            renderer,
        ),
        ExecuteAction::CleanupWorktrees => {
            crate::commands::execute_agent::run_cleanup_worktrees(renderer)
        }
        other => match run_execute_inner(dag, other, renderer) {
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
            let report = build_next_report(&doc, &state, &ranks).map_err(CoreError::from)?;
            let human = format_next_human(&report);
            let data = next_json(&report, &dag_display);
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

    ensure_running(&root, &doc, &state, task_id, &clock).map_err(CompleteOutcome::Other)?;
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
    let verdict = run_advanced_verify(&root, &cfg, &adv_req, &adv_runner)
        .map_err(CompleteOutcome::Other)?;
    if adv_req.verify {
        write_advanced_verdict(&root, &verdict).map_err(CompleteOutcome::Other)?;
    }
    if !verdict.ok {
        let msg = format_advanced_gate_fail(task_id, &verdict);
        return Err(CompleteOutcome::Other(CoreError::internal(msg)));
    }

    let raw_output = output.unwrap_or(MSG_OUTPUT_DEFAULT);
    let (truncated, _) = truncate_chars(raw_output.to_string(), cap);
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

    let human = MSG_COMPLETE_OK_TMPL.replace("{id}", task_id);
    let data = complete_json(task_id, &ralph, &verdict);
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
        doc,
        clock,
        state,
    } = prepare_mutation_ctx(dag)?;

    if !dag_has_task(&doc, task_id) {
        return Err(CoreError::not_found(format!(
            "task not found in DAG: {task_id}"
        )));
    }

    ensure_running(&root, &doc, &state, task_id, &clock)?;

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
        doc,
        clock,
        state,
    })
}

fn ensure_running(
    root: &ProjectRoot,
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

fn complete_json(task_id: &str, ralph: &RalphReport, verdict: &LoopVerdict) -> Value {
    let verification_path = format!("{VERIFICATION_DIR_REL}/{task_id}.json");
    let advanced_path = format!("{VERIFICATION_DIR_REL}/{task_id}.advanced.json");
    let ralph_val = serde_json::to_value(ralph).unwrap_or_else(|_| json!({}));
    let verdict_val = serde_json::to_value(verdict).unwrap_or_else(|_| json!({}));
    json!({
        "action": "complete",
        "taskId": task_id,
        "status": "DONE",
        "verificationPath": verification_path,
        "advancedVerificationPath": advanced_path,
        "ralph": ralph_val,
        "verdict": verdict_val,
    })
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

fn next_json(report: &NextReport, dag: &str) -> Value {
    let ready: Vec<Value> = report
        .ready
        .iter()
        .map(|t| {
            json!({
                "complexity": t.complexity,
                "id": t.id,
                "prompt": t.prompt,
                "rank": t.rank,
                "specFile": t.spec_file,
                "title": t.title,
            })
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
