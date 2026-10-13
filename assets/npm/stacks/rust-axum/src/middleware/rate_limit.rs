//! Rate limit via tower_governor — per-IP token bucket.
//!
//! We apply the layer through a generic helper instead of naming the
//! `GovernorLayer<K, M>` generics explicitly — the concrete extractor /
//! middleware types vary across tower_governor versions, so we let type
//! inference fill them in.
use std::sync::Arc;

use axum::Router;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

use crate::config::Config;

/// Apply the per-IP rate limit layer to a router.
pub fn with_rate_limit<S>(router: Router<S>, cfg: &Config) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(cfg.rate_limit_per_sec)
            .burst_size(cfg.rate_limit_burst)
            .finish()
            .expect("valid governor config"),
    );
    router.layer(GovernorLayer { config: conf })
}
