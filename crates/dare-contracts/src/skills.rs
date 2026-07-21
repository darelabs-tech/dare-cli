//! `.dare/skills.yml`.

use dare_core::CoreResult;
use dare_core::{ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::io::{from_yaml_str, read_limited, write_yaml_atomic};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillEntry {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SkillsManifest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub skills: Vec<SkillEntry>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub fn load_skills_manifest(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
) -> CoreResult<SkillsManifest> {
    let bytes = read_limited(root, rel)?;
    let text = String::from_utf8(bytes).map_err(|e| dare_core::CoreError::config(e.to_string()))?;
    from_yaml_str(&text)
}

pub fn save_skills_manifest(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    manifest: &SkillsManifest,
) -> CoreResult<()> {
    write_yaml_atomic(root, rel, manifest)
}
