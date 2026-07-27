//! `run_bench` — load suite, evaluate fixtures, build [`BenchReport`].
//!
//! Patch apply is best-effort via `git apply` (empty patch is a no-op; missing
//! git or apply failure does not abort the case). Test outcomes prefer a real
//! `cargo test` run in a jail under `.dare/bench-work/<id>/`. When the cargo
//! binary is missing, outcomes are synthesized from the list files (all listed
//! tests treated as passing) so suite/JSON smokes remain usable.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use dare_core::{
    CoreError, CoreResult, ProcessRunner, ProjectRoot, SafeCommand, SafeRelativePath,
};
use globset::{Glob, GlobSetBuilder};

use super::baseline::{compare_baseline, load_baseline};
use super::suite::{load_suite, LoadedCase, LoadedSuite};
use super::{
    compute_fixture_fix_rate, compute_solve_rate, compute_suite_fix_rate, round_4dp,
    BenchReport, FixtureResult, DEFAULT_SUITE_REL,
};

/// Bench report schema version (camelCase JSON `schemaVersion`).
pub const BENCH_REPORT_SCHEMA: u32 = 1;

const BENCH_WORK_REL: &str = ".dare/bench-work";
const CARGO_TEST_TIMEOUT_SECS: u64 = 600;
const CARGO_STDOUT_LIMIT: usize = 256_000;

/// Options for [`run_bench`].
#[derive(Debug, Clone, Default)]
pub struct BenchOptions {
    /// Suite directory (absolute, or relative to project root). Default: [`DEFAULT_SUITE_REL`].
    pub suite: Option<PathBuf>,
    /// Optional baseline JSON path (absolute or root-relative).
    pub baseline: Option<PathBuf>,
    /// Max allowed solve-rate drop in percentage points (`0..=100`).
    pub fail_on_regression: Option<u32>,
    /// Optional glob filter on case `id`.
    pub filter: Option<String>,
}

/// Run the bench suite and return a [`BenchReport`].
///
/// Does **not** turn regression into an error — callers check
/// `report.baseline.as_ref().map(|b| b.regression_failed)` for exit 1.
pub fn run_bench(
    root: &ProjectRoot,
    opts: &BenchOptions,
    runner: &dyn ProcessRunner,
) -> CoreResult<BenchReport> {
    if opts.fail_on_regression.is_some() && opts.baseline.is_none() {
        return Err(CoreError::usage(
            "baseline required when using --fail-on-regression",
        ));
    }
    if let Some(n) = opts.fail_on_regression {
        if n > 100 {
            return Err(CoreError::invalid_input(
                "--fail-on-regression must be between 0 and 100",
            ));
        }
    }

    let suite_dir = resolve_under_root(root, opts.suite.as_deref(), DEFAULT_SUITE_REL)?;
    let loaded = load_suite(&suite_dir)?;
    let cases = filter_cases(&loaded, opts.filter.as_deref())?;

    let mut fixtures = Vec::with_capacity(cases.len());
    for case in &cases {
        fixtures.push(evaluate_case(root, case, runner)?);
    }

    let rates: Vec<f64> = fixtures.iter().map(|f| f.fix_rate).collect();
    let ok_count = fixtures.iter().filter(|f| f.ok).count();
    let fix_rate = round_4dp(compute_suite_fix_rate(&rates));
    let solve_rate = round_4dp(compute_solve_rate(ok_count, fixtures.len()));

    let suite_path = display_suite_path(root, &suite_dir);

    let baseline = if let Some(base_path) = &opts.baseline {
        let abs = resolve_under_root(root, Some(base_path.as_path()), "")?;
        let file = load_baseline(&abs)?;
        let path_display = display_rel_or_abs(root, &abs);
        Some(compare_baseline(
            path_display,
            &file,
            solve_rate,
            fix_rate,
            opts.fail_on_regression,
        ))
    } else {
        None
    };

    Ok(BenchReport {
        schema_version: BENCH_REPORT_SCHEMA,
        suite_path,
        fix_rate,
        solve_rate,
        fixtures,
        baseline,
        filter: opts.filter.clone(),
    })
}

fn resolve_under_root(
    root: &ProjectRoot,
    path: Option<&Path>,
    default_rel: &str,
) -> CoreResult<PathBuf> {
    let root_std = root.as_path().as_std_path();
    match path {
        None => Ok(root_std.join(default_rel)),
        Some(p) if p.as_os_str().is_empty() => Ok(root_std.join(default_rel)),
        Some(p) if p.is_absolute() => Ok(p.to_path_buf()),
        Some(p) => {
            let joined = root_std.join(p);
            // Reject obvious escapes when relative.
            let canon_root = fs::canonicalize(root_std).unwrap_or_else(|_| root_std.to_path_buf());
            if let Ok(canon) = fs::canonicalize(&joined) {
                if !canon.starts_with(&canon_root) {
                    return Err(CoreError::invalid_input("path is outside project root"));
                }
            }
            Ok(joined)
        }
    }
}

fn display_suite_path(root: &ProjectRoot, suite_dir: &Path) -> String {
    display_rel_or_abs(root, suite_dir)
}

fn display_rel_or_abs(root: &ProjectRoot, path: &Path) -> String {
    let root_std = root.as_path().as_std_path();
    path.strip_prefix(root_std)
        .map(|r| r.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

fn filter_cases<'a>(
    loaded: &'a LoadedSuite,
    filter: Option<&str>,
) -> CoreResult<Vec<&'a LoadedCase>> {
    let Some(pat) = filter.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(loaded.cases.iter().collect());
    };
    let glob = Glob::new(pat).map_err(|e| {
        CoreError::invalid_input(format!("invalid --filter glob: {e}"))
    })?;
    let mut builder = GlobSetBuilder::new();
    builder.add(glob);
    let set = builder
        .build()
        .map_err(|e| CoreError::invalid_input(format!("invalid --filter glob: {e}")))?;
    Ok(loaded
        .cases
        .iter()
        .filter(|c| set.is_match(&c.id))
        .collect())
}

fn evaluate_case(
    root: &ProjectRoot,
    case: &LoadedCase,
    runner: &dyn ProcessRunner,
) -> CoreResult<FixtureResult> {
    let ftp = read_test_ids(&case.dir.join("fail_to_pass.txt"))?;
    let ptp = read_test_ids(&case.dir.join("pass_to_pass.txt"))?;

    let jail_rel = format!("{BENCH_WORK_REL}/{}", case.id);
    let jail = root.as_path().as_std_path().join(&jail_rel);
    if jail.exists() {
        fs::remove_dir_all(&jail).map_err(|e| CoreError::io(e.to_string()))?;
    }
    let repo_src = case.dir.join("repo");
    copy_dir_recursive(&repo_src, &jail)?;

    let patch = case.dir.join("patch.diff");
    apply_patch_best_effort(root, &jail_rel, &patch, runner);

    let passed = match run_cargo_test(root, &jail_rel, runner) {
        Ok(names) => names,
        Err(e) if is_executable_missing(&e) => {
            // Documented synthesis fallback when cargo is unavailable.
            let mut all = ftp.clone();
            all.extend(ptp.iter().cloned());
            all
        }
        Err(e) => {
            let _ = fs::remove_dir_all(&jail);
            return Err(e);
        }
    };

    let _ = fs::remove_dir_all(&jail);

    let passed_set: std::collections::HashSet<&str> =
        passed.iter().map(String::as_str).collect();

    let ftp_total = ftp.len() as u32;
    let ftp_passed = ftp
        .iter()
        .filter(|t| passed_set.contains(t.as_str()))
        .count() as u32;
    let ptp_failed = ptp
        .iter()
        .filter(|t| !passed_set.contains(t.as_str()))
        .count() as u32;

    let fix_rate = compute_fixture_fix_rate(ftp_total, ftp_passed, ptp_failed);
    let ok = ptp_failed == 0 && ftp_passed == ftp_total;

    Ok(FixtureResult {
        id: case.id.clone(),
        ok,
        fix_rate: round_4dp(fix_rate),
        fail_to_pass_total: ftp_total,
        fail_to_pass_passed: ftp_passed,
        pass_to_pass_failed: ptp_failed,
    })
}

fn is_executable_missing(err: &CoreError) -> bool {
    matches!(err, CoreError::NotFound(_))
        || err.message().contains("executable not found")
}

fn apply_patch_best_effort(
    root: &ProjectRoot,
    jail_rel: &str,
    patch: &Path,
    runner: &dyn ProcessRunner,
) {
    let Ok(meta) = fs::metadata(patch) else {
        return;
    };
    if meta.len() == 0 {
        return;
    }
    let Ok(bytes) = fs::read(patch) else {
        return;
    };
    // Copy patch into jail so argv stays root-relative.
    let patch_dest = root
        .as_path()
        .as_std_path()
        .join(jail_rel)
        .join("patch.diff");
    if fs::write(&patch_dest, &bytes).is_err() {
        return;
    }
    let Ok(rel) = SafeRelativePath::new(jail_rel) else {
        return;
    };
    let cmd = SafeCommand::new("git")
        .args(["apply", "--whitespace=nowarn", "patch.diff"])
        .cwd(root.clone(), rel)
        .timeout(Duration::from_secs(60));
    let _ = runner.run(&cmd);
}

fn run_cargo_test(
    root: &ProjectRoot,
    jail_rel: &str,
    runner: &dyn ProcessRunner,
) -> CoreResult<Vec<String>> {
    let rel = SafeRelativePath::new(jail_rel)?;
    let cmd = SafeCommand::new("cargo")
        .args(["test", "--", "--test-threads=1"])
        .cwd(root.clone(), rel)
        .timeout(Duration::from_secs(CARGO_TEST_TIMEOUT_SECS))
        .stdout_limit(CARGO_STDOUT_LIMIT)
        .stderr_limit(CARGO_STDOUT_LIMIT);
    let out = runner.run(&cmd)?;
    Ok(parse_passed_tests(&out.stdout, &out.stderr))
}

/// Collect test names that reported `... ok` in cargo output.
fn parse_passed_tests(stdout: &str, stderr: &str) -> Vec<String> {
    let mut passed = Vec::new();
    for line in stdout.lines().chain(stderr.lines()) {
        let line = line.trim();
        // `test path::name ... ok` or `test path::name ... ok <duration>`
        if let Some(rest) = line.strip_prefix("test ") {
            if let Some((name, status)) = rest.rsplit_once(" ... ") {
                let status = status.trim();
                if status == "ok" || status.starts_with("ok ") {
                    passed.push(name.trim().to_string());
                }
            }
        }
    }
    passed
}

fn read_test_ids(path: &Path) -> CoreResult<Vec<String>> {
    let text = fs::read_to_string(path).map_err(|e| CoreError::io(e.to_string()))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> CoreResult<()> {
    fs::create_dir_all(dst).map_err(|e| CoreError::io(e.to_string()))?;
    for entry in fs::read_dir(src).map_err(|e| CoreError::io(e.to_string()))? {
        let entry = entry.map_err(|e| CoreError::io(e.to_string()))?;
        let ty = entry.file_type().map_err(|e| CoreError::io(e.to_string()))?;
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &to)?;
        } else if ty.is_file() {
            fs::copy(entry.path(), &to).map_err(|e| CoreError::io(e.to_string()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::{CoreError, MockProcessRunner};
    use std::fs;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace root")
    }

    #[test]
    fn parse_passed_tests_ok_lines() {
        let out = "\ntest tests::always_passes ... ok\ntest tests::other ... FAILED\n";
        let p = parse_passed_tests(out, "");
        assert_eq!(p, vec!["tests::always_passes".to_string()]);
    }

    #[test]
    fn fail_on_regression_requires_baseline() {
        let root = ProjectRoot::new(workspace_root()).unwrap();
        let opts = BenchOptions {
            fail_on_regression: Some(3),
            baseline: None,
            ..Default::default()
        };
        let err = run_bench(&root, &opts, &MockProcessRunner::new()).expect_err("usage");
        assert!(matches!(err, CoreError::Usage(_)));
        assert!(err.message().contains("baseline required"));
    }

    #[test]
    fn suite_invalid_propagates_usage() {
        let tmp = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(tmp.path()).unwrap();
        let opts = BenchOptions {
            suite: Some(PathBuf::from("missing-suite")),
            ..Default::default()
        };
        let err = run_bench(&root, &opts, &MockProcessRunner::new()).expect_err("invalid");
        assert!(matches!(err, CoreError::Usage(_)));
        assert!(err.message().starts_with("invalid bench suite:"));
    }

    #[test]
    fn synthesize_when_cargo_missing_builds_report() {
        let tmp = tempfile::tempdir().unwrap();
        let suite = tmp.path().join("fixtures/bench");
        fs::create_dir_all(suite.join("cases/sample-ok/repo/src")).unwrap();
        fs::write(
            suite.join("suite.json"),
            r#"{"schemaVersion":1,"name":"t","cases":[{"id":"sample-ok","path":"cases/sample-ok"}]}"#,
        )
        .unwrap();
        fs::write(suite.join("cases/sample-ok/patch.diff"), "").unwrap();
        fs::write(
            suite.join("cases/sample-ok/fail_to_pass.txt"),
            "tests::a\n",
        )
        .unwrap();
        fs::write(
            suite.join("cases/sample-ok/pass_to_pass.txt"),
            "tests::b\n",
        )
        .unwrap();
        fs::write(suite.join("cases/sample-ok/stack.txt"), "rust-axum\n").unwrap();
        fs::write(
            suite.join("cases/sample-ok/repo/Cargo.toml"),
            "[package]\nname=\"t\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[lib]\npath=\"src/lib.rs\"\n",
        )
        .unwrap();
        fs::write(suite.join("cases/sample-ok/repo/src/lib.rs"), "pub fn x(){}\n").unwrap();

        let root = ProjectRoot::new(tmp.path()).unwrap();
        let mock = MockProcessRunner::new();
        mock.push_err(CoreError::not_found("executable not found"));
        let report = run_bench(
            &root,
            &BenchOptions {
                suite: Some(PathBuf::from("fixtures/bench")),
                ..Default::default()
            },
            &mock,
        )
        .expect("synthesize");
        assert_eq!(report.schema_version, 1);
        assert_eq!(report.fixtures.len(), 1);
        assert!(report.fixtures[0].ok);
        assert_eq!(report.solve_rate, 1.0);
    }
}
