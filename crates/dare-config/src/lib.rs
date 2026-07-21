//! Configuration loading, merge, validation and migrations (microplano 008).

mod defaults;
mod env;
mod load;
mod merge;
mod migrate;
mod r#override;
mod validate;

pub use defaults::default_config;
pub use env::{env_overrides_from_os, env_overrides_from_vars, env_overrides_from_vars_strict};
pub use load::{load_effective, DEFAULT_CONFIG_REL};
pub use merge::merge_layers;
pub use migrate::{
    apply_migrate, apply_plan_in_memory, dry_run_migrate, plan_migrate, MigrateDryRunReport,
    MigrateOptions, MigrationPlan, MigrationStep, MigrationStepKind,
};
pub use r#override::{CliOverrides, EnvOverrides};
pub use validate::validate;

use dare_contracts::schema_version;
use dare_core::{validate_nonempty_name, CoreResult};

/// Smoke: compõe core + contracts sem carregar disco.
pub fn config_layer_ping(label: &str) -> CoreResult<&'static str> {
    validate_nonempty_name(label)?;
    let _ = schema_version();
    let _ = default_config();
    Ok("config-ok")
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::CoreError;

    #[test]
    fn ping_ok() {
        assert_eq!(config_layer_ping("local"), Ok("config-ok"));
    }

    #[test]
    fn ping_empty_err() {
        assert!(matches!(
            config_layer_ping(""),
            Err(CoreError::InvalidInput(_))
        ));
    }
}
