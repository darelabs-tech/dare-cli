//! Load effective config from disk + overrides.

use dare_contracts::{load_dare_config, DareConfig};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};

use crate::defaults::default_config;
use crate::merge::merge_layers;
use crate::r#override::{CliOverrides, EnvOverrides};
use crate::validate::validate;

pub const DEFAULT_CONFIG_REL: &str = "dare.config.json";

pub fn load_effective(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    env: &EnvOverrides,
    cli: &CliOverrides,
) -> CoreResult<DareConfig> {
    let defaults = default_config();
    let file = match load_dare_config(root, rel) {
        Ok(cfg) => Some(cfg),
        Err(CoreError::NotFound(_)) => None,
        Err(e) => return Err(e),
    };
    let cfg = merge_layers(&defaults, file.as_ref(), env, cli);
    validate(&cfg)?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn missing_file_uses_defaults() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new(DEFAULT_CONFIG_REL).unwrap();
        let cfg = load_effective(
            &root,
            &rel,
            &EnvOverrides::default(),
            &CliOverrides::default(),
        )
        .unwrap();
        assert!(cfg.ide.is_none());
    }

    #[test]
    fn malformed_propagates_config_error() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new(DEFAULT_CONFIG_REL).unwrap();
        std::fs::write(dir.path().join(DEFAULT_CONFIG_REL), "{not-json").unwrap();
        let err = load_effective(
            &root,
            &rel,
            &EnvOverrides::default(),
            &CliOverrides::default(),
        )
        .unwrap_err();
        assert!(matches!(err, CoreError::Config(_)));
    }
}
