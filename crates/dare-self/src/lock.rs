//! Exclusive self-update lock on `SelfHome/update.lock` via fs4.

use std::fs::{File, OpenOptions};
use std::time::{Duration, SystemTime};

use fs4::{FileExt, TryLockError};
use thiserror::Error;

use crate::paths::SelfHome;

/// Frozen message when another process holds the lock (BLUEPRINT-053).
pub const MSG_LOCK_HELD: &str = "self-update lock is held by another process";

/// Age (seconds) after which [`force_unlock_if_stale`] may remove `update.lock`.
pub const STALE_LOCK_SECS: u64 = 3600;

/// Contended (or otherwise failed) acquire of the self-update lock.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub struct LockHeld(String);

impl LockHeld {
    fn contended() -> Self {
        Self(MSG_LOCK_HELD.to_string())
    }

    fn io(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }

    /// Human-readable message (includes lock-held semantics for contention).
    pub fn message(&self) -> &str {
        &self.0
    }
}

/// RAII guard: releases the exclusive fs4 lock on drop.
#[derive(Debug)]
pub struct LockGuard {
    file: File,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

/// Acquire an exclusive lock on `{home}/update.lock`.
pub fn acquire_lock(home: &SelfHome) -> Result<LockGuard, LockHeld> {
    home.ensure_dirs()
        .map_err(|e| LockHeld::io(e.to_string()))?;

    let path = home.lock_path();
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .map_err(|e| LockHeld::io(e.to_string()))?;

    FileExt::try_lock(&file).map_err(|e| match e {
        TryLockError::WouldBlock => LockHeld::contended(),
        TryLockError::Error(err) => LockHeld::io(err.to_string()),
    })?;

    // Refresh mtime so stale detection tracks the active holder.
    let _ = file.set_len(file.metadata().map(|m| m.len()).unwrap_or(0));
    let now = filetime_now();
    let _ = file.set_modified(now);

    Ok(LockGuard { file })
}

/// If `update.lock` exists and its mtime is older than [`STALE_LOCK_SECS`], remove it.
///
/// Returns `Ok(true)` when the file was removed, `Ok(false)` when no lock file exists.
/// Returns `Err` when the lock file exists but is not stale enough to force-unlock.
pub fn force_unlock_if_stale(home: &SelfHome) -> Result<bool, ForceUnlockError> {
    let path = home.lock_path();
    if !path.exists() {
        return Ok(false);
    }

    let meta = std::fs::metadata(&path).map_err(|e| ForceUnlockError::Io(e.to_string()))?;
    let modified = meta
        .modified()
        .map_err(|e| ForceUnlockError::Io(e.to_string()))?;
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::ZERO);

    if age.as_secs() < STALE_LOCK_SECS {
        return Err(ForceUnlockError::NotStale {
            age_secs: age.as_secs(),
        });
    }

    std::fs::remove_file(&path).map_err(|e| ForceUnlockError::Io(e.to_string()))?;
    Ok(true)
}

/// Errors from [`force_unlock_if_stale`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ForceUnlockError {
    #[error("self-update lock is not stale (age {age_secs}s < {STALE_LOCK_SECS}s)")]
    NotStale { age_secs: u64 },
    #[error("io error: {0}")]
    Io(String),
}

fn filetime_now() -> SystemTime {
    SystemTime::now()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::SelfHome;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn lock_contention_second_fails() {
        let dir = tempdir().unwrap();
        let home = SelfHome::from_path(dir.path().join("self")).unwrap();

        let first = acquire_lock(&home).expect("first acquire");
        let second = acquire_lock(&home);
        assert!(second.is_err(), "second lock should fail while first is held");
        let err = second.unwrap_err();
        assert!(
            err.message().contains("lock") && err.message().contains("held"),
            "message should describe lock held: {}",
            err.message()
        );
        assert_eq!(err.message(), MSG_LOCK_HELD);

        drop(first);
        let third = acquire_lock(&home);
        assert!(third.is_ok(), "lock after drop should succeed");
    }

    #[test]
    fn force_unlock_refuses_fresh_lock_file() {
        let dir = tempdir().unwrap();
        let home = SelfHome::from_path(dir.path().join("self")).unwrap();
        let _guard = acquire_lock(&home).unwrap();
        // Drop guard so we can remove the file if allowed; mtime is fresh.
        drop(_guard);
        let err = force_unlock_if_stale(&home).unwrap_err();
        assert!(matches!(err, ForceUnlockError::NotStale { .. }));
    }

    #[test]
    fn force_unlock_removes_stale_lock_file() {
        let dir = tempdir().unwrap();
        let home = SelfHome::from_path(dir.path().join("self")).unwrap();
        let path = home.lock_path();
        std::fs::write(&path, b"").unwrap();
        let stale = SystemTime::now() - Duration::from_secs(STALE_LOCK_SECS + 10);
        filetime_set_mtime(&path, stale).unwrap();

        assert!(force_unlock_if_stale(&home).unwrap());
        assert!(!path.exists());
    }

    fn filetime_set_mtime(path: &std::path::Path, mtime: SystemTime) -> std::io::Result<()> {
        let file = OpenOptions::new().write(true).open(path)?;
        file.set_modified(mtime)?;
        Ok(())
    }
}
