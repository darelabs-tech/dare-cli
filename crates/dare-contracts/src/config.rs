//! `dare.config.json` contract.

use dare_core::CoreResult;
use dare_core::{ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::io::{from_json_slice, read_limited, write_json_atomic};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ConfigObject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DareConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ide: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<ConfigObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<ConfigObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard: Option<ConfigObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph: Option<ConfigObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<ConfigObject>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub fn load_dare_config(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<DareConfig> {
    let bytes = read_limited(root, rel)?;
    from_json_slice(&bytes)
}

pub fn save_dare_config(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    cfg: &DareConfig,
) -> CoreResult<()> {
    write_json_atomic(root, rel, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn config_preserves_unknown_root_keys() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("dare.config.json").unwrap();
        let raw = r#"{"ide":"cursor","customExtension":{"x":1}}"#;
        std::fs::write(dir.path().join("dare.config.json"), raw).unwrap();
        let cfg = load_dare_config(&root, &rel).unwrap();
        assert_eq!(cfg.ide.as_deref(), Some("cursor"));
        assert!(cfg.extra.contains_key("customExtension"));
        save_dare_config(&root, &rel, &cfg).unwrap();
        let cfg2 = load_dare_config(&root, &rel).unwrap();
        assert_eq!(cfg2.extra.get("customExtension"), cfg.extra.get("customExtension"));
    }

    #[test]
    fn config_preserves_nested_block_extras() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("dare.config.json").unwrap();
        let raw = r#"{"guard":{"enabled":true,"customRule":"a"}}"#;
        std::fs::write(dir.path().join("dare.config.json"), raw).unwrap();
        let cfg = load_dare_config(&root, &rel).unwrap();
        let g = cfg.guard.as_ref().unwrap();
        assert_eq!(g.enabled, Some(true));
        assert_eq!(g.extra.get("customRule").and_then(|v| v.as_str()), Some("a"));
    }
}
