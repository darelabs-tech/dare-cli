//! Configuration loading placeholders (full loader in later microplans).

use dare_contracts::schema_version;
use dare_core::{validate_nonempty_name, CoreResult};

/// Smoke: compõe core + contracts sem carregar disco.
pub fn config_layer_ping(label: &str) -> CoreResult<&'static str> {
    validate_nonempty_name(label)?;
    let _ = schema_version();
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
            Err(CoreError::InvalidArgument(_))
        ));
    }
}
