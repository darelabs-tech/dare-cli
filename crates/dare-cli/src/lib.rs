//! Shared library surface for `dare` and `dare-mcp-server` binaries.

pub mod rest_serve;

pub use rest_serve::{run_rest_server, RestServerOpts, MSG_ALIAS_DEPRECATED};
