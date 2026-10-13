//! stdio transport — default for local agents.
//!
//! Uses rmcp's stdio server. The exact serve call may differ across rmcp
//! versions; adjust here if `cargo build` fails after an SDK bump.
use rmcp::ServiceExt;

use crate::server::build_service;

pub async fn run() -> anyhow::Result<()> {
    let service = build_service();
    let server = service.serve((tokio::io::stdin(), tokio::io::stdout())).await?;
    server.waiting().await?;
    Ok(())
}
