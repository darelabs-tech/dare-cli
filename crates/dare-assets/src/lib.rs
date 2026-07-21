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
pub use manifest::{load_manifest_from_str, sha256_hex, AssetEntry, AssetKind, AssetsManifest};
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
}
