//! Runtime configuration read from environment.
use std::env;

use tower_http::cors::AllowOrigin;

#[derive(Clone, Debug)]
pub struct Config {
    pub app_port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_secs: i64,
    pub rate_limit_per_sec: u64,
    pub rate_limit_burst: u32,
    pub cors_origins: Vec<String>,
    pub llm_provider: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            app_port: env::var("APP_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8000),
            database_url: env::var("DATABASE_URL")?,
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "replace-me-in-prod".into()),
            jwt_expires_secs: env::var("JWT_EXPIRES_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(900),
            rate_limit_per_sec: env::var("RATE_LIMIT_PER_SEC").ok().and_then(|s| s.parse().ok()).unwrap_or(10),
            rate_limit_burst: env::var("RATE_LIMIT_BURST").ok().and_then(|s| s.parse().ok()).unwrap_or(20),
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".into())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            llm_provider: env::var("LLM_PROVIDER").unwrap_or_else(|_| "dummy".into()),
        })
    }

    pub fn cors_layer_origins(&self) -> AllowOrigin {
        let parsed: Vec<axum::http::HeaderValue> = self
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        AllowOrigin::list(parsed)
    }
}
