//! Git worktree jail under `.dare/agent-worktrees/` (microplano 030).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use dare_core::{CoreError, CoreResult, ProcessRunner, ProjectRoot, SafeCommand, SafeRelativePath};

/// Relative directory for agent worktrees (jail root under project).
pub const AGENT_WORKTREES_REL: &str = ".dare/agent-worktrees";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeSpec {
    pub task_id: String,
    pub attempt: u32,
    pub branch: String,
    pub rel_path: String,
}

pub struct WorktreeManager {
    root: ProjectRoot,
    runner: Arc<dyn ProcessRunner>,
}

impl WorktreeManager {
    pub fn new(root: ProjectRoot, runner: Arc<dyn ProcessRunner>) -> Self {
        Self { root, runner }
    }

    pub fn create(&self, task_id: &str, attempt: u32) -> CoreResult<WorktreeSpec> {
        validate_task_id(task_id)?;
        if !self.root.as_path().join(".git").exists() {
            return Err(CoreError::invalid_input(
                "agent worktrees require a git repository (.git missing)",
            ));
        }

        let branch = format!("dare/agent-{task_id}-{attempt}");
        let rel_path = format!("{AGENT_WORKTREES_REL}/{task_id}-{attempt}");
        let rel = SafeRelativePath::new(&rel_path)?;

        let parent = self.root.as_path().join(AGENT_WORKTREES_REL);
        std::fs::create_dir_all(&parent)
            .map_err(|e| CoreError::internal(format!("create agent-worktrees dir: {e}")))?;

        let cwd_rel = SafeRelativePath::new(".")?;
        let cmd = SafeCommand::new("git")
            .args([
                "worktree".into(),
                "add".into(),
                "-b".into(),
                branch.clone(),
                rel.as_str().to_string(),
                "HEAD".into(),
            ])
            .cwd(self.root.clone(), cwd_rel)
            .timeout(Duration::from_secs(120));

        let out = self.runner.run(&cmd)?;
        if out.exit_code != 0 {
            return Err(CoreError::internal(format!(
                "git worktree add failed (exit {}): {}",
                out.exit_code,
                out.stderr.trim()
            )));
        }

        Ok(WorktreeSpec {
            task_id: task_id.to_string(),
            attempt,
            branch,
            rel_path,
        })
    }

    pub fn remove(&self, spec: &WorktreeSpec) -> CoreResult<()> {
        let rel = SafeRelativePath::new(&spec.rel_path)?;
        let cwd_rel = SafeRelativePath::new(".")?;
        let cmd = SafeCommand::new("git")
            .args([
                "worktree".into(),
                "remove".into(),
                "--force".into(),
                rel.as_str().to_string(),
            ])
            .cwd(self.root.clone(), cwd_rel)
            .timeout(Duration::from_secs(120));

        let out = self.runner.run(&cmd)?;
        if out.exit_code != 0 {
            return Err(CoreError::internal(format!(
                "git worktree remove failed (exit {}): {}",
                out.exit_code,
                out.stderr.trim()
            )));
        }
        Ok(())
    }

    pub fn list_orphans(&self) -> CoreResult<Vec<PathBuf>> {
        let base = self.root.as_path().join(AGENT_WORKTREES_REL);
        if !base.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        let entries = std::fs::read_dir(&base)
            .map_err(|e| CoreError::internal(format!("read agent-worktrees: {e}")))?;
        for entry in entries {
            let entry = entry.map_err(|e| CoreError::internal(format!("read dir entry: {e}")))?;
            let path = entry.path();
            if path.is_dir() {
                out.push(path);
            }
        }
        out.sort();
        Ok(out)
    }

    pub fn cleanup_all(&self) -> CoreResult<usize> {
        let orphans = self.list_orphans()?;
        let mut removed = 0usize;
        for path in orphans {
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| CoreError::invalid_input("unsafe orphan worktree name"))?;
            // Parse `{task_id}-{attempt}` — attempt is trailing numeric segment.
            let (task_id, attempt) = split_orphan_name(name)?;
            let spec = WorktreeSpec {
                task_id: task_id.clone(),
                attempt,
                branch: format!("dare/agent-{task_id}-{attempt}"),
                rel_path: format!("{AGENT_WORKTREES_REL}/{name}"),
            };
            match self.remove(&spec) {
                Ok(()) => removed += 1,
                Err(_) => {
                    // Best-effort: remove directory if git worktree remove fails
                    if std::fs::remove_dir_all(&path).is_ok() {
                        removed += 1;
                    }
                }
            }
        }
        Ok(removed)
    }
}

fn validate_task_id(task_id: &str) -> CoreResult<()> {
    if task_id.is_empty()
        || task_id.contains('/')
        || task_id.contains('\\')
        || task_id.contains("..")
        || task_id.contains('\0')
        || Path::new(task_id).components().count() != 1
    {
        return Err(CoreError::invalid_input(
            "task id is not path-safe for agent worktrees",
        ));
    }
    // Probe jail join
    let probe = format!("{AGENT_WORKTREES_REL}/{task_id}-1");
    SafeRelativePath::new(&probe)?;
    Ok(())
}

fn split_orphan_name(name: &str) -> CoreResult<(String, u32)> {
    let (task_id, attempt_s) = name.rsplit_once('-').ok_or_else(|| {
        CoreError::invalid_input(format!("orphan worktree name missing attempt: {name}"))
    })?;
    if task_id.is_empty() {
        return Err(CoreError::invalid_input("orphan worktree empty task id"));
    }
    let attempt: u32 = attempt_s
        .parse()
        .map_err(|_| CoreError::invalid_input(format!("orphan worktree bad attempt: {name}")))?;
    validate_task_id(task_id)?;
    Ok((task_id.to_string(), attempt))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use dare_core::{MockProcessRunner, ProcessOutput, SafeCommand};

    use super::*;

    struct RecordingRunner {
        calls: Mutex<Vec<(String, Vec<String>)>>,
        inner: MockProcessRunner,
    }

    impl RecordingRunner {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
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

        fn take_calls(&self) -> Vec<(String, Vec<String>)> {
            std::mem::take(&mut *self.calls.lock().unwrap())
        }
    }

    impl ProcessRunner for RecordingRunner {
        fn run(&self, cmd: &SafeCommand) -> CoreResult<ProcessOutput> {
            self.calls
                .lock()
                .unwrap()
                .push((cmd.program().to_string(), cmd.arg_list().to_vec()));
            self.inner.run(cmd)
        }
    }

    fn root_with_git() -> (tempfile::TempDir, ProjectRoot) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".git")).unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        (dir, root)
    }

    #[test]
    fn create_argv_sequence() {
        let (_tmp, root) = root_with_git();
        let rec = Arc::new(RecordingRunner::new());
        rec.push_ok();
        let mgr = WorktreeManager::new(root, rec.clone());
        let spec = mgr.create("task-001", 2).unwrap();
        assert_eq!(spec.branch, "dare/agent-task-001-2");
        assert_eq!(spec.rel_path, ".dare/agent-worktrees/task-001-2");
        let calls = rec.take_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "git");
        assert_eq!(
            calls[0].1,
            vec![
                "worktree",
                "add",
                "-b",
                "dare/agent-task-001-2",
                ".dare/agent-worktrees/task-001-2",
                "HEAD",
            ]
        );
    }

    #[test]
    fn remove_argv_force() {
        let (_tmp, root) = root_with_git();
        let rec = Arc::new(RecordingRunner::new());
        rec.push_ok();
        let mgr = WorktreeManager::new(root, rec.clone());
        let spec = WorktreeSpec {
            task_id: "t1".into(),
            attempt: 1,
            branch: "dare/agent-t1-1".into(),
            rel_path: ".dare/agent-worktrees/t1-1".into(),
        };
        mgr.remove(&spec).unwrap();
        let calls = rec.take_calls();
        assert_eq!(
            calls[0].1,
            vec![
                "worktree",
                "remove",
                "--force",
                ".dare/agent-worktrees/t1-1"
            ]
        );
    }

    #[test]
    fn unsafe_id_err() {
        let (_tmp, root) = root_with_git();
        let rec = Arc::new(RecordingRunner::new());
        let mgr = WorktreeManager::new(root, rec);
        assert!(mgr.create("../x", 1).is_err());
        assert!(mgr.create("a/b", 1).is_err());
        assert!(mgr.create("", 1).is_err());
    }

    #[test]
    #[ignore = "requires real git on PATH"]
    fn real_git_create_remove() {
        let dir = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.email", "t@t"])
            .current_dir(dir.path())
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["config", "user.name", "t"])
            .current_dir(dir.path())
            .status()
            .unwrap();
        std::fs::write(dir.path().join("README"), "x").unwrap();
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .status()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(dir.path())
            .status()
            .unwrap();

        let root = ProjectRoot::new(dir.path()).unwrap();
        let mgr = WorktreeManager::new(root, Arc::new(dare_core::SystemProcessRunner));
        let spec = mgr.create("task-a", 1).unwrap();
        assert!(dir.path().join(&spec.rel_path).exists());
        mgr.remove(&spec).unwrap();
    }

    #[test]
    fn cleanup_lists_and_removes_orphan_dirs() {
        let (_tmp, root) = root_with_git();
        let base = root.as_path().join(AGENT_WORKTREES_REL);
        std::fs::create_dir_all(base.join("orphan-1")).unwrap();
        let rec = Arc::new(RecordingRunner::new());
        rec.push_ok();
        let mgr = WorktreeManager::new(root, rec);
        let n = mgr.cleanup_all().unwrap();
        assert_eq!(n, 1);
    }
}
