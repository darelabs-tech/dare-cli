//! DARE Framework CLI (native Rust rewrite) — help/version surface only.

use anyhow::Result;
use clap::{CommandFactory, Parser};

#[derive(Debug, Parser)]
#[command(
    name = "dare",
    version,
    about = "DARE Framework CLI (native Rust rewrite)",
    disable_help_subcommand = true,
    arg_required_else_help = false
)]
struct Cli {}

fn main() -> Result<()> {
    if std::env::var_os("RUST_LOG").is_some() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    // Keep path deps linked (workspace architecture smoke).
    let _ = (
        dare_core::validate_nonempty_name("cli"),
        dare_contracts::schema_version(),
        dare_config::config_layer_ping("cli"),
        dare_assets::assets_layer_ping("cli"),
    );

    if std::env::args_os().len() <= 1 {
        Cli::command().print_help()?;
        println!();
        return Ok(());
    }

    let _cli = Cli::parse();
    Ok(())
}
