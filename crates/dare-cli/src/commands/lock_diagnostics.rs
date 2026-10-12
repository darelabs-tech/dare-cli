//! Read-only inspection of the canonical fs4 state lock.
//! A lock file left on disk does not prove an active holder or a stale lock.
use std::fs::OpenOptions;
use std::path::Path;

use fs4::{FileExt, TryLockError};
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LockDiagnostic {
    pub code: &'static str,
    pub state: &'static str,
    pub remediation: &'static str,
}

/// Probe only a pre-existing regular lock file. Never create, remove or truncate it.
pub fn state_lock_diagnostic(root: &Path) -> LockDiagnostic {
    let path = root.join(".dare/state.json.darelock");
    let unknown = || {
        LockDiagnostic {
            code: "DARE_STATE_LOCK_UNKNOWN",
            state: "unknown",
            remediation:
                "Inspect lock ownership and filesystem permissions manually; do not delete an unverified lock.",
        }
    };
    let meta = match std::fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return LockDiagnostic {
                code: "DARE_STATE_LOCK_ABSENT",
                state: "absent",
                remediation:
                    "No state lock file exists; absence does not validate state integrity.",
            };
        }
        Err(_) => return unknown(),
    };
    if !meta.file_type().is_file() {
        return unknown();
    }
    let file = match OpenOptions::new().read(true).write(true).open(&path) {
        Ok(file) => file,
        Err(_) => return unknown(),
    };
    match FileExt::try_lock(&file) {
        Ok(()) => {
            let _ = FileExt::unlock(&file);
            LockDiagnostic {
                code: "DARE_STATE_LOCK_UNHELD",
                state: "unheld",
                remediation:
                    "The lock is currently available. A residual file alone is not evidence of staleness.",
            }
        }
        Err(TryLockError::WouldBlock) => LockDiagnostic {
            code: "DARE_STATE_LOCK_HELD",
            state: "held",
            remediation:
                "Another process may hold the lock. Wait or inspect its owner; do not force-unlock.",
        },
        Err(TryLockError::Error(_)) => unknown(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn no_lock_does_not_create_directories_or_files() {
        let root = tempdir().unwrap();
        assert_eq!(state_lock_diagnostic(root.path()).state, "absent");
        assert!(!root.path().join(".dare").exists());
    }

    #[test]
    fn residual_lock_file_is_not_called_stale() {
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join(".dare")).unwrap();
        let path = root.path().join(".dare/state.json.darelock");
        std::fs::write(&path, b"").unwrap();
        assert_eq!(state_lock_diagnostic(root.path()).state, "unheld");
        assert!(path.exists());
    }

    #[test]
    fn contended_lock_is_held() {
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join(".dare")).unwrap();
        let path = root.path().join(".dare/state.json.darelock");
        let file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        FileExt::try_lock(&file).unwrap();
        assert_eq!(state_lock_diagnostic(root.path()).state, "held");
        FileExt::unlock(&file).unwrap();
    }

    #[test]
    fn nonregular_lock_is_unknown() {
        let root = tempdir().unwrap();
        std::fs::create_dir(root.path().join(".dare")).unwrap();
        std::fs::create_dir(root.path().join(".dare/state.json.darelock")).unwrap();
        assert_eq!(state_lock_diagnostic(root.path()).state, "unknown");
    }
}
