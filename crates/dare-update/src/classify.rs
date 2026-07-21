//! Asset path classification for `dare update` (SHA-256 + managed marker).

use dare_assets::sha256_hex;
use dare_contracts::read_limited;
use dare_core::{CoreResult, ProjectRoot, SafeRelativePath};
use dare_harness::content_is_managed;
use serde::{Deserialize, Serialize};

/// Per-asset status in an update plan (JSON lowercase).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetUpdateStatus {
    Identical,
    Missing,
    Apply,
    Customized,
}

/// Classify a project-relative path against an expected SHA-256.
pub fn classify_path(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    expected_sha256: &str,
) -> CoreResult<AssetUpdateStatus> {
    Ok(classify_path_detailed(root, rel, expected_sha256)?.0)
}

/// Like [`classify_path`], but also returns the actual SHA when the file exists.
pub(crate) fn classify_path_detailed(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    expected_sha256: &str,
) -> CoreResult<(AssetUpdateStatus, Option<String>)> {
    let resolved = root.resolve(rel)?;
    if !resolved.as_path().is_file() {
        return Ok((AssetUpdateStatus::Missing, None));
    }

    let bytes = read_limited(root, rel)?;
    let actual = sha256_hex(&bytes);

    if actual == expected_sha256 {
        return Ok((AssetUpdateStatus::Identical, Some(actual)));
    }
    if content_is_managed(&bytes) {
        return Ok((AssetUpdateStatus::Apply, Some(actual)));
    }
    Ok((AssetUpdateStatus::Customized, Some(actual)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_assets::sha256_hex;
    use dare_core::fs::atomic_write;
    use tempfile::tempdir;

    fn write_file(root: &ProjectRoot, rel: &str, bytes: &[u8]) {
        let path = SafeRelativePath::new(rel).unwrap();
        atomic_write(root, &path, bytes).unwrap();
    }

    #[test]
    fn classify_missing() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("AGENTS.md").unwrap();
        let status = classify_path(&root, &rel, &"a".repeat(64)).unwrap();
        assert_eq!(status, AssetUpdateStatus::Missing);
    }

    #[test]
    fn classify_identical() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let body = b"<!-- dare:managed -->\nidentical body\n";
        write_file(&root, "keep.md", body);
        let expected = sha256_hex(body);
        let rel = SafeRelativePath::new("keep.md").unwrap();
        let status = classify_path(&root, &rel, &expected).unwrap();
        assert_eq!(status, AssetUpdateStatus::Identical);
    }

    #[test]
    fn classify_apply_managed() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let body = b"<!-- dare:managed -->\nstale managed content\n";
        write_file(&root, "managed.md", body);
        let other = sha256_hex(b"different expected bytes");
        let rel = SafeRelativePath::new("managed.md").unwrap();
        let status = classify_path(&root, &rel, &other).unwrap();
        assert_eq!(status, AssetUpdateStatus::Apply);
    }

    #[test]
    fn classify_customized_unmanaged() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let body = b"# My custom notes\nnot managed\n";
        write_file(&root, "custom.md", body);
        let other = sha256_hex(b"different expected bytes");
        let rel = SafeRelativePath::new("custom.md").unwrap();
        let status = classify_path(&root, &rel, &other).unwrap();
        assert_eq!(status, AssetUpdateStatus::Customized);
    }

    #[test]
    fn asset_update_status_serde_roundtrip_lowercase() {
        for (status, expected) in [
            (AssetUpdateStatus::Identical, "\"identical\""),
            (AssetUpdateStatus::Missing, "\"missing\""),
            (AssetUpdateStatus::Apply, "\"apply\""),
            (AssetUpdateStatus::Customized, "\"customized\""),
        ] {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, expected);
            let back: AssetUpdateStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, status);
        }
    }
}
