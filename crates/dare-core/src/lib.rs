//! Core types, errors, and tracing stubs for the DARE CLI workspace.

mod error;
mod tracing_init;

pub use error::{CoreError, CoreResult};
pub use tracing_init::init_test_subscriber;

/// Valida que `name` não é vazio e não contém NUL.
pub fn validate_nonempty_name(name: &str) -> CoreResult<()> {
    if name.is_empty() {
        return Err(CoreError::InvalidArgument("name must not be empty".into()));
    }
    if name.contains('\0') {
        return Err(CoreError::InvalidArgument(
            "name must not contain NUL".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_nonempty_name_ok() {
        assert_eq!(validate_nonempty_name("dare"), Ok(()));
    }

    #[test]
    fn validate_nonempty_name_empty_err() {
        assert!(matches!(
            validate_nonempty_name(""),
            Err(CoreError::InvalidArgument(_))
        ));
    }

    #[test]
    fn validate_nonempty_name_nul_err() {
        assert!(matches!(
            validate_nonempty_name("a\0b"),
            Err(CoreError::InvalidArgument(_))
        ));
    }
}
