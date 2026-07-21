//! Validate effective `DareConfig` with JSON Pointer diagnostics.

use dare_contracts::DareConfig;
use dare_core::{CoreError, CoreResult};

pub fn validate(cfg: &DareConfig) -> CoreResult<()> {
    if let Some(ide) = &cfg.ide {
        if ide.is_empty() {
            return Err(CoreError::config(
                "invalid dare.config.json at /ide: must be non-empty",
            ));
        }
    }
    // enabled:false blocks: no deep validation required (types already deserialized)
    let _ = (&cfg.project, &cfg.agent, &cfg.guard, &cfg.graph, &cfg.hooks);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_contracts::ConfigObject;

    #[test]
    fn empty_ide_fails_with_pointer() {
        let cfg = DareConfig {
            ide: Some(String::new()),
            ..Default::default()
        };
        let err = validate(&cfg).unwrap_err();
        assert!(err.to_string().contains("/ide"));
    }

    #[test]
    fn enabled_false_ok() {
        let cfg = DareConfig {
            guard: Some(ConfigObject {
                enabled: Some(false),
                extra: Default::default(),
            }),
            ..Default::default()
        };
        assert!(validate(&cfg).is_ok());
    }
}
