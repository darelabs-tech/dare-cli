//! Advanced verify orchestrator (`run_advanced_verify`) — Blueprint-049 §5.3.
//!
//! Persistence choice: advanced aspects are written to a **sidecar** file
//! `.dare/verification/<taskId>.advanced.json` (full [`LoopVerdict`]), leaving the
//! Ralph report at `.dare/verification/<taskId>.json` unchanged (GateAspect Build|Test|Lint).

use std::fs;
use std::time::Duration;

use dare_contracts::DareConfig;
use dare_core::fs::atomic_write;
use dare_core::{
    redact, to_canonical_json_string, CoreError, CoreResult, ProcessRunner, ProjectRoot,
    SafeCommand, SafeRelativePath,
};

use crate::aspects::{
    check_anti_tamper, check_fail_to_pass, run_formal, run_mutation_with, FormalBackend,
};
use crate::ralph::GateAspect;
use crate::report::{AspectStatus, LoopVerdict, LOOP_VERDICT_SCHEMA};
use crate::stacks::{gate_commands, resolve_stack};
use crate::verification::{task_id_is_path_safe, VERIFICATION_DIR_REL};

/// Per-aspect spawn budget for ftp test re-run / git diff (seconds).
const ADVANCED_SPAWN_TIMEOUT_SECS: u64 = 600;

/// Request for [`run_advanced_verify`] (CLI / agent already merged flags + config).
#[derive(Debug, Clone)]
pub struct AdvancedVerifyRequest {
    pub task_id: String,
    pub full_mutation: bool,
    pub formal: bool,
    pub formal_backend: FormalBackend,
    pub verify: bool,
}

/// Orchestrate fail-to-pass → anti-tamper → mutation → formal (opt-in).
///
/// - `!req.verify` → `ok: true`, empty `aspects` (skip advanced path).
/// - Any aspect `fail` → `LoopVerdict.ok = false`.
/// - `skipped` does not block DONE.
///
/// Does **not** mutate Ralph [`GateAspect`] / verification report schema.
pub fn run_advanced_verify(
    root: &ProjectRoot,
    cfg: &DareConfig,
    req: &AdvancedVerifyRequest,
    runner: &dyn ProcessRunner,
) -> CoreResult<LoopVerdict> {
    let _ = cfg; // reserved for future verify.* knobs; flags already merged into `req`
    if !req.verify {
        return Ok(LoopVerdict {
            schema_version: LOOP_VERDICT_SCHEMA,
            task_id: req.task_id.clone(),
            ok: true,
            ralph_ok: true,
            policy: "fixed".into(),
            decay_action: "done".into(),
            best_of: None,
            winner_id: None,
            aspects: vec![],
            failure_signature: None,
        });
    }

    if !task_id_is_path_safe(&req.task_id) {
        return Err(CoreError::invalid_input(format!(
            "unsafe verification task id: {}",
            req.task_id
        )));
    }

    let stack = resolve_stack(root)?;
    let mut aspects = Vec::with_capacity(4);

    // 1) fail-to-pass
    aspects.push(run_fail_to_pass_aspect(root, &req.task_id, &stack, runner)?);

    // 2) anti-tamper
    aspects.push(run_anti_tamper_aspect(root, runner));

    // 3) mutation
    aspects.push(run_mutation_with(
        &stack,
        req.full_mutation,
        None,
        runner,
    ));

    // 4) formal (opt-in)
    if req.formal {
        aspects.push(run_formal(root, req.formal_backend, runner)?);
    }

    let any_fail = aspects.iter().any(|a| a.status == AspectStatus::Fail);
    Ok(LoopVerdict {
        schema_version: LOOP_VERDICT_SCHEMA,
        task_id: req.task_id.clone(),
        ok: !any_fail,
        ralph_ok: true,
        policy: "fixed".into(),
        decay_action: if any_fail {
            "blocked".into()
        } else {
            "done".into()
        },
        best_of: None,
        winner_id: None,
        aspects,
        failure_signature: None,
    })
}

/// Sidecar path: `.dare/verification/<taskId>.advanced.json`.
pub fn advanced_verdict_rel(task_id: &str) -> CoreResult<SafeRelativePath> {
    if !task_id_is_path_safe(task_id) {
        return Err(CoreError::invalid_input(format!(
            "unsafe verification task id: {task_id}"
        )));
    }
    let rel_str = format!("{VERIFICATION_DIR_REL}/{task_id}.advanced.json");
    SafeRelativePath::new(&rel_str)
}

/// Persist [`LoopVerdict`] to the advanced sidecar (redacted tails).
pub fn write_advanced_verdict(root: &ProjectRoot, verdict: &LoopVerdict) -> CoreResult<()> {
    let rel = advanced_verdict_rel(&verdict.task_id)?;
    let mut redacted = verdict.clone();
    for step in &mut redacted.aspects {
        step.stdout_tail = redact(&step.stdout_tail);
        step.stderr_tail = redact(&step.stderr_tail);
    }
    let value = serde_json::to_value(&redacted).map_err(|e| CoreError::internal(e.to_string()))?;
    let body = to_canonical_json_string(&value)?;
    atomic_write(root, &rel, body.as_bytes())
}

/// Read `verify.enabled` from `dare.config.json` extras (default `true`).
pub fn verify_enabled_from_cfg(cfg: &DareConfig) -> bool {
    cfg.extra
        .get("verify")
        .and_then(|v| v.get("enabled"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}

/// Read `verify.formal.enabled` from config extras (default `false`).
pub fn formal_enabled_from_cfg(cfg: &DareConfig) -> bool {
    cfg.extra
        .get("verify")
        .and_then(|v| v.get("formal"))
        .and_then(|v| v.get("enabled"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn run_fail_to_pass_aspect(
    root: &ProjectRoot,
    task_id: &str,
    stack: &str,
    runner: &dyn ProcessRunner,
) -> CoreResult<crate::report::AspectResult> {
    let ids = load_ftp_ids(root, task_id)?;
    let Some(ids) = ids else {
        return Ok(check_fail_to_pass(None, ""));
    };

    let combined = run_stack_test_output(root, stack, runner)?;
    Ok(check_fail_to_pass(Some(&ids), &combined))
}

fn load_ftp_ids(root: &ProjectRoot, task_id: &str) -> CoreResult<Option<Vec<String>>> {
    let rel_str = format!("{VERIFICATION_DIR_REL}/{task_id}.fail_to_pass.txt");
    let rel = SafeRelativePath::new(&rel_str)?;
    let abs = root.resolve(&rel)?;
    let path = abs.as_path().as_std_path();
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|e| CoreError::io(e.to_string()))?;
    let ids: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect();
    if ids.is_empty() {
        Ok(None)
    } else {
        Ok(Some(ids))
    }
}

fn run_stack_test_output(
    root: &ProjectRoot,
    stack: &str,
    runner: &dyn ProcessRunner,
) -> CoreResult<String> {
    let gates = gate_commands(stack)?;
    let Some((_, template)) = gates.into_iter().find(|(a, _)| *a == GateAspect::Test) else {
        return Ok(String::new());
    };
    let root_rel = SafeRelativePath::new(".")?;
    let cmd = SafeCommand::new(template.program())
        .args(template.arg_list().iter().cloned())
        .cwd(root.clone(), root_rel)
        .timeout(Duration::from_secs(ADVANCED_SPAWN_TIMEOUT_SECS));
    match runner.run(&cmd) {
        Ok(out) => {
            let mut combined = out.stdout;
            if !combined.is_empty() && !out.stderr.is_empty() {
                combined.push('\n');
            }
            combined.push_str(&out.stderr);
            Ok(combined)
        }
        Err(e) => Err(e),
    }
}

fn run_anti_tamper_aspect(root: &ProjectRoot, runner: &dyn ProcessRunner) -> crate::report::AspectResult {
    let diff = collect_git_diff(root, runner);
    check_anti_tamper(&diff)
}

fn collect_git_diff(root: &ProjectRoot, runner: &dyn ProcessRunner) -> String {
    let Ok(root_rel) = SafeRelativePath::new(".") else {
        return String::new();
    };
    let cmd = SafeCommand::new("git")
        .args(["diff", "HEAD"])
        .cwd(root.clone(), root_rel)
        .timeout(Duration::from_secs(60));
    match runner.run(&cmd) {
        Ok(out) => out.stdout,
        Err(_) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::{MockProcessRunner, ProcessOutput};
    use tempfile::tempdir;

    fn ok_out(stdout: &str) -> ProcessOutput {
        ProcessOutput {
            exit_code: 0,
            stdout: stdout.into(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        }
    }

    fn write_minimal_rust_project(dir: &std::path::Path) {
        fs::write(
            dir.join("dare.config.json"),
            r#"{"backend":"rust-axum"}"#,
        )
        .unwrap();
        fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"t\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
    }

    #[test]
    fn skip_verify_returns_ok_empty_aspects() {
        let dir = tempdir().unwrap();
        write_minimal_rust_project(dir.path());
        let root = ProjectRoot::new(dir.path()).unwrap();
        let cfg = DareConfig::default();
        let req = AdvancedVerifyRequest {
            task_id: "t1".into(),
            full_mutation: false,
            formal: false,
            formal_backend: FormalBackend::Dafny,
            verify: false,
        };
        let mock = MockProcessRunner::new();
        let v = run_advanced_verify(&root, &cfg, &req, &mock).unwrap();
        assert!(v.ok);
        assert!(v.aspects.is_empty());
    }

    #[test]
    fn all_pass_with_fake_runner() {
        let dir = tempdir().unwrap();
        write_minimal_rust_project(dir.path());
        let root = ProjectRoot::new(dir.path()).unwrap();
        let cfg = DareConfig::default();
        let req = AdvancedVerifyRequest {
            task_id: "mp049-005".into(),
            full_mutation: false,
            formal: false,
            formal_backend: FormalBackend::Dafny,
            verify: true,
        };
        let mock = MockProcessRunner::new();
        // anti-tamper: empty diff; mutation: tool missing → skipped
        mock.when_program("git", ok_out(""));
        mock.push_err(CoreError::not_found("cargo-mutants"));

        let v = run_advanced_verify(&root, &cfg, &req, &mock).unwrap();
        assert!(v.ok, "verdict={v:?}");
        assert_eq!(v.aspects.len(), 3); // ftp + anti-tamper + mutation (no formal)
        assert!(v.aspects.iter().all(|a| a.status != AspectStatus::Fail));
        write_advanced_verdict(&root, &v).unwrap();
        assert!(dir
            .path()
            .join(".dare/verification/mp049-005.advanced.json")
            .is_file());
    }

    #[test]
    fn mutation_fail_blocks_verdict() {
        let dir = tempdir().unwrap();
        write_minimal_rust_project(dir.path());
        let root = ProjectRoot::new(dir.path()).unwrap();
        let cfg = DareConfig::default();
        let req = AdvancedVerifyRequest {
            task_id: "t-fail".into(),
            full_mutation: true,
            formal: false,
            formal_backend: FormalBackend::Dafny,
            verify: true,
        };
        let mock = MockProcessRunner::new();
        mock.when_program("git", ok_out(""));
        // full_mutation + tool missing → fail
        mock.push_err(CoreError::not_found("cargo-mutants"));

        let v = run_advanced_verify(&root, &cfg, &req, &mock).unwrap();
        assert!(!v.ok);
        assert!(v
            .aspects
            .iter()
            .any(|a| a.aspect == crate::report::AdvancedAspect::Mutation
                && a.status == AspectStatus::Fail));
    }

    #[test]
    fn formal_opt_in_runs_aspect() {
        let dir = tempdir().unwrap();
        write_minimal_rust_project(dir.path());
        let root = ProjectRoot::new(dir.path()).unwrap();
        let cfg = DareConfig::default();
        let req = AdvancedVerifyRequest {
            task_id: "t-formal".into(),
            full_mutation: false,
            formal: true,
            formal_backend: FormalBackend::Dafny,
            verify: true,
        };
        let mock = MockProcessRunner::new();
        mock.when_program("git", ok_out(""));
        mock.push_err(CoreError::not_found("cargo-mutants"));
        // no @dare-formal targets → skipped formal
        let v = run_advanced_verify(&root, &cfg, &req, &mock).unwrap();
        assert!(v.ok);
        assert_eq!(v.aspects.len(), 4);
        assert_eq!(
            v.aspects[3].aspect,
            crate::report::AdvancedAspect::Formal
        );
        assert_eq!(v.aspects[3].status, AspectStatus::Skipped);
    }

    #[test]
    fn config_helpers_defaults() {
        let cfg = DareConfig::default();
        assert!(verify_enabled_from_cfg(&cfg));
        assert!(!formal_enabled_from_cfg(&cfg));
    }
}
