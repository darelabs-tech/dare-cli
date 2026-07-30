//! Minimal uninstall: remove only the target binary (no `--purge` in v1).

use std::fs;
use std::path::PathBuf;

use dare_core::{CoreError, CoreResult};

/// Options for [`uninstall`].
///
/// Production callers leave [`Self::target`] as `None` (uses `current_exe`).
/// Tests inject a fixture temp binary path.
#[derive(Debug, Clone, Default)]
pub struct UninstallOpts {
    /// Binary path to delete; `None` → `std::env::current_exe()`.
    pub target: Option<PathBuf>,
}

/// Report produced by a successful [`uninstall`] (schemaVersion 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallReport {
    pub schema_version: u32,
    pub ok: bool,
    pub mode: String,
    pub removed_path: PathBuf,
    /// Optional version label (CLI may fill later).
    pub version: Option<String>,
}

/// Delete **only** the target binary path.
///
/// Does **not** touch [`crate::paths::SelfHome`], user project trees, or implement `--purge`.
pub fn uninstall(opts: UninstallOpts) -> CoreResult<UninstallReport> {
    let target = match opts.target {
        Some(p) => p,
        None => std::env::current_exe()
            .map_err(|e| CoreError::not_found(format!("cannot resolve current_exe: {e}")))?,
    };

    if !target.is_file() {
        return Err(CoreError::not_found(format!(
            "uninstall target is not a file: {}",
            target.display()
        )));
    }

    fs::remove_file(&target).map_err(|e| {
        CoreError::io(format!(
            "failed to remove binary {}: {e}",
            target.display()
        ))
    })?;

    Ok(UninstallReport {
        schema_version: 1,
        ok: true,
        mode: "uninstall".to_string(),
        removed_path: target,
        version: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::{backup_binary_name, SelfHome};
    use tempfile::tempdir;

    #[test]
    fn uninstall_removes_only_exe() {
        let dir = tempdir().unwrap();
        let home = SelfHome::from_path(dir.path().join("self")).unwrap();

        // Marker under SelfHome/backup — must survive uninstall.
        let marker = home.backup_dir().join("keep-me.txt");
        fs::write(&marker, b"preserve").unwrap();

        let exe = dir.path().join("install").join(backup_binary_name());
        fs::create_dir_all(exe.parent().unwrap()).unwrap();
        fs::write(&exe, b"DARE-BINARY").unwrap();

        let report = uninstall(UninstallOpts {
            target: Some(exe.clone()),
        })
        .unwrap();

        assert!(report.ok);
        assert_eq!(report.mode, "uninstall");
        assert_eq!(report.schema_version, 1);
        assert_eq!(report.removed_path, exe);
        assert!(!exe.exists(), "target binary must be removed");
        assert!(
            home.backup_dir().is_dir(),
            "SelfHome backup dir must still exist"
        );
        assert!(
            home.root().is_dir(),
            "SelfHome root must still exist"
        );
        assert!(
            marker.is_file(),
            "SelfHome contents must not be purged"
        );
        assert_eq!(fs::read(&marker).unwrap(), b"preserve");
    }
}
