//! Resolution of the consumer-side handoff policy (D-03, A-07).
//!
//! The policy is opt-in and defaults to permissive, so a project that never
//! configures it keeps today's behaviour byte for byte. Keys are read out of
//! `dare.config.json` extras exactly as
//! [`dare_verify::verify_enabled_from_cfg`] reads `verify.enabled`, which keeps
//! the generated config untouched: absence of the whole `handoff` object means
//! every default applies (RF-10).

use dare_contracts::{DareConfig, HandoffKind, HandoffPolicy};

/// The two mutually overriding command-line flags.
///
/// Both default to `false`, which means "say nothing and let the config
/// decide".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HandoffFlags {
    /// `--strict-handoff`
    pub strict: bool,
    /// `--no-strict-handoff`
    pub no_strict: bool,
}

/// Resolves the policy from config extras and the command-line flags.
///
/// Precedence is `--strict-handoff` > `--no-strict-handoff` > config >
/// `false`. `require_contracts` defaults to whatever `strict` resolved to,
/// because strict enforcement without contracts is a misconfiguration the
/// preflight is meant to catch rather than a usable mode.
///
/// `required_kind` is always [`HandoffKind::VerifiedDeliverable`] in v1: a
/// follow-up handoff never satisfies a dependency.
pub fn handoff_policy_from_cfg(cfg: &DareConfig, flags: &HandoffFlags) -> HandoffPolicy {
    let configured_strict = bool_at(cfg, "strict").unwrap_or(false);
    let strict = if flags.strict {
        true
    } else if flags.no_strict {
        false
    } else {
        configured_strict
    };

    HandoffPolicy {
        strict,
        require_attested_revision: bool_at(cfg, "requireAttestedRevision").unwrap_or(false),
        require_contracts: bool_at(cfg, "requireContracts").unwrap_or(strict),
        required_kind: HandoffKind::VerifiedDeliverable,
    }
}

/// Reads `handoff.<key>` from config extras as a bool, if it is one.
fn bool_at(cfg: &DareConfig, key: &str) -> Option<bool> {
    cfg.extra
        .get("handoff")
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_bool())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cfg_with(handoff: serde_json::Value) -> DareConfig {
        let mut cfg = DareConfig::default();
        cfg.extra.insert("handoff".to_string(), handoff);
        cfg
    }

    fn flags(strict: bool, no_strict: bool) -> HandoffFlags {
        HandoffFlags { strict, no_strict }
    }

    #[test]
    fn policy_default_is_not_strict() {
        let policy = handoff_policy_from_cfg(&DareConfig::default(), &HandoffFlags::default());
        assert!(!policy.strict, "absence of config must never enable strict");
        assert_eq!(policy, HandoffPolicy::default());
    }

    #[test]
    fn policy_from_config_true() {
        let cfg = cfg_with(json!({ "strict": true }));
        let policy = handoff_policy_from_cfg(&cfg, &HandoffFlags::default());
        assert!(policy.strict);
    }

    #[test]
    fn policy_flag_overrides_config_false() {
        let cfg = cfg_with(json!({ "strict": false }));
        let policy = handoff_policy_from_cfg(&cfg, &flags(true, false));
        assert!(policy.strict, "--strict-handoff outranks the config");
    }

    #[test]
    fn policy_no_strict_overrides_config_true() {
        let cfg = cfg_with(json!({ "strict": true }));
        let policy = handoff_policy_from_cfg(&cfg, &flags(false, true));
        assert!(!policy.strict, "--no-strict-handoff outranks the config");
    }

    #[test]
    fn policy_strict_flag_wins_over_no_strict() {
        // clap's `overrides_with` keeps the pair exclusive, but the resolver
        // must still be total if both ever arrive set.
        let cfg = cfg_with(json!({ "strict": false }));
        let policy = handoff_policy_from_cfg(&cfg, &flags(true, true));
        assert!(policy.strict);
    }

    #[test]
    fn policy_require_contracts_follows_strict() {
        let strict = handoff_policy_from_cfg(&DareConfig::default(), &flags(true, false));
        assert!(
            strict.require_contracts,
            "strict without contracts is a misconfiguration, not a mode"
        );

        let loose = handoff_policy_from_cfg(&DareConfig::default(), &HandoffFlags::default());
        assert!(!loose.require_contracts);

        // An explicit value still wins over the derived default.
        let cfg = cfg_with(json!({ "strict": true, "requireContracts": false }));
        let explicit = handoff_policy_from_cfg(&cfg, &HandoffFlags::default());
        assert!(explicit.strict);
        assert!(!explicit.require_contracts);
    }

    #[test]
    fn policy_require_attested_default_false() {
        let policy = handoff_policy_from_cfg(&DareConfig::default(), &flags(true, false));
        assert!(
            !policy.require_attested_revision,
            "attestation is opt-in even under strict"
        );

        let cfg = cfg_with(json!({ "requireAttestedRevision": true }));
        let on = handoff_policy_from_cfg(&cfg, &HandoffFlags::default());
        assert!(on.require_attested_revision);
    }

    #[test]
    fn policy_required_kind_is_always_final() {
        for f in [
            HandoffFlags::default(),
            flags(true, false),
            flags(false, true),
        ] {
            let policy = handoff_policy_from_cfg(&DareConfig::default(), &f);
            assert_eq!(policy.required_kind, HandoffKind::VerifiedDeliverable);
        }
    }

    #[test]
    fn policy_ignores_non_bool_values() {
        let cfg = cfg_with(json!({ "strict": "yes", "requireContracts": 1 }));
        let policy = handoff_policy_from_cfg(&cfg, &HandoffFlags::default());
        assert!(!policy.strict, "a non-bool must not be coerced into true");
        assert!(!policy.require_contracts);
    }
}
