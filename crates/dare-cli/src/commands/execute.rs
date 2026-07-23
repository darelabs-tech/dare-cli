//! `dare execute --status|--next|--watch` (microplano 028).

use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use dare_contracts::{load_dag, load_runtime_state, RuntimeStateV1};
use dare_core::{CoreError, CoreResult, ProjectRoot};
use dare_dag::{
    build_next_report, build_status_snapshot, compute_ranks, ensure_state, write as write_canvas,
    ExecuteOutcome, NextReport, StatusSnapshot, SystemClock, CANVAS_REL, DEFAULT_DAG_REL,
    MSG_BLOCKED, MSG_EMPTY, MSG_RESOLVED, STATE_REL,
};
use dare_project::find_project_root;
use serde_json::{json, Map, Value};

use crate::commands::path_resolve::resolve_project_rel;
use crate::output::OutputRenderer;

#[derive(Debug, Clone)]
pub enum ExecuteAction {
    Status,
    Next,
    Watch {
        interval_secs: u64,
        max_ticks: Option<u64>,
    },
}

pub fn run_execute(
    dag: Option<PathBuf>,
    action: ExecuteAction,
    renderer: &OutputRenderer<'_>,
) -> ExitCode {
    match run_execute_inner(dag, action, renderer) {
        Ok(Some((human, data))) => {
            if let Err(e) = renderer.write_success(&human, data) {
                return ExitCode::from(renderer.write_error(&e) as u8);
            }
            ExitCode::SUCCESS
        }
        Ok(None) => ExitCode::SUCCESS, // watch already streamed ticks
        Err(e) => ExitCode::from(renderer.write_error(&e) as u8),
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
    }
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
