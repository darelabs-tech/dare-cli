//! Formal verification aspect (opt-in): Dafny / Verus / Lean.

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Instant;

use dare_core::{
    truncate_chars, CoreError, CoreResult, ProcessRunner, ProjectRoot, SafeCommand,
    SafeRelativePath,
};
use serde::{Deserialize, Serialize};

use crate::report::{AdvancedAspect, AspectResult, AspectStatus};

/// Message when the formal backend binary is not on PATH.
pub const MSG_FORMAL_MISSING: &str =
    "formal backend not found on PATH [FORMAL_TOOL_MISSING]";

/// Max filesystem entries visited while discovering `@dare-formal` targets.
pub const FORMAL_WALK_CAP: usize = 2000;

/// Per-backend spawn timeout (seconds), aligned with advanced verify budget.
pub const FORMAL_TIMEOUT_SECS: u64 = 600;

const MARKER_TAG: &str = "@dare-formal";
const BYPASS_MARKERS: &[&str] = &["FAKE_PROOF", "BYPASS_FORMAL"];
const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git"];

/// Formal backends accepted by `--formal-backend` / config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormalBackend {
    #[default]
    Dafny,
    Verus,
    Lean,
}

impl FormalBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dafny => "dafny",
            Self::Verus => "verus",
            Self::Lean => "lean",
        }
    }

    /// Program name on PATH (same as [`Self::as_str`]).
    pub fn program(self) -> &'static str {
        self.as_str()
    }

    /// Minimal argv for one target file (relative POSIX path under project root).
    ///
    /// - `dafny` → `verify <file>`
    /// - `verus` → `<file>`
    /// - `lean` → `<file>`
    pub fn args_for(self, target_rel: &str) -> Vec<String> {
        match self {
            Self::Dafny => vec!["verify".into(), target_rel.into()],
            Self::Verus | Self::Lean => vec![target_rel.into()],
        }
    }
}

impl FromStr for FormalBackend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "dafny" => Ok(Self::Dafny),
            "verus" => Ok(Self::Verus),
            "lean" => Ok(Self::Lean),
            other => Err(format!("unknown formal backend: {other}")),
        }
    }
}

impl std::fmt::Display for FormalBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Pure evaluation of formal outcome (discovery + tool I/O).
///
/// Used by unit tests and by [`run_formal`] after walking / spawning.
pub fn check(
    has_targets: bool,
    tool_missing: bool,
    exit_code: Option<i32>,
    stdout: &str,
    stderr: &str,
) -> AspectResult {
    if !has_targets {
        return AspectResult {
            aspect: AdvancedAspect::Formal,
            status: AspectStatus::Skipped,
            score: None,
            reason: Some("no_targets".into()),
            exit_code: None,
            duration_ms: 0,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
        };
    }

    if tool_missing {
        return AspectResult {
            aspect: AdvancedAspect::Formal,
            status: AspectStatus::Fail,
            score: None,
            reason: Some("FORMAL_TOOL_MISSING".into()),
            exit_code: None,
            duration_ms: 0,
            stdout_tail: String::new(),
            stderr_tail: MSG_FORMAL_MISSING.into(),
        };
    }

    let (stdout_tail, _) = truncate_chars(stdout.to_string(), 4000);
    let (stderr_tail, _) = truncate_chars(stderr.to_string(), 4000);

    if contains_bypass_marker(stdout) || contains_bypass_marker(stderr) {
        return AspectResult {
            aspect: AdvancedAspect::Formal,
            status: AspectStatus::Fail,
            score: None,
            reason: Some("anti_bypass".into()),
            exit_code: exit_code.or(Some(1)),
            duration_ms: 0,
            stdout_tail,
            stderr_tail,
        };
    }

    let code = exit_code.unwrap_or(1);
    if code == 0 {
        AspectResult {
            aspect: AdvancedAspect::Formal,
            status: AspectStatus::Pass,
            score: Some(1.0),
            reason: None,
            exit_code: Some(0),
            duration_ms: 0,
            stdout_tail,
            stderr_tail,
        }
    } else {
        AspectResult {
            aspect: AdvancedAspect::Formal,
            status: AspectStatus::Fail,
            score: None,
            reason: Some("formal_failed".into()),
            exit_code: Some(code),
            duration_ms: 0,
            stdout_tail,
            stderr_tail,
        }
    }
}

/// Discover `@dare-formal` targets and run the chosen backend via [`ProcessRunner`].
///
/// Walk caps at [`FORMAL_WALK_CAP`] entries; skips `target` / `node_modules` / `.git`.
/// No targets → skipped `no_targets`. Tool missing → fail `FORMAL_TOOL_MISSING`.
pub fn run_formal(
    root: &ProjectRoot,
    backend: FormalBackend,
    runner: &dyn ProcessRunner,
) -> CoreResult<AspectResult> {
    let start = Instant::now();
    let targets = discover_formal_targets(root.as_path().as_std_path())?;
    if targets.is_empty() {
        let mut r = check(false, false, None, "", "");
        r.duration_ms = start.elapsed().as_millis() as u64;
        return Ok(r);
    }

    let root_rel = SafeRelativePath::new(".")?;
    let mut combined_stdout = String::new();
    let mut combined_stderr = String::new();
    let mut last_exit: Option<i32> = None;

    for target in &targets {
        let rel = path_relative_posix(root.as_path().as_std_path(), target)?;
        let cmd = SafeCommand::new(backend.program())
            .args(backend.args_for(&rel))
            .cwd(root.clone(), root_rel.clone())
            .timeout(std::time::Duration::from_secs(FORMAL_TIMEOUT_SECS));

        match runner.run(&cmd) {
            Ok(out) => {
                if !combined_stdout.is_empty() && !out.stdout.is_empty() {
                    combined_stdout.push('\n');
                }
                combined_stdout.push_str(&out.stdout);
                if !combined_stderr.is_empty() && !out.stderr.is_empty() {
                    combined_stderr.push('\n');
                }
                combined_stderr.push_str(&out.stderr);
                let code = if out.timed_out { 124 } else { out.exit_code };
                last_exit = Some(code);
                if code != 0 {
                    break;
                }
            }
            Err(e) if e.kind() == dare_core::ErrorKind::NotFound => {
                let mut r = check(true, true, None, "", "");
                r.duration_ms = start.elapsed().as_millis() as u64;
                return Ok(r);
            }
            Err(e) => return Err(e),
        }
    }

    let mut r = check(
        true,
        false,
        last_exit,
        &combined_stdout,
        &combined_stderr,
    );
    r.duration_ms = start.elapsed().as_millis() as u64;
    Ok(r)
}

fn contains_bypass_marker(text: &str) -> bool {
    let upper = text.to_ascii_uppercase();
    BYPASS_MARKERS.iter().any(|m| upper.contains(m))
}

/// Walk project root for files whose content contains `@dare-formal`.
///
/// Counts every visited entry (file or dir) toward [`FORMAL_WALK_CAP`].
fn discover_formal_targets(root: &Path) -> CoreResult<Vec<PathBuf>> {
    let mut targets = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    let mut visited = 0usize;

    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) => return Err(CoreError::io(e.to_string())),
        };

        for entry in entries {
            if visited >= FORMAL_WALK_CAP {
                return Ok(targets);
            }
            let entry = entry.map_err(|e| CoreError::io(e.to_string()))?;
            visited += 1;

            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            let ft = entry
                .file_type()
                .map_err(|e| CoreError::io(e.to_string()))?;
            if ft.is_dir() {
                if SKIP_DIRS.iter().any(|s| name_str == *s) {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if !ft.is_file() {
                continue;
            }

            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            if content.contains(MARKER_TAG) {
                targets.push(path);
            }
        }
    }

    targets.sort();
    Ok(targets)
}

fn path_relative_posix(root: &Path, file: &Path) -> CoreResult<String> {
    let rel = file
        .strip_prefix(root)
        .map_err(|_| CoreError::invalid_input("formal target escaped project root"))?;
    let s = rel.to_string_lossy().replace('\\', "/");
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::{MockProcessRunner, ProcessOutput};
    use tempfile::tempdir;

    #[test]
    fn formal_backend_parse_and_default() {
        assert_eq!(FormalBackend::default(), FormalBackend::Dafny);
        assert_eq!("dafny".parse::<FormalBackend>().unwrap(), FormalBackend::Dafny);
        assert_eq!("verus".parse::<FormalBackend>().unwrap(), FormalBackend::Verus);
        assert_eq!("LEAN".parse::<FormalBackend>().unwrap(), FormalBackend::Lean);
        let err = "coq".parse::<FormalBackend>().unwrap_err();
        assert!(err.contains("unknown formal backend: coq"));
    }

    #[test]
    fn formal_no_targets() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let mock = MockProcessRunner::new();
        let r = run_formal(&root, FormalBackend::Dafny, &mock).unwrap();
        assert_eq!(r.aspect, AdvancedAspect::Formal);
        assert_eq!(r.status, AspectStatus::Skipped);
        assert_eq!(r.reason.as_deref(), Some("no_targets"));
    }

    #[test]
    fn formal_missing_tool() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("Spec.dfy"),
            "// @dare-formal\nmethod Main() {}\n",
        )
        .unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let mock = MockProcessRunner::new();
        mock.push_err(CoreError::not_found("executable not found"));
        let r = run_formal(&root, FormalBackend::Dafny, &mock).unwrap();
        assert_eq!(r.status, AspectStatus::Fail);
        assert_eq!(r.reason.as_deref(), Some("FORMAL_TOOL_MISSING"));
        assert!(r.stderr_tail.contains("FORMAL_TOOL_MISSING"));
        assert_eq!(r.stderr_tail, MSG_FORMAL_MISSING);
    }

    #[test]
    fn anti_bypass_marker() {
        let fake = check(true, false, Some(0), "proof ok FAKE_PROOF", "");
        assert_eq!(fake.status, AspectStatus::Fail);
        assert_eq!(fake.reason.as_deref(), Some("anti_bypass"));

        let bypass = check(true, false, Some(0), "", "bypass_formal done");
        assert_eq!(bypass.status, AspectStatus::Fail);
        assert_eq!(bypass.reason.as_deref(), Some("anti_bypass"));

        let ok = check(true, false, Some(0), "verified", "");
        assert_eq!(ok.status, AspectStatus::Pass);
    }

    #[test]
    fn run_formal_pass_with_mock() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.dfy"), "@dare-formal\n").unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let mock = MockProcessRunner::new();
        mock.push(ProcessOutput {
            exit_code: 0,
            stdout: "verified".into(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        });
        let r = run_formal(&root, FormalBackend::Dafny, &mock).unwrap();
        assert_eq!(r.status, AspectStatus::Pass);
    }

    #[test]
    fn walk_skips_target_dir() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("hidden.dfy"), "@dare-formal\n").unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let found = discover_formal_targets(root.as_path().as_std_path()).unwrap();
        assert!(found.is_empty());
    }
}
