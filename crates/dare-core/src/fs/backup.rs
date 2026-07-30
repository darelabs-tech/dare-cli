//! Backup / restore under `.dare/backups/`.

use sha2::{Digest, Sha256};

use crate::error::{CoreError, CoreResult};
use crate::fs::atomic::{atomic_write, read_to_string};
use crate::path::{ProjectRoot, SafeRelativePath};

fn backup_rel_for(source_rel: &SafeRelativePath) -> CoreResult<SafeRelativePath> {
    let mut hasher = Sha256::new();
    hasher.update(source_rel.as_str().as_bytes());
    let hash = hasher.finalize();
    let sha8 = format!("{hash:x}")[..8].to_string();
    let ts = utc_stamp();
    let posix = source_rel.as_str();
    let path = format!(".dare/backups/{ts}-{sha8}/{posix}");
    SafeRelativePath::new(&path)
}

fn utc_stamp() -> String {
    // YYYYMMDDThhmmssZ without extra deps: use SystemTime
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // Approximate UTC stamp from epoch; good enough for uniqueness + ordering
    // Format: use chrono-less conversion
    const SECS_PER_DAY: u64 = 86400;
    let days = secs / SECS_PER_DAY;
    let rem = secs % SECS_PER_DAY;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;
    // Civil date from days since 1970-01-01 (Howard Hinnant algorithm)
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

pub fn backup(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<SafeRelativePath> {
    let content = read_to_string(root, rel)?;
    let bak = backup_rel_for(rel)?;
    atomic_write(root, &bak, content.as_bytes())?;
    Ok(bak)
}

pub fn restore(
    root: &ProjectRoot,
    backup_rel: &SafeRelativePath,
    dest_rel: &SafeRelativePath,
) -> CoreResult<()> {
    let content = read_to_string(root, backup_rel).map_err(|e| match e {
        CoreError::NotFound(_) => {
            CoreError::not_found(format!("backup not found: {}", backup_rel.as_str()))
        }
        other => other,
    })?;
    atomic_write(root, dest_rel, content.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::{ProjectRoot, SafeRelativePath};
    use tempfile::tempdir;

    #[test]
    fn backup_restore_roundtrip() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("data/file.txt").unwrap();
        atomic_write(&root, &rel, b"payload-42").unwrap();
        let bak = backup(&root, &rel).unwrap();
        assert!(bak.as_str().starts_with(".dare/backups/"));
        let dest = SafeRelativePath::new("data/restored.txt").unwrap();
        restore(&root, &bak, &dest).unwrap();
        assert_eq!(read_to_string(&root, &dest).unwrap(), "payload-42");
    }
}
