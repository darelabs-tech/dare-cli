//! DARE Framework CLI (native Rust rewrite) — output foundation (microplano 004).

mod output;

use std::process::ExitCode;

use clap::Parser;
use dare_core::{init_tracing, CoreError, ExecutionContext};
use output::OutputRenderer;

#[derive(Debug, Parser)]
#[command(
    name = "dare",
    version,
    about = "DARE Framework CLI (native Rust rewrite)",
    disable_help_subcommand = true,
    arg_required_else_help = false
)]
struct Cli {
    /// Emit JSON envelopes on stdout (ADR-002).
    #[arg(long, global = true)]
    json: bool,

    /// Disable ANSI colors (also honors NO_COLOR).
    #[arg(long, global = true)]
    no_color: bool,
}

fn main() -> ExitCode {
    // Keep path deps linked (workspace architecture smoke).
    let _ = (
        dare_core::validate_nonempty_name("cli"),
        dare_contracts::schema_version(),
        dare_config::config_layer_ping("cli"),
        dare_assets::assets_layer_ping("cli"),
    );

    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();

    // Pre-scan flags for context even when clap parse fails later.
    let json = args.iter().any(|a| a == "--json");
    let no_color = args.iter().any(|a| a == "--no-color");
    let ctx = ExecutionContext::from_cli(json, no_color);
    let _ = init_tracing(&ctx);
    let renderer = OutputRenderer::new(&ctx);

    if args.len() <= 1 {
        use clap::CommandFactory;
        let mut cmd = Cli::command();
        if let Err(e) = cmd.print_help() {
            let code = renderer.write_error(&CoreError::io(e.to_string()));
            return exit(code);
        }
        println!();
        return ExitCode::SUCCESS;
    }

    match Cli::try_parse() {
        Ok(_cli) => ExitCode::SUCCESS,
        Err(e) => {
            // Help/version are clap "errors" that should still succeed.
            if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                let _ = e.print();
                return ExitCode::SUCCESS;
            }
            let msg = e.to_string();
            let code = renderer.write_error(&CoreError::usage(msg));
            exit(code)
        }
    }
}

fn exit(code: i32) -> ExitCode {
    ExitCode::from(code as u8)
}
