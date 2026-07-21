//! Tracing initialization for the DARE CLI.

use tracing_subscriber::EnvFilter;

use crate::context::ExecutionContext;
use crate::error::CoreError;

/// Init subscriber: EnvFilter from `RUST_LOG`, default `warn`.
/// Includes `correlation_id` as a field on the root span when callers create one.
/// Idempotent enough for tests (Ok if already initialized).
pub fn init_tracing(ctx: &ExecutionContext) -> Result<(), CoreError> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    let _ = ctx; // correlation applied by callers via span fields
    let result = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
    match result {
        Ok(()) => Ok(()),
        Err(_) => Ok(()), // already initialized
    }
}

/// Instala subscriber fmt para testes. Idempotente o suficiente para testes unitários.
pub fn init_test_subscriber() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
}
