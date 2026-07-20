//! Asset inventory and hashing placeholders.

use dare_core::{validate_nonempty_name, CoreResult};

/// Smoke: valida label via core sem materializar assets.
pub fn assets_layer_ping(label: &str) -> CoreResult<&'static str> {
    validate_nonempty_name(label)?;
    Ok("assets-ok")
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::CoreError;

    #[test]
    fn ping_ok() {
        assert_eq!(assets_layer_ping("pack"), Ok("assets-ok"));
    }

    #[test]
    fn ping_empty_err() {
        assert!(matches!(
            assets_layer_ping(""),
            Err(CoreError::InvalidArgument(_))
        ));
    }
}
