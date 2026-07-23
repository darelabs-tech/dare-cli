//! Real CLI agent drivers — argv overrides and shared finalize (microplano 031).

pub mod argv;
pub mod codex;
pub mod common;

pub use argv::{parse_argv_override, ENV_ANTIGRAVITY, ENV_CLAUDE, ENV_CODEX, ENV_CURSOR};
pub use codex::CodexDriver;
pub use common::{
    executable_not_found, finalize_result, AGENT_DRIVER_TIMEOUT, MSG_MALFORMED, SUMMARY_MAX_CHARS,
};
