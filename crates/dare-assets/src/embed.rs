//! rust-embed wrapper for repo `assets/` folder.

use rust_embed::Embed;

/// Embedded files from workspace `assets/` (relative to this crate).
#[derive(Embed)]
#[folder = "../../assets"]
pub struct EmbeddedAssets;
