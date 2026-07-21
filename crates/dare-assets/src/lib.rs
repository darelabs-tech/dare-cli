//! Inventory, embed and verification of DARE assets (microplano 009).
//! Canonical capabilities (ADR-007) — microplano 010.

mod capability;
mod embed;
mod manifest;
mod materialize;
mod verify;

pub use capability::{
    load_capability_matrix_from_str, render_agent_skill, render_claude_command,
    validate_capability_matrix, Capability, CapabilityException, CapabilityMatrix, HarnessOutputs,
};
pub use embed::EmbeddedAssets;
pub use manifest::{
    assert_safe_asset_path, load_manifest_from_str, sha256_hex, AssetEntry, AssetKind,
    AssetsManifest,
};
pub use materialize::materialize_to;
pub use verify::verify_embedded_assets;

use dare_core::{validate_nonempty_name, CoreResult};

/// Smoke: layer ping.
pub fn assets_layer_ping(label: &str) -> CoreResult<&'static str> {
    validate_nonempty_name(label)?;
    Ok("assets-ok")
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::CoreError;

    #[test]
    fn ping_ok() {
        assert_eq!(assets_layer_ping("local"), Ok("assets-ok"));
    }

    #[test]
    fn ping_empty_err() {
        assert!(matches!(
            assets_layer_ping(""),
            Err(CoreError::InvalidInput(_))
        ));
    }

    #[test]
    fn verify_embedded_ok() {
        verify_embedded_assets().expect("embedded assets must verify");
    }

    #[test]
    fn manifest_hashes_match_assets_dir() {
        use crate::manifest::{load_manifest_from_str, sha256_hex, AssetKind};
        use std::fs;
        use std::path::PathBuf;

        let assets_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let manifest_path = assets_root.join("manifest.yml");
        let yaml = fs::read_to_string(&manifest_path).unwrap_or_else(|e| {
            panic!("read {}: {e}", manifest_path.display());
        });
        let manifest = load_manifest_from_str(&yaml).expect("parse manifest");
        for entry in &manifest.assets {
            if matches!(entry.kind, AssetKind::External) {
                continue;
            }
            let file_path = assets_root.join(&entry.path);
            let bytes = fs::read(&file_path).unwrap_or_else(|e| {
                panic!("read {}: {e}", file_path.display());
            });
            let got = sha256_hex(&bytes);
            assert_eq!(
                got, entry.sha256,
                "hash mismatch for {} ({})",
                entry.id, entry.path
            );
        }
    }
}
