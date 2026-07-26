//! `run_hooks` — trust gate, idempotency, argv-only spawn via `ProcessRunner`.

use std::time::Duration;

use dare_contracts::DareConfig;
use dare_core::{
    CoreError, CoreResult, ProcessRunner, ProjectRoot, SafeCommand, SafeRelativePath,
};

use crate::action::{action_argv, HookAction};
use crate::config::{hooks_enabled, hooks_trusted, MSG_HOOKS_DISABLED, MSG_HOOKS_TRUST};
use crate::defs::load_hooks_defs;
use crate::event::HookEvent;
use crate::idempotency::{digest_key, marker_exists, prune_if_needed, write_marker};
use crate::report::{HookActionResult, HooksRunReport, HOOKS_RUN_SCHEMA};

const HOOK_TIMEOUT_SECS: u64 = 120;

/// Request for [`run_hooks`].
pub struct RunHooksRequest<'a> {
    pub event: HookEvent,
    pub file: Option<&'a str>,
    pub task: Option<&'a str>,
    pub trust_flag: bool,
}

/// Run hooks for `req.event`: trust gate → idempotency → spawn via `runner`.
///
/// Returns `Ok(report)` when trust/defs succeed, even if some actions fail
/// (non-zero exit). CLI maps any `status == "failed"` to exit 1.
pub fn run_hooks(
    root: &ProjectRoot,
    cfg: &DareConfig,
    req: &RunHooksRequest<'_>,
    runner: &dyn ProcessRunner,
) -> CoreResult<HooksRunReport> {
    if !hooks_enabled(cfg) {
        return Err(CoreError::usage(MSG_HOOKS_DISABLED));
    }
    let trusted = hooks_trusted(cfg) || req.trust_flag;
    if !trusted {
        return Err(CoreError::usage(MSG_HOOKS_TRUST));
    }

    let file_posix: Option<String> = match req.file {
        Some(f) => {
            let rel = SafeRelativePath::new(f)?;
            Some(rel.as_str().to_string())
        }
        None => None,
    };
    let task = req.task.map(str::to_string);

    let (file_defs, _source) = load_hooks_defs(root)?;
    let actions: Vec<HookAction> = file_defs
        .hooks
        .into_iter()
        .filter(|h| h.event == req.event)
        .flat_map(|h| h.actions)
        .collect();

    let event_str = req.event.as_str();
    let mut results = Vec::with_capacity(actions.len());

    for action in actions {
        let action_str = action.as_str();
        let key = digest_key(
            event_str,
            action_str,
            file_posix.as_deref(),
            task.as_deref(),
        )?;

        if marker_exists(root, &key)? {
            results.push(HookActionResult {
                action: action_str.to_string(),
                status: "skipped".into(),
                exit_code: None,
                skipped: true,
                reason: Some("idempotent".into()),
                idempotency_key: key,
                stdout_truncated: false,
                stderr_truncated: false,
            });
            continue;
        }

        prune_if_needed(root)?;

        let program = std::env::current_exe()
            .map_err(|e| CoreError::io(e.to_string()))?
            .to_string_lossy()
            .into_owned();
        let cwd_rel = SafeRelativePath::new(".")?;
        let cmd = SafeCommand::new(program)
            .args(action_argv(action).iter().copied())
            .cwd(root.clone(), cwd_rel)
            .timeout(Duration::from_secs(HOOK_TIMEOUT_SECS));

        let out = runner.run(&cmd)?;
        if out.exit_code == 0 {
            write_marker(root, &key)?;
            results.push(HookActionResult {
                action: action_str.to_string(),
                status: "ok".into(),
                exit_code: Some(0),
                skipped: false,
                reason: None,
                idempotency_key: key,
                stdout_truncated: out.stdout_truncated,
                stderr_truncated: out.stderr_truncated,
            });
        } else {
            results.push(HookActionResult {
                action: action_str.to_string(),
                status: "failed".into(),
                exit_code: Some(out.exit_code),
                skipped: false,
                reason: None,
                idempotency_key: key,
                stdout_truncated: out.stdout_truncated,
                stderr_truncated: out.stderr_truncated,
            });
        }
    }

    Ok(HooksRunReport {
        schema_version: HOOKS_RUN_SCHEMA,
        event: event_str.to_string(),
        file: file_posix,
        task,
        trusted: true,
        results,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use dare_core::{ErrorKind, MockProcessRunner, ProcessOutput, SafeCommand};
    use tempfile::tempdir;

    use super::*;
    use crate::idempotency::{marker_exists, IDEMPOTENCY_DIR_REL};

    struct RecordingRunner {
        calls: Mutex<usize>,
        inner: MockProcessRunner,
    }

    impl RecordingRunner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(0),
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

        fn push_fail(&self, code: i32) {
            self.inner.push(ProcessOutput {
                exit_code: code,
                stdout: String::new(),
                stderr: "fail".into(),
                stdout_truncated: false,
                stderr_truncated: false,
                timed_out: false,
                cancelled: false,
            });
        }

        fn call_count(&self) -> usize {
            *self.calls.lock().unwrap()
        }
    }

    impl ProcessRunner for RecordingRunner {
        fn run(&self, cmd: &SafeCommand) -> CoreResult<ProcessOutput> {
            *self.calls.lock().unwrap() += 1;
            self.inner.run(cmd)
        }
    }

    fn ok_cfg() -> DareConfig {
        DareConfig::default()
    }

    #[test]
    fn untrusted_err() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let runner = RecordingRunner::new();
        runner.push_ok();
        let req = RunHooksRequest {
            event: HookEvent::OnSave,
            file: None,
            task: None,
            trust_flag: false,
        };
        let err = run_hooks(&root, &ok_cfg(), &req, &runner).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Usage);
        assert!(err.message().contains("HOOKS_TRUST"));
        assert_eq!(runner.call_count(), 0);
    }

    #[test]
    fn trusted_spawns() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let runner = RecordingRunner::new();
        runner.push_ok();
        let req = RunHooksRequest {
            event: HookEvent::OnSave,
            file: Some("src/lib.rs"),
            task: None,
            trust_flag: true,
        };
        let report = run_hooks(&root, &ok_cfg(), &req, &runner).expect("run");
        assert_eq!(runner.call_count(), 1);
        assert!(report.trusted);
        assert_eq!(report.event, "on-save");
        assert_eq!(report.file.as_deref(), Some("src/lib.rs"));
        assert_eq!(report.results.len(), 1);
        assert_eq!(report.results[0].action, "dare-validate");
        assert_eq!(report.results[0].status, "ok");
        assert_eq!(report.results[0].exit_code, Some(0));
        assert!(!report.results[0].skipped);
        assert!(marker_exists(&root, &report.results[0].idempotency_key).unwrap());
    }

    #[test]
    fn idempotent_skips_second() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let runner = RecordingRunner::new();
        runner.push_ok();
        let req = RunHooksRequest {
            event: HookEvent::OnSave,
            file: Some("a.rs"),
            task: None,
            trust_flag: true,
        };
        let first = run_hooks(&root, &ok_cfg(), &req, &runner).expect("first");
        assert_eq!(first.results[0].status, "ok");
        assert_eq!(runner.call_count(), 1);

        let second = run_hooks(&root, &ok_cfg(), &req, &runner).expect("second");
        assert_eq!(runner.call_count(), 1, "second run must not spawn");
        assert_eq!(second.results.len(), 1);
        assert_eq!(second.results[0].status, "skipped");
        assert!(second.results[0].skipped);
        assert_eq!(second.results[0].reason.as_deref(), Some("idempotent"));
        assert_eq!(second.results[0].exit_code, None);
        assert_eq!(
            second.results[0].idempotency_key,
            first.results[0].idempotency_key
        );
    }

    #[test]
    fn failed_no_marker() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let runner = RecordingRunner::new();
        runner.push_fail(7);
        runner.push_fail(7);
        let req = RunHooksRequest {
            event: HookEvent::OnSave,
            file: Some("b.rs"),
            task: None,
            trust_flag: true,
        };
        let first = run_hooks(&root, &ok_cfg(), &req, &runner).expect("first");
        assert_eq!(first.results[0].status, "failed");
        assert_eq!(first.results[0].exit_code, Some(7));
        assert!(!marker_exists(&root, &first.results[0].idempotency_key).unwrap());
        let idemp = dir.path().join(IDEMPOTENCY_DIR_REL);
        if idemp.is_dir() {
            let markers: Vec<_> = std::fs::read_dir(&idemp)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|x| x == "ok"))
                .collect();
            assert!(markers.is_empty(), "failed run must not create marker");
        }
        assert_eq!(runner.call_count(), 1);

        let second = run_hooks(&root, &ok_cfg(), &req, &runner).expect("second");
        assert_eq!(runner.call_count(), 2, "second run must still spawn");
        assert_eq!(second.results[0].status, "failed");
    }
}
