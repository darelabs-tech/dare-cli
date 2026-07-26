//! SHA-256 idempotency markers for hook runs (BLUEPRINT-048 §0.6).

use std::time::SystemTime;

use dare_core::fs::atomic_write;
use dare_core::{to_canonical_json_string, CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde_json::json;
use sha2::{Digest, Sha256};

pub const IDEMPOTENCY_DIR_REL: &str = ".dare/hooks-idempotency";
pub const IDEMPOTENCY_CAP: usize = 512;

/// Canonical digest key for `(event, action, file, task)`.
pub fn digest_key(
    event: &str,
    action: &str,
    file: Option<&str>,
    task: Option<&str>,
) -> CoreResult<String> {
    let value = json!({
        "schemaVersion": 1,
        "action": action,
        "event": event,
        "file": file,
        "task": task,
    });
    let canon = to_canonical_json_string(&value)?;
    let mut hasher = Sha256::new();
    hasher.update(canon.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

/// Relative path of the marker file for `hash`.
pub fn marker_rel(hash: &str) -> CoreResult<SafeRelativePath> {
    SafeRelativePath::new(&format!("{IDEMPOTENCY_DIR_REL}/{hash}.ok"))
}

/// Whether the idempotency marker exists under `root`.
pub fn marker_exists(root: &ProjectRoot, hash: &str) -> CoreResult<bool> {
    let rel = marker_rel(hash)?;
    let abs = root.resolve(&rel)?;
    Ok(abs.as_path().as_std_path().is_file())
}

/// Atomically write marker content `"ok\n"` (creates parent dirs).
pub fn write_marker(root: &ProjectRoot, hash: &str) -> CoreResult<()> {
    let rel = marker_rel(hash)?;
    atomic_write(root, &rel, b"ok\n")
}

/// If marker count exceeds [`IDEMPOTENCY_CAP`], delete oldest by mtime ASC until ≤ cap.
pub fn prune_if_needed(root: &ProjectRoot) -> CoreResult<()> {
    let dir_rel = SafeRelativePath::new(IDEMPOTENCY_DIR_REL)?;
    let abs = root.resolve(&dir_rel)?;
    let dir = abs.as_path().as_std_path();
    if !dir.is_dir() {
        return Ok(());
    }

    let mut entries: Vec<(SystemTime, std::path::PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| CoreError::io(e.to_string()))? {
        let entry = entry.map_err(|e| CoreError::io(e.to_string()))?;
        let path = entry.path();
        let is_ok = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e == "ok");
        if !is_ok {
            continue;
        }
        let meta = entry.metadata().map_err(|e| CoreError::io(e.to_string()))?;
        let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        entries.push((mtime, path));
    }

    if entries.len() <= IDEMPOTENCY_CAP {
        return Ok(());
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    let excess = entries.len() - IDEMPOTENCY_CAP;
    for (_, path) in entries.into_iter().take(excess) {
        std::fs::remove_file(&path).map_err(|e| CoreError::io(e.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn digest_stable() {
        let a = digest_key("on-save", "dare-validate", Some("src/main.rs"), None).unwrap();
        let b = digest_key("on-save", "dare-validate", Some("src/main.rs"), None).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(a.chars().all(|c| !c.is_ascii_uppercase()));

        let c = digest_key("on-save", "dare-validate", None, None).unwrap();
        assert_ne!(a, c);
    }

    #[test]
    fn prune_over_cap() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let idemp = dir.path().join(".dare").join("hooks-idempotency");
        std::fs::create_dir_all(&idemp).unwrap();

        for i in 0..(IDEMPOTENCY_CAP + 8) {
            let hash = format!("{i:064x}");
            let path = idemp.join(format!("{hash}.ok"));
            std::fs::write(&path, b"ok\n").unwrap();
        }

        let before: usize = std::fs::read_dir(&idemp)
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .ok()
                    .and_then(|e| e.path().extension().map(|x| x == "ok"))
                    .unwrap_or(false)
            })
            .count();
        assert!(before > IDEMPOTENCY_CAP);

        prune_if_needed(&root).unwrap();

        let after: usize = std::fs::read_dir(&idemp)
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .ok()
                    .and_then(|e| e.path().extension().map(|x| x == "ok"))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(after, IDEMPOTENCY_CAP);
    }
}
