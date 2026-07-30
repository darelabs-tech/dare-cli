//! Restore the previous binary from `SelfHome/backup`.

use std::path::PathBuf;

use dare_core::{CoreError, CoreResult};

use crate::apply::atomic_replace;
use crate::lock::{acquire_lock, force_unlock_if_stale, LockHeld};
use crate::paths::SelfHome;

/// Frozen message when no rollback backup exists (BLUEPRINT-053).
pub const MSG_NO_BACKUP: &str = "no rollback backup found";

/// Options for [`rollback`].
///
/// Production callers leave [`Self::home`] / [`Self::current_exe`] as `None`.
/// Tests inject a fixture home and target binary path.
#[derive(Debug, Clone, Default)]
pub struct RollbackOpts {
    /// Override [`SelfHome`] (tests); `None` → [`SelfHome::resolve`].
    pub home: Option<SelfHome>,
    /// Binary to overwrite; `None` → `std::env::current_exe()`.
    pub current_exe: Option<PathBuf>,
    /// Attempt stale lock removal before acquire (same as apply `--force-unlock`).
    pub force_unlock: bool,
}

/// Report produced by a successful [`rollback`] (schemaVersion 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackReport {
    pub schema_version: u32,
    pub ok: bool,
    pub mode: String,
    pub backup_path: PathBuf,
    pub restored_path: PathBuf,
    /// Optional prior / restored version label (CLI may fill later).
    pub version: Option<String>,
}

/// Restore `backup/dare[.exe]` over the current binary under `update.lock`.
pub fn rollback(opts: RollbackOpts) -> CoreResult<RollbackReport> {
    let home = match opts.home {
        Some(h) => h,
        None => SelfHome::resolve().map_err(|e| CoreError::io(e.to_string()))?,
    };
    let current_exe = match opts.current_exe {
        Some(p) => p,
        None => std::env::current_exe()
            .map_err(|e| CoreError::not_found(format!("cannot resolve current_exe: {e}")))?,
    };

    if !current_exe.is_file() {
        return Err(CoreError::not_found(format!(
            "current_exe is not a file: {}",
            current_exe.display()
        )));
    }

    home.ensure_dirs()
        .map_err(|e| CoreError::io(e.to_string()))?;

    if opts.force_unlock {
        let _ = force_unlock_if_stale(&home);
    }

    let _lock = acquire_lock(&home).map_err(|e: LockHeld| {
        if e.message() == crate::lock::MSG_LOCK_HELD {
            CoreError::invalid_input(e.message())
        } else {
            CoreError::io(e.message())
        }
    })?;

    let backup_path = home.backup_binary_path();
    if !backup_path.is_file() {
        return Err(CoreError::invalid_input(MSG_NO_BACKUP));
    }

    atomic_replace(&backup_path, &current_exe)?;

    Ok(RollbackReport {
        schema_version: 1,
        ok: true,
        mode: "rollback".to_string(),
        backup_path,
        restored_path: current_exe,
        version: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::backup_binary_name;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn rollback_restores_backup() {
        let dir = tempdir().unwrap();
        let home = SelfHome::from_path(dir.path().join("self")).unwrap();

        let current = dir.path().join("bin").join(backup_binary_name());
        fs::create_dir_all(current.parent().unwrap()).unwrap();
        fs::write(&current, b"NEW-BROKEN").unwrap();

        let backup = home.backup_binary_path();
        fs::write(&backup, b"OLD-GOOD").unwrap();

        let report = rollback(RollbackOpts {
            home: Some(home.clone()),
            current_exe: Some(current.clone()),
            force_unlock: false,
        })
        .unwrap();

        assert!(report.ok);
        assert_eq!(report.mode, "rollback");
        assert_eq!(report.schema_version, 1);
        assert_eq!(fs::read(&current).unwrap(), b"OLD-GOOD");
        assert!(backup.is_file());
        assert_eq!(report.restored_path, current);
        assert_eq!(report.backup_path, backup);
    }

    #[test]
    fn rollback_missing_backup_err() {
        let dir = tempdir().unwrap();
        let home = SelfHome::from_path(dir.path().join("self")).unwrap();

        let current = dir.path().join("bin").join(backup_binary_name());
        fs::create_dir_all(current.parent().unwrap()).unwrap();
        fs::write(&current, b"CURRENT").unwrap();

        assert!(!home.backup_binary_path().exists());

        let err = rollback(RollbackOpts {
            home: Some(home),
            current_exe: Some(current),
            force_unlock: false,
        })
        .unwrap_err();

        assert_eq!(err.message(), MSG_NO_BACKUP);
    }
}
