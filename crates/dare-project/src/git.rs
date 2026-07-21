//! Git root discovery (walk `.git` + optional `git rev-parse`).

use std::path::{Path, PathBuf};
use std::time::Duration;

use dare_core::{ProcessRunner, ProjectRoot, SafeCommand, SafeRelativePath, SystemProcessRunner};

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
}
