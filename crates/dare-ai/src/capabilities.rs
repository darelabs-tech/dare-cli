//! Provider capabilities report (schemaVersion 1).

use serde::{Deserialize, Serialize};

use crate::provider::ProviderId;
use crate::{
    ENRICH_TIMEOUT, ENV_ANTIGRAVITY, ENV_CLAUDE, ENV_CODEX, ENV_CURSOR,
};

/// Frozen JSON schema version for [`ProvidersReport`].
pub const PROVIDERS_SCHEMA_VERSION: u32 = 1;

/// Canonical provider order (BLUEPRINT-050 §4.2).
pub const PROVIDER_ORDER: &[ProviderId] = &[
    ProviderId::Mock,
    ProviderId::Codex,
    ProviderId::ClaudeCode,
    ProviderId::CursorCli,
    ProviderId::AntigravityCli,
];

const CAPABILITY_COMMANDS: &[&str] = &["design", "blueprint"];

/// One provider row in [`ProvidersReport`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCapability {
    pub id: String,
    pub enrich: bool,
    pub implemented: bool,
    pub env_override: Option<String>,
    pub default_timeout_secs: u64,
    pub commands: Vec<String>,
}

/// Providers capability list (`schemaVersion` 1, camelCase JSON).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProvidersReport {
    pub schema_version: u32,
    pub providers: Vec<ProviderCapability>,
}

/// Env override variable name for a provider, if any.
pub fn env_override_name(id: ProviderId) -> Option<&'static str> {
    match id {
        ProviderId::Mock => None,
        ProviderId::Codex => Some(ENV_CODEX),
        ProviderId::ClaudeCode => Some(ENV_CLAUDE),
        ProviderId::CursorCli => Some(ENV_CURSOR),
        ProviderId::AntigravityCli => Some(ENV_ANTIGRAVITY),
    }
}

/// Default program basename when no env override is set.
pub fn default_program(id: ProviderId) -> &'static str {
    match id {
        ProviderId::Mock => "mock",
        ProviderId::Codex => "codex",
        ProviderId::ClaudeCode => "claude",
        ProviderId::CursorCli => "cursor",
        ProviderId::AntigravityCli => "antigravity",
    }
}

/// Whether enrich is implemented for this provider in v1.
pub fn is_implemented(id: ProviderId) -> bool {
    matches!(
        id,
        ProviderId::Mock
            | ProviderId::Codex
            | ProviderId::ClaudeCode
            | ProviderId::CursorCli
            | ProviderId::AntigravityCli
    )
}

/// Default enrich timeout in seconds (1200).
pub fn default_timeout_secs() -> u64 {
    ENRICH_TIMEOUT.as_secs()
}

/// List provider capabilities in canonical order.
pub fn list_provider_capabilities() -> ProvidersReport {
    let timeout = default_timeout_secs();
    let commands: Vec<String> = CAPABILITY_COMMANDS.iter().map(|s| (*s).to_string()).collect();
    let providers = PROVIDER_ORDER
        .iter()
        .copied()
        .map(|id| {
            let implemented = is_implemented(id);
            ProviderCapability {
                id: id.as_str().to_string(),
                enrich: implemented,
                implemented,
                env_override: env_override_name(id).map(str::to_string),
                default_timeout_secs: timeout,
                commands: commands.clone(),
            }
        })
        .collect();
    ProvidersReport {
        schema_version: PROVIDERS_SCHEMA_VERSION,
        providers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn providers_order() {
        let report = list_provider_capabilities();
        assert_eq!(report.schema_version, 1);
        let ids: Vec<&str> = report.providers.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "mock",
                "codex",
                "claude-code",
                "cursor-cli",
                "antigravity-cli"
            ]
        );
        for p in &report.providers {
            assert_eq!(p.default_timeout_secs, 1200);
            assert_eq!(p.commands, vec!["design", "blueprint"]);
        }
        assert!(report.providers[0].implemented);
        assert!(report.providers[0].enrich);
        assert!(report.providers[1].implemented);
        assert!(report.providers[2].implemented);
        assert!(report.providers[2].enrich);
        assert!(report.providers[3].implemented);
        assert!(report.providers[4].implemented);
        assert_eq!(report.providers[1].env_override.as_deref(), Some(ENV_CODEX));
        assert_eq!(report.providers[2].env_override.as_deref(), Some(ENV_CLAUDE));
        assert!(report.providers[0].env_override.is_none());
    }

    #[test]
    fn providers_report_camel_case_json() {
        let report = list_provider_capabilities();
        let json = serde_json::to_string(&report).expect("serialize");
        assert!(json.contains("\"schemaVersion\":1"));
        assert!(json.contains("\"envOverride\""));
        assert!(json.contains("\"defaultTimeoutSecs\":1200"));
    }
}
