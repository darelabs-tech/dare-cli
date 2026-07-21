//! Parse allowlisted `DARE_*` environment variables.

use dare_core::{CoreError, CoreResult};

use crate::r#override::EnvOverrides;

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn block_from_env_key(key: &str) -> Option<&'static str> {
    match key {
        "DARE_GUARD_ENABLED" => Some("guard"),
        "DARE_GRAPH_ENABLED" => Some("graph"),
        "DARE_AGENT_ENABLED" => Some("agent"),
        "DARE_HOOKS_ENABLED" => Some("hooks"),
        "DARE_PROJECT_ENABLED" => Some("project"),
        _ => None,
    }
}

/// Lenient parse of allowlisted `DARE_*` vars.
///
/// Unknown keys are ignored. Invalid `*_ENABLED` bools are **skipped** (no error).
/// Prefer [`env_overrides_from_vars_strict`] when invalid values must fail loudly.
pub fn env_overrides_from_vars<K, V, I>(vars: I) -> EnvOverrides
where
    K: AsRef<str>,
    V: AsRef<str>,
    I: IntoIterator<Item = (K, V)>,
{
    let mut out = EnvOverrides::default();
    for (k, v) in vars {
        let key = k.as_ref();
        let val = v.as_ref();
        if key == "DARE_IDE" {
            if !val.is_empty() {
                out.ide = Some(val.to_string());
            }
            continue;
        }
        if let Some(block) = block_from_env_key(key) {
            if let Some(b) = parse_bool(val) {
                out.block_enabled.insert(block.to_string(), b);
            }
        }
    }
    out
}

/// Strict parse: invalid `*_ENABLED` values return `CoreError::Config` with pointer `/env/{KEY}`.
///
/// Error messages intentionally omit the raw env value (RS-02 redact). Unknown keys ignored.
pub fn env_overrides_from_vars_strict<K, V, I>(vars: I) -> CoreResult<EnvOverrides>
where
    K: AsRef<str>,
    V: AsRef<str>,
    I: IntoIterator<Item = (K, V)>,
{
    let mut out = EnvOverrides::default();
    for (k, v) in vars {
        let key = k.as_ref();
        let val = v.as_ref();
        if key == "DARE_IDE" {
            if !val.is_empty() {
                out.ide = Some(val.to_string());
            }
            continue;
        }
        if let Some(block) = block_from_env_key(key) {
            match parse_bool(val) {
                Some(b) => {
                    out.block_enabled.insert(block.to_string(), b);
                }
                None => {
                    return Err(CoreError::config(format!(
                        "invalid dare.config.json at /env/{key}: expected boolean"
                    )));
                }
            }
        }
    }
    Ok(out)
}

/// Read process env via [`env_overrides_from_vars_strict`] (`std::env::vars()`).
pub fn env_overrides_from_os() -> CoreResult<EnvOverrides> {
    env_overrides_from_vars_strict(std::env::vars())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_allowlist() {
        let env = env_overrides_from_vars([
            ("DARE_IDE", "claude"),
            ("DARE_GUARD_ENABLED", "true"),
            ("DARE_UNKNOWN", "x"),
            ("OTHER", "y"),
        ]);
        assert_eq!(env.ide.as_deref(), Some("claude"));
        assert_eq!(env.block_enabled.get("guard"), Some(&true));
    }

    #[test]
    fn strict_rejects_bad_bool() {
        let err = env_overrides_from_vars_strict([("DARE_GUARD_ENABLED", "maybe")]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("/env/DARE_GUARD_ENABLED"), "{msg}");
        assert!(
            !msg.contains("maybe"),
            "error must not echo raw env value: {msg}"
        );
    }
}
