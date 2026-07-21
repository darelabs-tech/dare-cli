//! Asset manifest types + SHA-256.

use dare_core::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Canonical,
    Generated,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetEntry {
    pub id: String,
    pub path: String,
    pub sha256: String,
    pub kind: AssetKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetsManifest {
    pub version: u32,
    pub assets: Vec<AssetEntry>,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Reject absolute paths, `..` segments, and backslashes (RS-01 / Blueprint T-05).
pub fn assert_safe_asset_path(path: &str) -> CoreResult<()> {
    if path.is_empty() {
        return Err(CoreError::config("invalid asset path: empty"));
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return Err(CoreError::config(format!(
            "invalid asset path (absolute): {path}"
        )));
    }
    if path.contains('\\') {
        return Err(CoreError::config(format!(
            "invalid asset path (backslash): {path}"
        )));
    }
    for seg in path.split('/') {
        if seg == ".." {
            return Err(CoreError::config(format!(
                "invalid asset path (dot-dot): {path}"
            )));
        }
    }
    Ok(())
}

pub fn load_manifest_from_str(yaml: &str) -> CoreResult<AssetsManifest> {
    let m: AssetsManifest =
        serde_yaml::from_str(yaml).map_err(|e| CoreError::config(format!("invalid assets manifest: {e}")))?;
    if m.version != 1 {
        return Err(CoreError::config(format!(
            "unsupported assets manifest version: {}",
            m.version
        )));
    }
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_manifest() {
        let yaml = r#"
version: 1
assets:
  - id: a
    path: templates/x.md
    sha256: abc
    kind: canonical
"#;
        let m = load_manifest_from_str(yaml).unwrap();
        assert_eq!(m.assets.len(), 1);
        assert_eq!(m.assets[0].kind, AssetKind::Canonical);
    }

    #[test]
    fn rejects_dotdot_path() {
        let err = assert_safe_asset_path("templates/../etc/passwd").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("..") || msg.contains("dot-dot"), "{msg}");
        assert!(msg.contains("templates/../etc/passwd"), "{msg}");
    }

    #[test]
    fn rejects_backslash_path() {
        let err = assert_safe_asset_path(r"foo\bar").unwrap_err();
        assert!(err.to_string().contains(r"foo\bar"));
    }

    #[test]
    fn accepts_posix_relative() {
        assert!(assert_safe_asset_path("templates/DESIGN-template.md").is_ok());
    }
}
