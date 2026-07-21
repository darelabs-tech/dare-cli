//! Materialize embedded assets under a project-relative directory.

use dare_core::fs::atomic_write;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};

use crate::embed::EmbeddedAssets;
use crate::manifest::{assert_safe_asset_path, load_manifest_from_str, AssetKind};
use crate::verify::verify_embedded_assets;

/// Writes embedded canonical/generated assets under `dest_rel/` (e.g. `.dare/assets`).
/// Returns number of files written.
pub fn materialize_to(root: &ProjectRoot, dest_rel: &SafeRelativePath) -> CoreResult<usize> {
    verify_embedded_assets()?;
    let manifest_file = EmbeddedAssets::get("manifest.yml")
        .ok_or_else(|| CoreError::config("asset missing: manifest.yml"))?;
    let yaml = std::str::from_utf8(manifest_file.data.as_ref())
        .map_err(|e| CoreError::config(format!("invalid assets manifest encoding: {e}")))?;
    let manifest = load_manifest_from_str(yaml)?;

    let mut count = 0usize;
    let base = dest_rel.as_str().trim_end_matches('/');

    let man_rel = SafeRelativePath::new(&format!("{base}/manifest.yml"))?;
    atomic_write(root, &man_rel, manifest_file.data.as_ref())?;
    count += 1;

    for entry in &manifest.assets {
        assert_safe_asset_path(&entry.path)?;
        if matches!(entry.kind, AssetKind::External) {
            continue;
        }
        let Some(file) = EmbeddedAssets::get(&entry.path) else {
            return Err(CoreError::config(format!("asset missing: {}", entry.path)));
        };
        let rel = SafeRelativePath::new(&format!("{base}/{}", entry.path))?;
        atomic_write(root, &rel, file.data.as_ref())?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn materialize_writes_files() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let dest = SafeRelativePath::new(".dare/assets").unwrap();
        let n = materialize_to(&root, &dest).unwrap();
        assert!(n >= 7);
        assert!(dir.path().join(".dare/assets/manifest.yml").is_file());
        assert!(dir
            .path()
            .join(".dare/assets/templates/DESIGN-template.md")
            .is_file());
    }
}
