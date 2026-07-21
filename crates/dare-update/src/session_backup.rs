//! Session backup root, file copy, and rollback (BLUEPRINT-022 §5.2–5.4).

use dare_core::fs::{atomic_write, restore};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};

use crate::APPLY_READ_CAP;

/// In-memory journal for one apply session (restore / delete on failure).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionJournal {
    pub backup_root: String,
    pub backed_up: Vec<(String, String)>,
    pub created: Vec<String>,
    pub created_dirs: Vec<String>,
}

fn sanitize_version_segment(cli_version: &str) -> String {
    let mut out = String::with_capacity(cli_version.len());
    for c in cli_version.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "unknown".to_string()
    } else {
        out
    }
}

fn utc_stamp() -> String {
    // Mirror dare-core::fs::backup style (no chrono).
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    const SECS_PER_DAY: u64 = 86400;
    let days = secs / SECS_PER_DAY;
    let rem = secs % SECS_PER_DAY;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;
    let (y, m, d) = civil_from_days(days as i64);
    format!("{y:04}{m:02}{d:02}T{hour:02}{min:02}{sec:02}Z")
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

/// Ensure a unique session backup directory under `.dare/`.
pub fn ensure_backup_root(root: &ProjectRoot, cli_version: &str) -> CoreResult<SafeRelativePath> {
    let sanitized = sanitize_version_segment(cli_version);
    let candidate = SafeRelativePath::new(&format!(".dare/backup-{sanitized}"))?;
    let abs = root.resolve(&candidate)?;
    if !abs.as_path().exists() {
        std::fs::create_dir_all(abs.as_path().as_std_path())
            .map_err(|e| CoreError::io(e.to_string()))?;
        return Ok(candidate);
    }

    let stamp = utc_stamp();
    let candidate2 = SafeRelativePath::new(&format!(".dare/backup-{sanitized}-{stamp}"))?;
    let abs2 = root.resolve(&candidate2)?;
    std::fs::create_dir_all(abs2.as_path().as_std_path())
        .map_err(|e| CoreError::io(e.to_string()))?;
    Ok(candidate2)
}

fn read_apply_capped(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<Vec<u8>> {
    let abs = root.resolve(rel)?;
    let meta = std::fs::metadata(abs.as_path().as_std_path()).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CoreError::not_found(format!("file not found: {}", rel.as_str()))
        } else {
            CoreError::io(e.to_string())
        }
    })?;
    if meta.len() > APPLY_READ_CAP as u64 {
        return Err(CoreError::invalid_input(format!(
            "file exceeds apply read cap ({} bytes): {}",
            APPLY_READ_CAP,
            rel.as_str()
        )));
    }
    std::fs::read(abs.as_path().as_std_path()).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CoreError::not_found(format!("file not found: {}", rel.as_str()))
        } else {
            CoreError::io(e.to_string())
        }
    })
}

/// Create missing parent directories for `file_rel`, journaling each new dir.
pub fn ensure_parent_dirs(
    root: &ProjectRoot,
    file_rel: &SafeRelativePath,
    journal: &mut SessionJournal,
) -> CoreResult<()> {
    let posix = file_rel.as_str();
    let Some((parent, _)) = posix.rsplit_once('/') else {
        return Ok(());
    };
    let mut acc = String::new();
    for part in parent.split('/') {
        if !acc.is_empty() {
            acc.push('/');
        }
        acc.push_str(part);
        let rel = SafeRelativePath::new(&acc)?;
        let abs = root.resolve(&rel)?;
        if abs.as_path().is_dir() {
            continue;
        }
        std::fs::create_dir_all(abs.as_path().as_std_path())
            .map_err(|e| CoreError::io(e.to_string()))?;
        journal.created_dirs.push(acc.clone());
    }
    Ok(())
}

/// Copy an existing dest file into the session backup tree and journal it.
pub fn session_backup_file(
    root: &ProjectRoot,
    backup_root: &SafeRelativePath,
    dest_rel: &str,
    journal: &mut SessionJournal,
) -> CoreResult<()> {
    let dest = SafeRelativePath::new(dest_rel)?;
    let bytes = read_apply_capped(root, &dest)?;
    let bak = SafeRelativePath::new(&format!("{}/{}", backup_root.as_str(), dest_rel))?;
    ensure_parent_dirs(root, &bak, journal)?;
    atomic_write(root, &bak, &bytes)?;
    journal
        .backed_up
        .push((dest_rel.to_string(), bak.as_str().to_string()));
    Ok(())
}

/// Restore backed-up files, delete created files, remove empty created dirs.
/// Does **not** delete the session `backup_root` directory.
pub fn rollback_session(root: &ProjectRoot, journal: &SessionJournal) -> CoreResult<()> {
    for (dest, bak) in journal.backed_up.iter().rev() {
        let dest_rel = SafeRelativePath::new(dest)
            .map_err(|e| CoreError::internal(format!("rollback invalid dest path {dest}: {e}")))?;
        let bak_rel = SafeRelativePath::new(bak)
            .map_err(|e| CoreError::internal(format!("rollback invalid backup path {bak}: {e}")))?;
        restore(root, &bak_rel, &dest_rel)
            .map_err(|e| CoreError::internal(format!("rollback restore failed for {dest}: {e}")))?;
    }

    for rel in journal.created.iter().rev() {
        let p = match SafeRelativePath::new(rel) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let abs = match root.resolve(&p) {
            Ok(a) => a,
            Err(_) => continue,
        };
        if abs.as_path().is_file() {
            match std::fs::remove_file(abs.as_path().as_std_path()) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => {}
            }
        }
    }

    for rel in journal.created_dirs.iter().rev() {
        let p = match SafeRelativePath::new(rel) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let abs = match root.resolve(&p) {
            Ok(a) => a,
            Err(_) => continue,
        };
        if abs.as_path().is_dir() {
            let _ = std::fs::remove_dir(abs.as_path().as_std_path());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ErrorKind;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn ensure_backup_root_creates_and_collides() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let first = ensure_backup_root(&root, "0.1.0-alpha.0").unwrap();
        assert_eq!(first.as_str(), ".dare/backup-0.1.0-alpha.0");
        assert!(root.resolve(&first).unwrap().as_path().is_dir());

        let second = ensure_backup_root(&root, "0.1.0-alpha.0").unwrap();
        assert!(
            second.as_str().starts_with(".dare/backup-0.1.0-alpha.0-"),
            "expected utc suffix, got {}",
            second.as_str()
        );
        assert_ne!(first.as_str(), second.as_str());
        assert!(root.resolve(&second).unwrap().as_path().is_dir());
    }

    #[test]
    fn session_backup_and_rollback_restores() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let backup_root = ensure_backup_root(&root, "1.2.3").unwrap();

        let dest = SafeRelativePath::new("nested/asset.txt").unwrap();
        atomic_write(&root, &dest, b"original-v1").unwrap();

        let mut journal = SessionJournal {
            backup_root: backup_root.as_str().to_string(),
            ..Default::default()
        };
        session_backup_file(&root, &backup_root, "nested/asset.txt", &mut journal).unwrap();
        assert_eq!(journal.backed_up.len(), 1);

        // Simulate a bad replace, then rollback
        atomic_write(&root, &dest, b"corrupted").unwrap();
        assert_eq!(
            fs::read(dir.path().join("nested/asset.txt")).unwrap(),
            b"corrupted"
        );

        // Also track a created file + nested dir under project for rollback
        let created = SafeRelativePath::new("brand/new.txt").unwrap();
        atomic_write(&root, &created, b"new").unwrap();
        journal.created.push("brand/new.txt".into());
        journal.created_dirs.push("brand".into());

        rollback_session(&root, &journal).unwrap();

        assert_eq!(
            fs::read(dir.path().join("nested/asset.txt")).unwrap(),
            b"original-v1"
        );
        assert!(!dir.path().join("brand/new.txt").exists());
        // backup root must remain
        assert!(root.resolve(&backup_root).unwrap().as_path().is_dir());
        let bak_rel =
            SafeRelativePath::new(&format!("{}/nested/asset.txt", backup_root.as_str())).unwrap();
        assert!(root.resolve(&bak_rel).unwrap().as_path().is_file());
    }

    #[test]
    fn read_cap_rejects_huge_backup_source() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let backup_root = ensure_backup_root(&root, "9.9.9").unwrap();

        let huge = vec![b'x'; APPLY_READ_CAP + 1];
        let dest = SafeRelativePath::new("huge.bin").unwrap();
        atomic_write(&root, &dest, &huge).unwrap();

        let mut journal = SessionJournal::default();
        let err = session_backup_file(&root, &backup_root, "huge.bin", &mut journal).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(journal.backed_up.is_empty());
    }

    #[test]
    fn sanitize_empty_and_special_chars() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let path = ensure_backup_root(&root, "!!!").unwrap();
        assert_eq!(path.as_str(), ".dare/backup-___");
        let empty = ensure_backup_root(&root, "").unwrap();
        assert_eq!(empty.as_str(), ".dare/backup-unknown");
    }
}
