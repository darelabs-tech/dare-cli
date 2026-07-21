//! UpdateManifestV2 load/parse (desired-state inventário fechado).

use dare_assets::{assert_safe_asset_path, EmbeddedAssets};
use dare_core::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

use crate::UPDATE_MANIFEST_V2_SCHEMA;

/// Desired-state update manifest (schemaVersion 2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManifestV2 {
    pub schema_version: u32,
    pub cli_version: String,
    pub releases: Vec<ReleaseEntry>,
    pub assets: Vec<DesiredAsset>,
}

/// One declared release in the V2 series.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseEntry {
    pub version: String,
    #[serde(default)]
    pub notes: String,
}

/// Closed-inventory asset entry with expected SHA and harness filter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DesiredAsset {
    pub path: String,
    pub sha256: String,
    pub applies_to: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

const EMBEDDED_V2_PATH: &str = "update-manifest.v2.json";

/// Parse and validate an UpdateManifestV2 from a JSON string.
pub fn load_desired_manifest_v2_from_str(s: &str) -> CoreResult<UpdateManifestV2> {
    let m: UpdateManifestV2 = serde_json::from_str(s)
        .map_err(|e| CoreError::config(format!("invalid update manifest v2 json: {e}")))?;
    validate_manifest_v2(&m)?;
    Ok(m)
}

/// Load the embedded `assets/update-manifest.v2.json` desired-state manifest.
pub fn load_desired_manifest_v2_embedded() -> CoreResult<UpdateManifestV2> {
    let file = EmbeddedAssets::get(EMBEDDED_V2_PATH).ok_or_else(|| {
        CoreError::not_found(format!("embedded asset missing: {EMBEDDED_V2_PATH}"))
    })?;
    let text = std::str::from_utf8(file.data.as_ref())
        .map_err(|e| CoreError::config(format!("invalid update manifest v2 encoding: {e}")))?;
    load_desired_manifest_v2_from_str(text)
}

fn validate_manifest_v2(m: &UpdateManifestV2) -> CoreResult<()> {
    if m.schema_version != UPDATE_MANIFEST_V2_SCHEMA {
        return Err(CoreError::config(
            "unsupported update manifest schemaVersion",
        ));
    }
    if m.assets.is_empty() {
        return Err(CoreError::config(
            "update manifest v2 assets must not be empty",
        ));
    }

    for asset in &m.assets {
        assert_safe_asset_path(&asset.path)?;
        validate_sha256_hex(&asset.sha256)?;
        if asset.applies_to.is_empty() {
            return Err(CoreError::config(format!(
                "update manifest v2 appliesTo empty for path: {}",
                asset.path
            )));
        }
        if let Some(source) = &asset.source {
            assert_safe_asset_path(source)?;
        }
    }

    let has_codex_agents = m
        .assets
        .iter()
        .any(|a| a.path == "AGENTS.md" && a.applies_to.iter().any(|t| t == "codex"));
    if !has_codex_agents {
        return Err(CoreError::config(
            "update manifest v2 requires AGENTS.md with appliesTo containing codex",
        ));
    }

    Ok(())
}

fn validate_sha256_hex(s: &str) -> CoreResult<()> {
    if s.len() != 64 {
        return Err(CoreError::config(format!(
            "invalid sha256 length (want 64): {}",
            s.len()
        )));
    }
    if !s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        return Err(CoreError::config(
            "invalid sha256: must be 64 lowercase hex chars",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ErrorKind;

    fn valid_sha() -> String {
        "a".repeat(64)
    }

    fn minimal_valid_json(agents_applies: &str, extra_path: Option<&str>) -> String {
        let extra = match extra_path {
            Some(p) => format!(
                r#",{{"path":"{p}","sha256":"{}","appliesTo":["*"]}}"#,
                valid_sha()
            ),
            None => String::new(),
        };
        format!(
            r#"{{
              "schemaVersion": 2,
              "cliVersion": "0.1.0-alpha.0",
              "releases": [{{"version":"0.1.0-alpha.0","notes":""}}],
              "assets": [
                {{"path":"AGENTS.md","sha256":"{}","appliesTo":{agents_applies}}}
                {extra}
              ]
            }}"#,
            valid_sha()
        )
    }

    #[test]
    fn v2_rejects_schema_1() {
        let raw = r#"{
          "schemaVersion": 1,
          "cliVersion": "0.1.0-alpha.0",
          "releases": [],
          "assets": [
            {"path":"AGENTS.md","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","appliesTo":["codex"]}
          ]
        }"#;
        let err = load_desired_manifest_v2_from_str(raw).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Config);
        assert!(
            err.message()
                .contains("unsupported update manifest schemaVersion"),
            "{}",
            err.message()
        );
    }

    #[test]
    fn v2_requires_codex_agents() {
        let raw = format!(
            r#"{{
              "schemaVersion": 2,
              "cliVersion": "0.1.0-alpha.0",
              "releases": [{{"version":"0.1.0-alpha.0","notes":""}}],
              "assets": [
                {{"path":"CLAUDE.md","sha256":"{}","appliesTo":["claude-code"]}}
              ]
            }}"#,
            valid_sha()
        );
        let err = load_desired_manifest_v2_from_str(&raw).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Config);
        assert!(
            err.message().contains("AGENTS.md") && err.message().contains("codex"),
            "{}",
            err.message()
        );
    }

    #[test]
    fn v2_rejects_bad_path() {
        let raw = format!(
            r#"{{
              "schemaVersion": 2,
              "cliVersion": "0.1.0-alpha.0",
              "releases": [{{"version":"0.1.0-alpha.0","notes":""}}],
              "assets": [
                {{"path":"AGENTS.md","sha256":"{}","appliesTo":["codex"]}},
                {{"path":"../x","sha256":"{}","appliesTo":["*"]}}
              ]
            }}"#,
            valid_sha(),
            valid_sha()
        );
        let err = load_desired_manifest_v2_from_str(&raw).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Config);
        assert!(
            err.message().contains("dot-dot") || err.message().contains("../"),
            "{}",
            err.message()
        );
    }

    #[test]
    fn legacy_v1_still_loads() {
        let raw = r#"{"schemaVersion":1,"releases":[{"version":"3.8.2","notes":"baseline"}]}"#;
        let m = dare_contracts::update_manifest_from_str(raw).expect("v1 must load");
        assert_eq!(m.schema_version, 1);
    }

    #[test]
    fn embedded_v2_loads() {
        let m = load_desired_manifest_v2_embedded().expect("embedded v2");
        assert_eq!(m.schema_version, UPDATE_MANIFEST_V2_SCHEMA);
        assert_eq!(m.cli_version, "0.1.0-alpha.0");
        assert!(m.releases.iter().any(|r| r.version == "0.1.0-alpha.0"));
        assert!(m
            .assets
            .iter()
            .any(|a| { a.path == "AGENTS.md" && a.applies_to.iter().any(|t| t == "codex") }));
        assert!(m
            .assets
            .iter()
            .any(|a| a.applies_to.iter().any(|t| t == "*")));
        assert!(m
            .assets
            .iter()
            .any(|a| a.applies_to.iter().any(|t| t == "claude-code")));
        assert!(m
            .assets
            .iter()
            .any(|a| a.applies_to.iter().any(|t| t == "cursor")));
        assert!(m
            .assets
            .iter()
            .any(|a| a.applies_to.iter().any(|t| t == "antigravity")));
    }

    #[test]
    fn minimal_valid_parses() {
        let raw = minimal_valid_json(r#"["codex"]"#, Some("templates/x.md"));
        let m = load_desired_manifest_v2_from_str(&raw).unwrap();
        assert_eq!(m.assets.len(), 2);
    }
}
