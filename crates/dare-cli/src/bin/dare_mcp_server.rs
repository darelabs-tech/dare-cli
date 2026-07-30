//! Transition alias `dare-mcp-server` (BLUEPRINT-052 §0.3 / RF-21).
//!
//! Always serves legacy REST (`AppMode::Rest`) and prints a deprecation
//! notice on stderr. Never starts MCP/JSON-RPC.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use dare_cli::{run_rest_server, RestServerOpts, MSG_ALIAS_DEPRECATED};

#[derive(Debug, Parser)]
#[command(
    name = "dare-mcp-server",
    about = "Deprecated REST alias — use `dare server --protocol rest` or `dare mcp serve`"
)]
struct Cli {
    /// Bind address (default: 127.0.0.1).
    #[arg(long)]
    bind: Option<String>,
    /// Listen port (default: 3000).
    #[arg(long)]
    port: Option<u16>,
    /// Project directory (default: cwd; also honors DARE_PROJECT_PATH).
    #[arg(short = 'd', long = "dir")]
    dir: Option<PathBuf>,
}

fn main() -> ExitCode {
    // ALWAYS print before listen (and before clap early-exit paths that still reach main).
    eprintln!("{MSG_ALIAS_DEPRECATED}");

    let cli = Cli::parse();
    match run_rest_server(RestServerOpts {
        bind: cli.bind,
        port: cli.port,
        dir: cli.dir,
    }) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e.message());
            ExitCode::from(e.exit_code() as u8)
        }
    }
}
