//! Git root discovery (walk `.git` + optional `git rev-parse`) and provenance
//! primitives (HEAD sha, worktree state, changed files).
//!
//! Every helper here degrades to `None` when Git is absent or fails. A missing
//! revision is reported as "no revision", never as a fabricated one (RF-09).

use std::path::{Path, PathBuf};
use std::time::Duration;

use dare_core::{ProcessRunner, ProjectRoot, SafeCommand, SafeRelativePath, SystemProcessRunner};

/// Timeout shared by every provenance call, matching `try_git_rev_parse`.
const GIT_TIMEOUT: Duration = Duration::from_secs(5);

/// stdout cap for the short, fixed-shape provenance commands.
const PROVENANCE_STDOUT_LIMIT: usize = 4096;

/// stdout cap for `git diff --name-only`, whose output scales with the diff.
const DIFF_STDOUT_LIMIT: usize = 65536;

fn absolutize(start: &Path) -> PathBuf {
    if let Ok(c) = std::fs::canonicalize(start) {
        return c;
    }
    if start.is_absolute() {
        return start.to_path_buf();
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(start)
}

fn walk_git(start: &Path) -> Option<PathBuf> {
    let mut cur = absolutize(start);
    loop {
        if cur.join(".git").exists() {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

fn try_git_rev_parse(project_root: &Path) -> Option<PathBuf> {
    let root = ProjectRoot::new(project_root).ok()?;
    let rel = SafeRelativePath::new(".").ok()?;
    let cmd = SafeCommand::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .cwd(root, rel)
        .timeout(Duration::from_secs(5))
        .stdout_limit(4096);
    let out = SystemProcessRunner.run(&cmd).ok()?;
    if out.exit_code != 0 {
        return None;
    }
    let path = out.stdout.trim();
    if path.is_empty() {
        return None;
    }
    let pb = PathBuf::from(path);
    if pb.exists() {
        Some(pb)
    } else {
        None
    }
}

/// Find Git toplevel. Walk-up `.git` first; optional `git rev-parse` if project_root known.
/// Never returns `Err` — missing git degrades to `None`.
pub fn find_git_root(start: &Path, project_root: Option<&Path>) -> Option<PathBuf> {
    if let Some(g) = walk_git(start) {
        return Some(g);
    }
    if let Some(pr) = project_root {
        // Also walk from project_root in case start was outside ancestry somehow
        if let Some(g) = walk_git(pr) {
            return Some(g);
        }
        return try_git_rev_parse(pr);
    }
    None
}

fn is_lower_hex(b: u8) -> bool {
    matches!(b, b'0'..=b'9' | b'a'..=b'f')
}

/// `^[0-9a-f]{40}$` — a full object name, the only shape `head_sha` reports.
fn is_full_sha(s: &str) -> bool {
    s.len() == 40 && s.bytes().all(is_lower_hex)
}

/// `^[0-9a-f]{7,40}$` — the only shape allowed to reach `git` as a revision.
///
/// This is the argument-injection guard: anything else (`--upload-pack=evil`,
/// `--output=/etc/x`, `HEAD`, `; rm -rf /`) would be read by `git` as a flag or
/// a revision expression, so it is rejected before a command is ever built.
fn is_revision_arg(s: &str) -> bool {
    (7..=40).contains(&s.len()) && s.bytes().all(is_lower_hex)
}

/// Build a `git` invocation rooted at `root`, mirroring `try_git_rev_parse`.
fn git_command(root: &ProjectRoot, args: &[&str], stdout_limit: usize) -> Option<SafeCommand> {
    let rel = SafeRelativePath::new(".").ok()?;
    Some(
        SafeCommand::new("git")
            .args(args.iter().copied())
            .cwd(root.clone(), rel)
            .timeout(GIT_TIMEOUT)
            .stdout_limit(stdout_limit),
    )
}

fn head_sha_with<R: ProcessRunner>(runner: &R, root: &ProjectRoot) -> Option<String> {
    let cmd = git_command(root, &["rev-parse", "HEAD"], PROVENANCE_STDOUT_LIMIT)?;
    let out = runner.run(&cmd).ok()?;
    if out.exit_code != 0 {
        return None;
    }
    let sha = out.stdout.trim();
    if is_full_sha(sha) {
        Some(sha.to_string())
    } else {
        None
    }
}

/// Commit sha of `HEAD`, or `None` when there is no attestable revision.
///
/// `Some` only when `git rev-parse HEAD` exits 0 **and** its trimmed stdout is a
/// 40-character lowercase hex object name. A repository with no commits yet, a
/// directory outside any repository, and a machine without `git` all yield
/// `None` — never an invented sha.
pub fn head_sha(root: &ProjectRoot) -> Option<String> {
    head_sha_with(&SystemProcessRunner, root)
}

fn worktree_is_dirty_with<R: ProcessRunner>(runner: &R, root: &ProjectRoot) -> Option<bool> {
    let cmd = git_command(root, &["status", "--porcelain"], PROVENANCE_STDOUT_LIMIT)?;
    let out = runner.run(&cmd).ok()?;
    if out.exit_code != 0 {
        return None;
    }
    // Truncation is harmless here: a truncated porcelain listing is still a
    // non-empty one, and non-empty is exactly what "dirty" means.
    Some(!out.stdout.trim().is_empty())
}

/// Whether the working tree has uncommitted changes.
///
/// `Some(true)` when `git status --porcelain` prints anything, `Some(false)`
/// when it prints nothing, `None` when `git` is unavailable or exits non-zero.
pub fn worktree_is_dirty(root: &ProjectRoot) -> Option<bool> {
    worktree_is_dirty_with(&SystemProcessRunner, root)
}

fn changed_files_with<R: ProcessRunner>(
    runner: &R,
    root: &ProjectRoot,
    from_sha: &str,
    to_sha: Option<&str>,
) -> Option<Vec<String>> {
    // Injection guard runs BEFORE the command is built: an unvalidated string
    // never reaches `git`, not even as a discarded argument.
    if !is_revision_arg(from_sha) {
        return None;
    }
    let range = match to_sha {
        Some(to) => {
            if !is_revision_arg(to) {
                return None;
            }
            format!("{from_sha}..{to}")
        }
        None => from_sha.to_string(),
    };

    let cmd = git_command(
        root,
        &["diff", "--name-only", range.as_str()],
        DIFF_STDOUT_LIMIT,
    )?;
    let out = runner.run(&cmd).ok()?;
    if out.exit_code != 0 {
        return None;
    }

    let mut lines: Vec<&str> = out.stdout.split('\n').collect();
    if out.stdout_truncated {
        // The last line of a truncated stream may be a fragment; reporting it
        // would fabricate a path that does not exist.
        lines.pop();
    }

    let mut paths: Vec<String> = lines
        .iter()
        .filter_map(|line| SafeRelativePath::new(line).ok())
        // `SafeRelativePath` rejects absolute forms and `..`, so anything that
        // escapes the project root is already gone; "." is the root itself, not
        // a changed file.
        .filter(|p| p.as_str() != ".")
        .map(|p| p.as_str().to_string())
        .collect();
    paths.sort();
    paths.dedup();
    Some(paths)
}

/// Paths changed between two revisions, as sorted, deduplicated POSIX paths.
///
/// `git diff --name-only <from>..<to>`, or `git diff --name-only <from>` when
/// `to_sha` is `None`, which also folds in the working tree. Both revisions must
/// match `^[0-9a-f]{7,40}$`; anything else returns `None` without running `git`.
/// Entries that escape the project root are dropped.
pub fn changed_files(
    root: &ProjectRoot,
    from_sha: &str,
    to_sha: Option<&str>,
) -> Option<Vec<String>> {
    changed_files_with(&SystemProcessRunner, root, from_sha, to_sha)
}

#[cfg(test)]
mod git_tests {
    use super::*;
    use std::fs;

    #[test]
    fn git_root_dot_git_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let g = find_git_root(dir.path(), Some(dir.path())).expect("git");
        assert_eq!(
            g.canonicalize().unwrap(),
            dir.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn git_missing_degrades_to_null() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        assert!(find_git_root(dir.path(), Some(dir.path())).is_none());
    }

    // --- provenance helpers -------------------------------------------------

    use dare_core::{CoreResult, ProcessOutput};

    /// A runner that fails the test if it is ever asked to spawn anything.
    /// Used to prove the injection guard short-circuits before the command runs.
    struct NeverRunner;

    impl ProcessRunner for NeverRunner {
        fn run(&self, cmd: &SafeCommand) -> CoreResult<ProcessOutput> {
            panic!(
                "no process may be spawned for a rejected revision (attempted: {} {:?})",
                cmd.program(),
                cmd.arg_list()
            );
        }
    }

    fn plain_dir() -> (tempfile::TempDir, ProjectRoot) {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("project root");
        (dir, root)
    }

    fn run_git(root: &ProjectRoot, args: &[&str]) -> Option<ProcessOutput> {
        let rel = SafeRelativePath::new(".").ok()?;
        let cmd = SafeCommand::new("git")
            .args(args.iter().copied())
            .cwd(root.clone(), rel)
            .timeout(Duration::from_secs(30))
            .stdout_limit(DIFF_STDOUT_LIMIT);
        SystemProcessRunner.run(&cmd).ok()
    }

    fn git_ok(root: &ProjectRoot, args: &[&str]) -> bool {
        matches!(run_git(root, args), Some(out) if out.exit_code == 0)
    }

    /// `true` when a real `git` is on PATH; repo-backed tests skip otherwise so
    /// a CI image without git stays green instead of failing.
    fn git_available() -> bool {
        let Ok(dir) = tempfile::tempdir() else {
            return false;
        };
        let Ok(root) = ProjectRoot::new(dir.path()) else {
            return false;
        };
        git_ok(&root, &["--version"])
    }

    fn commit_all(root: &ProjectRoot, message: &str) -> bool {
        git_ok(root, &["add", "-A"])
            && git_ok(
                root,
                &["-c", "commit.gpgsign=false", "commit", "-m", message],
            )
    }

    /// Initialise a repository with one commit containing `a.txt`.
    /// Returns `None` when git is absent, which the caller treats as a skip.
    fn init_repo() -> Option<(tempfile::TempDir, ProjectRoot)> {
        if !git_available() {
            eprintln!("skipping: no `git` on PATH");
            return None;
        }
        let (dir, root) = plain_dir();
        let setup = git_ok(&root, &["init"])
            && git_ok(&root, &["config", "user.email", "dare@example.invalid"])
            && git_ok(&root, &["config", "user.name", "DARE Test"])
            && git_ok(&root, &["config", "core.autocrlf", "false"]);
        assert!(setup, "git repository setup failed");
        fs::write(dir.path().join("a.txt"), "one\n").expect("write a.txt");
        assert!(commit_all(&root, "c1"), "initial commit failed");
        Some((dir, root))
    }

    #[test]
    fn head_sha_without_git_is_none() {
        let (_dir, root) = plain_dir();
        assert_eq!(head_sha(&root), None);
    }

    #[test]
    fn worktree_dirty_without_git_is_none() {
        let (_dir, root) = plain_dir();
        assert_eq!(worktree_is_dirty(&root), None);
    }

    #[test]
    fn changed_files_without_git_is_none() {
        let (_dir, root) = plain_dir();
        assert_eq!(changed_files(&root, "0123456789abcdef", None), None);
    }

    #[test]
    fn head_sha_shape_in_real_repo() {
        let Some((_dir, root)) = init_repo() else {
            return;
        };
        let sha = head_sha(&root).expect("head sha");
        assert_eq!(sha.len(), 40, "{sha}");
        assert!(
            sha.bytes()
                .all(|b| b.is_ascii_digit() || b.is_ascii_lowercase()),
            "{sha}"
        );
        assert!(is_full_sha(&sha), "{sha}");
    }

    #[test]
    fn head_sha_rejects_malformed_stdout() {
        // Exercises the shape guard independently of a real repository.
        assert!(!is_full_sha(""));
        assert!(!is_full_sha("HEAD"));
        assert!(!is_full_sha(&"a".repeat(39)));
        assert!(!is_full_sha(&"A".repeat(40)));
        assert!(is_full_sha(&"0a".repeat(20)));
    }

    #[test]
    fn worktree_clean_after_commit() {
        let Some((_dir, root)) = init_repo() else {
            return;
        };
        assert_eq!(worktree_is_dirty(&root), Some(false));
    }

    #[test]
    fn worktree_dirty_after_touch() {
        let Some((dir, root)) = init_repo() else {
            return;
        };
        fs::write(dir.path().join("a.txt"), "one\ntwo\n").expect("modify a.txt");
        assert_eq!(worktree_is_dirty(&root), Some(true));
    }

    #[test]
    fn changed_files_between_commits() {
        let Some((dir, root)) = init_repo() else {
            return;
        };
        let first = head_sha(&root).expect("first sha");
        fs::write(dir.path().join("b.txt"), "two\n").expect("write b.txt");
        assert!(commit_all(&root, "c2"), "second commit failed");
        let second = head_sha(&root).expect("second sha");
        assert_ne!(first, second);

        let changed = changed_files(&root, &first, Some(&second)).expect("diff");
        assert_eq!(changed, vec!["b.txt".to_string()]);
    }

    #[test]
    fn changed_files_from_commit_includes_worktree() {
        let Some((dir, root)) = init_repo() else {
            return;
        };
        let first = head_sha(&root).expect("first sha");
        fs::write(dir.path().join("a.txt"), "one\ntwo\n").expect("modify a.txt");

        let changed = changed_files(&root, &first, None).expect("diff");
        assert_eq!(changed, vec!["a.txt".to_string()]);
    }

    #[test]
    fn changed_files_rejects_flag_injection() {
        let (_dir, root) = plain_dir();
        assert_eq!(
            changed_files_with(&NeverRunner, &root, "--upload-pack=evil", None),
            None
        );
        assert_eq!(
            changed_files_with(
                &NeverRunner,
                &root,
                &"a".repeat(40),
                Some("--output=/etc/x")
            ),
            None
        );
    }

    #[test]
    fn changed_files_rejects_shell_metachar() {
        let (_dir, root) = plain_dir();
        assert_eq!(
            changed_files_with(&NeverRunner, &root, "; rm -rf /", None),
            None
        );
        assert_eq!(
            changed_files_with(&NeverRunner, &root, &"b".repeat(8), Some("a|b&c$(x)")),
            None
        );
    }

    #[test]
    fn changed_files_rejects_non_hex() {
        let (_dir, root) = plain_dir();
        for bad in ["zzz", "HEAD", "", "ABCDEF0", &"a".repeat(41)] {
            assert_eq!(
                changed_files_with(&NeverRunner, &root, bad, None),
                None,
                "{bad:?} must be rejected"
            );
            assert_eq!(
                changed_files_with(&NeverRunner, &root, &"c".repeat(12), Some(bad)),
                None,
                "{bad:?} must be rejected as to_sha"
            );
        }
    }

    #[test]
    fn revision_arg_accepts_only_short_or_full_hex() {
        assert!(is_revision_arg(&"a".repeat(7)));
        assert!(is_revision_arg(&"0f".repeat(20)));
        assert!(!is_revision_arg(&"a".repeat(6)));
        assert!(!is_revision_arg(&"a".repeat(41)));
    }
}
