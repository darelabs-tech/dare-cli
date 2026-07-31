//! Golden suite runner: spawn dare argv-only, compare axes with normalize.

use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use dare_core::{CoreError, CoreResult};

use crate::axis::CompareAxis;
use crate::case::{load_case, CaseSpec, DiffClass, MSG_SKIP_NEEDS_CLASS};
use crate::diff_log::{DiffLogIndex, MSG_UNCLASSIFIED_DIFF};
use crate::normalize::{normalize_text, NormalizeCtx};
use crate::report::{CaseResult, CaseStatus, DiffReport};

/// Default per-case process timeout.
pub const DEFAULT_CASE_TIMEOUT: Duration = Duration::from_secs(30);

/// Options for [`run_suite`].
#[derive(Debug, Clone)]
pub struct SuiteOpts {
    /// Per-case timeout (default 30s).
    pub timeout: Duration,
    /// Optional override for the `dare` binary path.
    pub bin: Option<PathBuf>,
    /// Optional fixtures root (defaults to `<repo>/tests/fixtures` beside golden).
    pub fixtures_root: Option<PathBuf>,
    /// Optional path to `parity-diff-log.md`.
    pub diff_log_path: Option<PathBuf>,
    /// When set, write `DiffReport` JSON here after the suite.
    pub report_out: Option<PathBuf>,
}

impl Default for SuiteOpts {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_CASE_TIMEOUT,
            bin: None,
            fixtures_root: None,
            diff_log_path: None,
            report_out: None,
        }
    }
}

/// Captured process output for axis comparison.
#[derive(Debug, Clone)]
struct ProcOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

/// Run a single golden case against `bin`.
pub fn run_case(
    spec: &CaseSpec,
    bin: &Path,
    fixtures_root: &Path,
    diff_log: &DiffLogIndex,
) -> CoreResult<CaseResult> {
    run_case_with_timeout(spec, bin, fixtures_root, diff_log, DEFAULT_CASE_TIMEOUT, None)
}

fn run_case_with_timeout(
    spec: &CaseSpec,
    bin: &Path,
    fixtures_root: &Path,
    diff_log: &DiffLogIndex,
    timeout: Duration,
    case_dir: Option<&Path>,
) -> CoreResult<CaseResult> {
    if let Some(skip) = &spec.skip {
        // Class presence is enforced at parse; re-check for safety.
        let _ = skip.class;
        if skip.reason.trim().is_empty() {
            return Ok(CaseResult {
                id: spec.id.clone(),
                status: CaseStatus::Fail,
                failed_axes: vec![],
                class: Some(skip.class),
                message: Some(MSG_SKIP_NEEDS_CLASS.to_string()),
            });
        }
        if skip.class == DiffClass::C {
            let adr_ok = skip
                .adr_ref
                .as_ref()
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
            let log_ok = diff_log.contains_surface_or_id(&spec.id)
                || diff_log.contains_surface_or_id(skip.reason.trim())
                || skip
                    .adr_ref
                    .as_ref()
                    .map(|a| diff_log.contains_surface_or_id(a))
                    .unwrap_or(false);
            if !adr_ok && !log_ok {
                return Err(CoreError::invalid_input(MSG_UNCLASSIFIED_DIFF));
            }
        }
        return Ok(CaseResult {
            id: spec.id.clone(),
            status: CaseStatus::Skip,
            failed_axes: vec![],
            class: Some(skip.class),
            message: Some(skip.reason.clone()),
        });
    }

    if !bin.is_file() {
        return Err(CoreError::not_found(format!(
            "dare binary missing at {}",
            bin.display()
        )));
    }

    let (cwd, _tmp_guard): (PathBuf, Option<tempfile::TempDir>) =
        if let Some(fx) = &spec.cwd_fixture {
            let path = fixtures_root.join(fx);
            if !path.is_dir() {
                return Err(CoreError::not_found(format!(
                    "cwd_fixture missing: {}",
                    path.display()
                )));
            }
            (path, None)
        } else {
            let tmp = tempfile::tempdir()
                .map_err(|e| CoreError::io(format!("temp cwd: {e}")))?;
            let path = tmp.path().to_path_buf();
            (path, Some(tmp))
        };

    let output = spawn_dare(bin, &spec.command, &spec.env, &cwd, timeout)?;

    let norm_ctx = NormalizeCtx {
        temp_prefixes: vec![cwd.clone()],
        binary_version: None,
    };

    let mut failed: Vec<CompareAxis> = Vec::new();
    let mut messages: Vec<String> = Vec::new();

    for axis in &spec.axes {
        match axis {
            CompareAxis::Exit => {
                let expected = spec.expected_exit.ok_or_else(|| {
                    CoreError::invalid_input(format!(
                        "case {}: exit axis requires expected.exit",
                        spec.id
                    ))
                })?;
                if output.exit_code != expected {
                    failed.push(CompareAxis::Exit);
                    messages.push(format!(
                        "exit: expected {expected}, got {}",
                        output.exit_code
                    ));
                }
            }
            CompareAxis::Stdout => {
                let expected_path = resolve_expected(
                    case_dir,
                    spec.expected_stdout_path.as_deref(),
                    "stdout",
                    &spec.id,
                )?;
                let expected_raw = read_utf8_lossy(&expected_path)?;
                let expected = normalize_text(&scrub_cli_name(&expected_raw), &norm_ctx);
                let actual = normalize_text(&scrub_cli_name(&output.stdout), &norm_ctx);
                if actual != expected {
                    failed.push(CompareAxis::Stdout);
                    messages.push(format!("stdout mismatch for {}", spec.id));
                }
            }
            CompareAxis::Stderr => {
                let expected_path = resolve_expected(
                    case_dir,
                    spec.expected_stderr_path.as_deref(),
                    "stderr",
                    &spec.id,
                )?;
                let expected_raw = read_utf8_lossy(&expected_path)?;
                let expected = normalize_text(&scrub_cli_name(&expected_raw), &norm_ctx);
                let actual = normalize_text(&scrub_cli_name(&output.stderr), &norm_ctx);
                if actual != expected {
                    failed.push(CompareAxis::Stderr);
                    messages.push(format!("stderr mismatch for {}", spec.id));
                }
            }
            CompareAxis::Content => {
                if spec.expected_content.is_empty() {
                    return Err(CoreError::invalid_input(format!(
                        "case {}: content axis requires expected.content",
                        spec.id
                    )));
                }
                for item in &spec.expected_content {
                    let actual_path = cwd.join(&item.rel);
                    let expected_path = match case_dir {
                        Some(dir) => dir.join(&item.file),
                        None => item.file.clone(),
                    };
                    if !expected_path.is_file() {
                        return Err(CoreError::not_found(format!(
                            "missing expected content {}",
                            expected_path.display()
                        )));
                    }
                    if !actual_path.is_file() {
                        failed.push(CompareAxis::Content);
                        messages.push(format!("missing content file {}", item.rel));
                        continue;
                    }
                    let expected =
                        normalize_text(&read_utf8_lossy(&expected_path)?, &norm_ctx);
                    let actual = normalize_text(&read_utf8_lossy(&actual_path)?, &norm_ctx);
                    if actual != expected {
                        failed.push(CompareAxis::Content);
                        messages.push(format!("content mismatch: {}", item.rel));
                    }
                }
            }
            CompareAxis::Tree => {
                let expected_path = resolve_expected(
                    case_dir,
                    spec.expected_tree_path.as_deref(),
                    "tree",
                    &spec.id,
                )?;
                let expected = normalize_text(&read_utf8_lossy(&expected_path)?, &norm_ctx);
                let actual_tree = list_rel_tree(&cwd)?;
                let actual = normalize_text(&actual_tree, &norm_ctx);
                if actual != expected {
                    failed.push(CompareAxis::Tree);
                    messages.push(format!("tree mismatch for {}", spec.id));
                }
            }
            CompareAxis::State => {
                let expected_path = resolve_expected(
                    case_dir,
                    spec.expected_state_path.as_deref(),
                    "state",
                    &spec.id,
                )?;
                let state_candidates = [
                    cwd.join("dare.config.json"),
                    cwd.join(".dare").join("state.json"),
                ];
                let actual_path = state_candidates
                    .iter()
                    .find(|p| p.is_file())
                    .ok_or_else(|| {
                        CoreError::not_found(format!(
                            "case {}: state axis: no dare.config.json / .dare/state.json",
                            spec.id
                        ))
                    })?;
                let expected = normalize_text(&read_utf8_lossy(&expected_path)?, &norm_ctx);
                let actual = normalize_text(&read_utf8_lossy(actual_path)?, &norm_ctx);
                if actual != expected {
                    failed.push(CompareAxis::State);
                    messages.push(format!("state mismatch for {}", spec.id));
                }
            }
            CompareAxis::Http => {
                // HTTP axis requires a live server helper — not exercised by mp054-003 help cases.
                failed.push(CompareAxis::Http);
                messages.push(format!(
                    "http axis not implemented in run_case v1 for {}",
                    spec.id
                ));
            }
        }
    }

    // Dedup failed axes (content may push multiple times).
    let mut seen = BTreeSet::new();
    failed.retain(|a| seen.insert(*a));

    if failed.is_empty() {
        Ok(CaseResult {
            id: spec.id.clone(),
            status: CaseStatus::Pass,
            failed_axes: vec![],
            class: None,
            message: None,
        })
    } else {
        Ok(CaseResult {
            id: spec.id.clone(),
            status: CaseStatus::Fail,
            failed_axes: failed,
            class: None,
            message: Some(messages.join("; ")),
        })
    }
}

/// Discover and run all `cases/*/case.yaml` under `golden_root`.
pub fn run_suite(golden_root: &Path, opts: SuiteOpts) -> CoreResult<DiffReport> {
    let cases_dir = golden_root.join("cases");
    if !cases_dir.is_dir() {
        return Err(CoreError::not_found(format!(
            "golden cases dir missing: {}",
            cases_dir.display()
        )));
    }

    let fixtures_root = opts.fixtures_root.clone().unwrap_or_else(|| {
        golden_root
            .parent()
            .map(|p| p.join("fixtures"))
            .unwrap_or_else(|| PathBuf::from("tests/fixtures"))
    });

    let diff_log_path = opts.diff_log_path.clone().unwrap_or_else(|| {
        golden_root
            .parent()
            .and_then(|p| p.parent())
            .map(|repo| repo.join("docs/compatibility/parity-diff-log.md"))
            .unwrap_or_else(|| PathBuf::from("docs/compatibility/parity-diff-log.md"))
    });
    let diff_log = if diff_log_path.is_file() {
        DiffLogIndex::load(&diff_log_path)?
    } else {
        DiffLogIndex::empty()
    };

    let bin = match &opts.bin {
        Some(p) => p.clone(),
        None => resolve_default_bin()?,
    };

    let mut case_dirs: Vec<PathBuf> = std::fs::read_dir(&cases_dir)
        .map_err(|e| CoreError::io(format!("read {}: {e}", cases_dir.display())))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("case.yaml").is_file())
        .collect();
    case_dirs.sort();

    let mut results = Vec::with_capacity(case_dirs.len());
    for dir in &case_dirs {
        let spec = load_case(dir)?;
        let result = run_case_with_timeout(
            &spec,
            &bin,
            &fixtures_root,
            &diff_log,
            opts.timeout,
            Some(dir.as_path()),
        )?;
        results.push(result);
    }

    let report = DiffReport::from_cases("1970-01-01T00:00:00Z", results);

    if let Some(out) = &opts.report_out {
        if let Some(parent) = out.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json = report
            .to_json_pretty()
            .map_err(|e| CoreError::internal(format!("serialize DiffReport: {e}")))?;
        std::fs::write(out, json)
            .map_err(|e| CoreError::io(format!("write {}: {e}", out.display())))?;
    }

    Ok(report)
}

fn resolve_default_bin() -> CoreResult<PathBuf> {
    if let Ok(p) = std::env::var("DARE_BIN") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(CoreError::not_found(
        "dare binary not configured; set SuiteOpts.bin or DARE_BIN",
    ))
}

fn resolve_expected(
    case_dir: Option<&Path>,
    rel: Option<&Path>,
    axis: &str,
    id: &str,
) -> CoreResult<PathBuf> {
    let rel = rel.ok_or_else(|| {
        CoreError::invalid_input(format!("case {id}: {axis} axis requires expected file"))
    })?;
    let path = match case_dir {
        Some(dir) => dir.join(rel),
        None => rel.to_path_buf(),
    };
    if !path.is_file() {
        return Err(CoreError::not_found(format!(
            "missing expected {axis} file {}",
            path.display()
        )));
    }
    Ok(path)
}

fn read_utf8_lossy(path: &Path) -> CoreResult<String> {
    let bytes = std::fs::read(path)
        .map_err(|e| CoreError::io(format!("read {}: {e}", path.display())))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Cross-platform scrub: clap prints `dare.exe` on Windows.
fn scrub_cli_name(s: &str) -> String {
    s.replace("dare.exe", "dare")
}

fn spawn_dare(
    bin: &Path,
    args: &[String],
    env: &std::collections::BTreeMap<String, String>,
    cwd: &Path,
    timeout: Duration,
) -> CoreResult<ProcOutput> {
    let mut cmd = Command::new(bin);
    cmd.args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("NO_COLOR", "1")
        .env("TERM", "dumb");
    for (k, v) in env {
        cmd.env(k, v);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| CoreError::io(format!("spawn {}: {e}", bin.display())))?;

    let mut stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| CoreError::internal("stdout pipe missing"))?;
    let mut stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| CoreError::internal("stderr pipe missing"))?;

    let stdout_handle = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buf);
        buf
    });
    let stderr_handle = thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buf);
        buf
    });

    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(CoreError::internal(format!(
                        "case timed out after {}s",
                        timeout.as_secs()
                    )));
                }
                thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                return Err(CoreError::io(format!("wait child: {e}")));
            }
        }
    };

    let stdout_bytes = stdout_handle
        .join()
        .map_err(|_| CoreError::internal("stdout reader panicked"))?;
    let stderr_bytes = stderr_handle
        .join()
        .map_err(|_| CoreError::internal("stderr reader panicked"))?;

    // Signal termination on Unix → treat as failure exit.
    let exit_code = status.code().unwrap_or(1);

    Ok(ProcOutput {
        exit_code,
        stdout: String::from_utf8_lossy(&stdout_bytes).into_owned(),
        stderr: String::from_utf8_lossy(&stderr_bytes).into_owned(),
    })
}

fn list_rel_tree(root: &Path) -> CoreResult<String> {
    let mut entries: Vec<String> = Vec::new();
    fn walk(base: &Path, dir: &Path, out: &mut Vec<String>) -> CoreResult<()> {
        let rd = std::fs::read_dir(dir)
            .map_err(|e| CoreError::io(format!("read_dir {}: {e}", dir.display())))?;
        for ent in rd {
            let ent = ent.map_err(|e| CoreError::io(format!("dir entry: {e}")))?;
            let path = ent.path();
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if path.is_dir() {
                out.push(format!("{rel}/"));
                walk(base, &path, out)?;
            } else {
                out.push(rel);
            }
        }
        Ok(())
    }
    walk(root, root, &mut entries)?;
    entries.sort();
    Ok(entries.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::case::{SkipSpec, CASE_SCHEMA_VERSION};
    use std::collections::BTreeMap;

    #[test]
    fn skip_class_c_without_adr_or_log_is_unclassified() {
        let spec = CaseSpec {
            schema_version: CASE_SCHEMA_VERSION,
            id: "golden.example.skip".into(),
            command: vec!["--help".into()],
            cwd_fixture: None,
            env: BTreeMap::new(),
            axes: vec![CompareAxis::Exit],
            expected_exit: Some(0),
            expected_stdout_path: None,
            expected_stderr_path: None,
            expected_tree_path: None,
            expected_content: vec![],
            expected_state_path: None,
            expected_http: None,
            skip: Some(SkipSpec {
                reason: "not ready".into(),
                class: DiffClass::C,
                adr_ref: None,
            }),
        };
        let err = run_case(
            &spec,
            Path::new("nonexistent-bin"),
            Path::new("."),
            &DiffLogIndex::empty(),
        )
        .expect_err("must unclassified");
        assert!(
            matches!(err, CoreError::InvalidInput(ref m) if m == MSG_UNCLASSIFIED_DIFF),
            "{err:?}"
        );
    }

    #[test]
    fn skip_class_c_with_adr_ok() {
        let spec = CaseSpec {
            schema_version: CASE_SCHEMA_VERSION,
            id: "golden.example.skip".into(),
            command: vec!["--help".into()],
            cwd_fixture: None,
            env: BTreeMap::new(),
            axes: vec![CompareAxis::Exit],
            expected_exit: Some(0),
            expected_stdout_path: None,
            expected_stderr_path: None,
            expected_tree_path: None,
            expected_content: vec![],
            expected_state_path: None,
            expected_http: None,
            skip: Some(SkipSpec {
                reason: "intentional".into(),
                class: DiffClass::C,
                adr_ref: Some("ADR-001".into()),
            }),
        };
        let res = run_case(
            &spec,
            Path::new("nonexistent-bin"),
            Path::new("."),
            &DiffLogIndex::empty(),
        )
        .expect("adr classifies");
        assert_eq!(res.status, CaseStatus::Skip);
    }
}
