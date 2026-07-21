//! Canonical defaults for `DareConfig`.

use dare_contracts::DareConfig;

/// Defaults: no ide, no blocks (maximizes legacy fixture compatibility).
pub fn default_config() -> DareConfig {
    DareConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::to_canonical_json_string;
    use serde_json::from_str;

    #[test]
    fn default_roundtrip() {
        let cfg = default_config();
        let s = to_canonical_json_string(&serde_json::to_value(&cfg).unwrap()).unwrap();
        let back: DareConfig = from_str(&s).unwrap();
        assert_eq!(back, cfg);
        assert!(back.ide.is_none());
        assert!(back.guard.is_none());
    }
}
