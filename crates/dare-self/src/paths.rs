//! Self-update state home (`~/.dare/self` or `DARE_SELF_HOME`).

use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Env override for the self-update state directory.
pub const ENV_SELF_HOME: &str = "DARE_SELF_HOME";

/// Exclusive lock file name under [`SelfHome`].
pub const LOCK_NAME: &str = "update.lock";

/// Backup directory name under [`SelfHome`].
pub const BACKUP_DIR_NAME: &str = "backup";

/// Scratch directory name under [`SelfHome`].
pub const TMP_DIR_NAME: &str = "tmp";

/// Errors resolving or preparing [`SelfHome`].
#[derive(Debug, Error)]
pub enum PathsError {
    #[error("cannot resolve user home directory")]
    HomeUnresolved,
    #[error("io error: {0}")]
    Io(String),
}

/// Layout root for self-update state.
///
/// ```text
/// $DARE_SELF_HOME/   # default: {home}/.dare/self
///   update.lock
///   backup/
///   tmp/
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelfHome {
    root: PathBuf,
}

impl SelfHome {
    /// Resolve from `DARE_SELF_HOME`, or `{user_home}/.dare/self`, and ensure layout dirs.
    pub fn resolve() -> Result<Self, PathsError> {
        let root = match std::env::var_os(ENV_SELF_HOME) {
            Some(v) if !v.is_empty() => PathBuf::from(v),
            _ => user_home()?.join(".dare").join("self"),
        };
        Self::from_path(root)
    }

    /// Use an explicit root (tests / callers that already chose a path). Ensures `backup/` and `tmp/`.
    pub fn from_path(root: impl Into<PathBuf>) -> Result<Self, PathsError> {
        let home = Self { root: root.into() };
        home.ensure_dirs()?;
        Ok(home)
    }

    /// Absolute root of this self-home.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Path to `update.lock`.
    pub fn lock_path(&self) -> PathBuf {
        self.root.join(LOCK_NAME)
    }

    /// Path to `backup/`.
    pub fn backup_dir(&self) -> PathBuf {
        self.root.join(BACKUP_DIR_NAME)
    }

    /// Path to `tmp/`.
    pub fn tmp_dir(&self) -> PathBuf {
        self.root.join(TMP_DIR_NAME)
    }

    /// Ensure `backup/` and `tmp/` exist under the root.
    pub fn ensure_dirs(&self) -> Result<(), PathsError> {
        fs::create_dir_all(self.backup_dir()).map_err(|e| PathsError::Io(e.to_string()))?;
        fs::create_dir_all(self.tmp_dir()).map_err(|e| PathsError::Io(e.to_string()))?;
        Ok(())
    }
}

fn user_home() -> Result<PathBuf, PathsError> {
    if let Some(h) = std::env::var_os("HOME").filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(h));
    }
    if let Some(h) = std::env::var_os("USERPROFILE").filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(h));
    }
    Err(PathsError::HomeUnresolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn from_path_creates_backup_and_tmp() {
        let dir = tempdir().unwrap();
        let home = SelfHome::from_path(dir.path().join("self")).unwrap();
        assert!(home.backup_dir().is_dir());
        assert!(home.tmp_dir().is_dir());
        assert_eq!(home.lock_path(), home.root().join("update.lock"));
    }
}
