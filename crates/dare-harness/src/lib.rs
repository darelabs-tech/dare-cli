//! IDE harness adapters (Claude, Cursor, Codex, Antigravity).

pub mod claude;

pub use claude::{
    detect_claude, generate_claude_md, install_commands, validate_install, write_settings_json,
    ClaudeDetect,
};
