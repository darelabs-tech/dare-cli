//! Public crate surface — re-exported for tests and bins.
pub mod config;
pub mod errors;
pub mod handlers;
pub mod llm;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod services;

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub cfg: config::Config,
}

pub type SharedState = Arc<AppState>;
