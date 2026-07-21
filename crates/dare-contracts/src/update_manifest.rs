//! `templates/UPDATE-MANIFEST.json` schemaVersion 1.

use dare_core::{CoreError, CoreResult};
use dare_core::{ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::io::{from_json_slice, read_limited, write_json_atomic};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManifestV1 {
    pub schema_version: u32,
    #[serde(default)]
    pub releases: Vec<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub fn load_update_manifest(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
) -> CoreResult<UpdateManifestV1> {
    let bytes = read_limited(root, rel)?;
    let m: UpdateManifestV1 = from_json_slice(&bytes)?;
    if m.schema_version != 1 {
        return Err(CoreError::config(
            "unsupported update manifest schemaVersion",
        ));
    }
    Ok(m)
}

pub fn save_update_manifest(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    manifest: &UpdateManifestV1,
) -> CoreResult<()> {
    if manifest.schema_version != 1 {
        return Err(CoreError::config(
            "unsupported update manifest schemaVersion",
        ));
    }
    write_json_atomic(root, rel, manifest)
}

pub fn update_manifest_from_str(s: &str) -> CoreResult<UpdateManifestV1> {
    let m: UpdateManifestV1 = from_json_slice(s.as_bytes())?;
    if m.schema_version != 1 {
        return Err(CoreError::config(
            "unsupported update manifest schemaVersion",
        ));
    }
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_manifest_rejects_schema_0() {
        let raw = r#"{"schemaVersion":0,"releases":[]}"#;
        let err = update_manifest_from_str(raw).unwrap_err();
        assert!(err.to_string().contains("schemaVersion"));
    }
}
