//! Bind, serve Axum app, optional browser open, graceful shutdown.

use std::future::Future;
use std::net::{IpAddr, SocketAddr};

use dare_core::{CoreError, CoreResult, SystemProcessRunner};
use tokio::net::TcpListener;
use tracing::warn;

use crate::app::create_app;
use crate::browser::open_browser;
use crate::config::{ServerConfig, TokenSource};
use crate::mode::AppMode;
use crate::state::AppState;

/// Bind `cfg.bind:cfg.port`, serve the shared app, and shut down when `shutdown` completes.
///
/// On [`AppMode::Dashboard`] with `cfg.open_browser`, opens the dashboard URL after listen.
/// Browser failures are logged as warnings and do not fail `serve` (R-06).
pub async fn serve(
    mode: AppMode,
    cfg: ServerConfig,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> CoreResult<()> {
    let addr = SocketAddr::new(cfg.bind, cfg.port);
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| CoreError::io(format!("bind {addr}: {e}")))?;
    let local = listener
        .local_addr()
        .map_err(|e| CoreError::io(format!("local_addr: {e}")))?;

    log_startup(&cfg, local.port());

    let state = AppState::new(
        cfg.project_root.clone(),
        cfg.token.clone(),
        cfg.body_limit,
        mode,
        env!("CARGO_PKG_VERSION"),
    );
    let app = create_app(mode, &cfg, state);

    if mode == AppMode::Dashboard && cfg.open_browser {
        let url = dashboard_open_url(cfg.bind, local.port());
        if let Err(e) = open_browser(&url, &SystemProcessRunner) {
            warn!("failed to open browser for {url}: {e}");
        }
    }

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await
    .map_err(|e| CoreError::io(format!("serve: {e}")))?;

    Ok(())
}

fn log_startup(cfg: &ServerConfig, port: u16) {
    let token_kind = match cfg.token_source {
        TokenSource::Env => "set",
        TokenSource::Generated => "generated",
    };
    if cfg.log_token_value {
        eprintln!(
            "dare-server listening bind={} port={} token={} value={}",
            cfg.bind, port, token_kind, cfg.token
        );
    } else {
        eprintln!(
            "dare-server listening bind={} port={} token={}",
            cfg.bind, port, token_kind
        );
    }
}

/// Prefer `127.0.0.1` for loopback binds (allowlist-safe); otherwise format `cfg.bind`.
fn dashboard_open_url(bind: IpAddr, port: u16) -> String {
    let host = if bind.is_loopback() {
        "127.0.0.1".to_string()
    } else {
        bind.to_string()
    };
    format!("http://{host}:{port}/dashboard")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn dashboard_url_loopback() {
        let ip = IpAddr::from_str("127.0.0.1").unwrap();
        assert_eq!(
            dashboard_open_url(ip, 4100),
            "http://127.0.0.1:4100/dashboard"
        );
    }
}
