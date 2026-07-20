//! Persisted contract schemas and compatibility placeholders.

use dare_core as _;

/// Identificador de schema de contrato (placeholder estável).
pub const CONTRACTS_SCHEMA_VERSION: &str = "0.0.0-placeholder";

/// Retorna a versão de schema anunciada por esta crate.
pub fn schema_version() -> &'static str {
    CONTRACTS_SCHEMA_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_is_placeholder() {
        assert_eq!(schema_version(), "0.0.0-placeholder");
    }
}
