//! Real CLI agent drivers — argv overrides and shared finalize (microplano 031).

pub mod antigravity;
pub mod argv;
pub mod claude;
pub mod codex;
pub mod common;
pub mod cursor;
mod text_cli;

pub use antigravity::AntigravityDriver;
pub use argv::{parse_argv_override, ENV_ANTIGRAVITY, ENV_CLAUDE, ENV_CODEX, ENV_CURSOR};
pub use claude::ClaudeDriver;
pub use codex::CodexDriver;
pub use common::{
    executable_not_found, finalize_result, AGENT_DRIVER_TIMEOUT, MSG_MALFORMED, SUMMARY_MAX_CHARS,
};
pub use cursor::CursorDriver;
