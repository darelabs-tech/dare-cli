//! Hooks config helpers (`enabled` / `trusted`) from `DareConfig`.

use dare_contracts::DareConfig;

pub const MSG_HOOKS_TRUST: &str =
    "hooks run requires trust (pass --trust or set hooks.trusted: true) [HOOKS_TRUST]";
pub const MSG_HOOKS_DISABLED: &str =
    "hooks are disabled (hooks.enabled: false) [HOOKS_DISABLED]";

/// Whether hooks are enabled. Default `true` if `hooks` or `enabled` omitted.
pub fn hooks_enabled(cfg: &DareConfig) -> bool {
    cfg.hooks
        .as_ref()
        .and_then(|h| h.enabled)
        .unwrap_or(true)
}

/// Whether hooks are trusted. Default `false`; reads `hooks.extra["trusted"]`.
pub fn hooks_trusted(cfg: &DareConfig) -> bool {
    cfg.hooks
        .as_ref()
        .and_then(|h| h.extra.get("trusted"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_contracts::ConfigObject;
    use serde_json::{Map, Value};

    #[test]
    fn trusted_default_false() {
        let cfg = DareConfig::default();
        assert!(!hooks_trusted(&cfg));

        let cfg = DareConfig {
            hooks: Some(ConfigObject {
                enabled: Some(true),
                extra: Map::new(),
            }),
            ..DareConfig::default()
        };
        assert!(!hooks_trusted(&cfg));

        let mut extra = Map::new();
        extra.insert("trusted".into(), Value::Bool(true));
        let cfg = DareConfig {
            hooks: Some(ConfigObject {
                enabled: None,
                extra,
            }),
            ..DareConfig::default()
        };
        assert!(hooks_trusted(&cfg));
    }

    #[test]
    fn enabled_false() {
        let cfg = DareConfig {
            hooks: Some(ConfigObject {
                enabled: Some(false),
                extra: Map::new(),
            }),
            ..DareConfig::default()
        };
        assert!(!hooks_enabled(&cfg));

        let cfg = DareConfig::default();
        assert!(hooks_enabled(&cfg));

        let cfg = DareConfig {
            hooks: Some(ConfigObject {
                enabled: None,
                extra: Map::new(),
            }),
            ..DareConfig::default()
        };
        assert!(hooks_enabled(&cfg));
    }
}
