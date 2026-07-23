//! Scan rules loading.

use dare_core::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

use crate::report::FindingSeverity;

/// Embedded default rules (same as assets/rules/scan-rules.json).
pub const DEFAULT_RULES_JSON: &str = include_str!("../../../assets/rules/scan-rules.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanRulesFile {
    pub version: u32,
    pub rules: Vec<ScanRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanRule {
    pub id: String,
    pub severity: FindingSeverity,
    #[serde(default)]
    pub description: String,
    pub pattern: String,
}

pub fn load_rules_from_str(json: &str) -> CoreResult<ScanRulesFile> {
    let file: ScanRulesFile =
        serde_json::from_str(json).map_err(|e| CoreError::config(format!("scan-rules: {e}")))?;
    if file.rules.is_empty() {
        return Err(CoreError::config("scan-rules: empty rules"));
    }
    for r in &file.rules {
        if r.id.is_empty() || r.pattern.is_empty() {
            return Err(CoreError::config("scan-rules: rule id/pattern required"));
        }
    }
    Ok(file)
}

/// Load rules from path, else `DARE_GUARD_SCAN_RULES_PATH`, else embedded default.
pub fn load_rules(path_override: Option<&std::path::Path>) -> CoreResult<ScanRulesFile> {
    if let Some(p) = path_override {
        let raw = std::fs::read_to_string(p)
            .map_err(|e| CoreError::io(format!("read scan-rules {}: {e}", p.display())))?;
        return load_rules_from_str(&raw);
    }
    if let Ok(env_path) = std::env::var("DARE_GUARD_SCAN_RULES_PATH") {
        if !env_path.is_empty() {
            let raw = std::fs::read_to_string(&env_path).map_err(|e| {
                CoreError::io(format!("read DARE_GUARD_SCAN_RULES_PATH {env_path}: {e}"))
            })?;
            return load_rules_from_str(&raw);
        }
    }
    load_rules_from_str(DEFAULT_RULES_JSON)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_rules_parse() {
        let f = load_rules_from_str(DEFAULT_RULES_JSON).expect("parse");
        assert_eq!(f.rules.len(), 4);
        assert!(f.rules.iter().any(|r| r.id == "instr-override"));
    }
}
