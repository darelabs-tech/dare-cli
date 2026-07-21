//! Verify embedded bytes against `assets/manifest.yml`.

use dare_core::{CoreError, CoreResult};

use crate::embed::EmbeddedAssets;
use crate::manifest::{assert_safe_asset_path, load_manifest_from_str, sha256_hex, AssetKind};

pub fn verify_embedded_assets() -> CoreResult<()> {
    let manifest_file = EmbeddedAssets::get("manifest.yml")
        .ok_or_else(|| CoreError::config("asset missing: manifest.yml"))?;
    let yaml = std::str::from_utf8(manifest_file.data.as_ref())
        .map_err(|e| CoreError::config(format!("invalid assets manifest encoding: {e}")))?;
    let manifest = load_manifest_from_str(yaml)?;

    for entry in &manifest.assets {
        assert_safe_asset_path(&entry.path)?;
        if matches!(entry.kind, AssetKind::External) {
            continue;
        }
        let file = EmbeddedAssets::get(&entry.path)
            .ok_or_else(|| CoreError::config(format!("asset missing: {}", entry.path)))?;
        let got = sha256_hex(file.data.as_ref());
        if got != entry.sha256 {
            return Err(CoreError::config(format!(
                "asset hash mismatch: {}",
                entry.path
            )));
        }
    }
    Ok(())
}
