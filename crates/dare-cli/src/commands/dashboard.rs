//! `dare dashboard` — thin CLI over `dare_server::serve` (microplano 051).

use std::path::PathBuf;
use std::process::ExitCode;

use dare_core::{CoreError, CoreResult};
use dare_server::{parse_server_config_from_env, serve, AppMode};

use crate::output::OutputRenderer;

/// CLI args for `dare dashboard`.
pub struct DashboardCliOpts {
    pub port: Option<u16>,
    pub no_open: bool,
    pub dir: Option<PathBuf>,
}

/// Run `dare dashboard [--port] [--no-open] [-d]`.
pub fn run_dashboard_cmd(opts: DashboardCliOpts, renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_dashboard(opts) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let code = renderer.write_error(&e);
            ExitCode::from(code as u8)
        }
    }
}

fn run_dashboard(opts: DashboardCliOpts) -> CoreResult<()> {
    let root_path = opts
        .dir
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let cfg = parse_server_config_from_env(
        AppMode::Dashboard,
        None,
        opts.port,
        &root_path,
        !opts.no_open,
    )?;
    serve_until_ctrl_c(AppMode::Dashboard, cfg)
}

/// Block the current thread on `serve` until Ctrl+C / shutdown.
pub(crate) fn serve_until_ctrl_c(
    mode: AppMode,
    cfg: dare_server::ServerConfig,
) -> CoreResult<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CoreError::io(format!("tokio runtime: {e}")))?;
    rt.block_on(async move {
        serve(mode, cfg, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
    })
}
