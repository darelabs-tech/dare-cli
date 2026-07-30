//! `dare server --protocol rest` — thin CLI over `dare_server::serve` (microplano 051).

use std::path::PathBuf;
use std::process::ExitCode;

use dare_core::{CoreError, CoreResult};
use dare_server::{parse_server_config_from_env, AppMode};

use crate::commands::dashboard::serve_until_ctrl_c;
use crate::output::OutputRenderer;

/// Message pattern for unknown `--protocol` (BLUEPRINT §0.1 `MSG_UNKNOWN_PROTOCOL`).
pub fn msg_unknown_protocol(protocol: &str) -> String {
    format!("unknown protocol: {protocol} (expected rest)")
}

/// CLI args for `dare server`.
pub struct ServerCliOpts {
    pub protocol: String,
    pub bind: Option<String>,
    pub port: Option<u16>,
    pub dir: Option<PathBuf>,
}

/// Run `dare server --protocol rest [--bind] [--port] [-d]`.
pub fn run_server_cmd(opts: ServerCliOpts, renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_server(opts) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let code = renderer.write_error(&e);
            ExitCode::from(code as u8)
        }
    }
}

fn run_server(opts: ServerCliOpts) -> CoreResult<()> {
    if opts.protocol != "rest" {
        return Err(CoreError::usage(msg_unknown_protocol(&opts.protocol)));
    }
    let root_path = opts
        .dir
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let cfg = parse_server_config_from_env(
        AppMode::Rest,
        opts.bind.as_deref(),
        opts.port,
        &root_path,
        false,
    )?;
    serve_until_ctrl_c(AppMode::Rest, cfg)
}
