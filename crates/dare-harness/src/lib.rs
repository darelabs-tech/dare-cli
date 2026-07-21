//! IDE harness adapters (Claude, Cursor, Codex, Antigravity).

pub mod antigravity;
pub mod claude;
pub mod codex;
pub mod cursor;
pub mod managed;

pub use antigravity::{
    detect_antigravity, ensure_workflows_dir, generate_antigravityrules, install_antigravity,
    validate_antigravity_install, validate_skill_frontmatter, AntigravityDetect,
};
pub use claude::{
    detect_claude, generate_claude_md, install_commands, validate_install, write_settings_json,
    ClaudeDetect,
};
pub use codex::{
    detect_codex, generate_agents_md, install_codex_skills, update_policies_include_codex,
    validate_codex_install, CodexDetect, UPDATE_HARNESS_IDES,
};
pub use cursor::{
    detect_cursor, generate_cursorrules, install_cursor_commands, validate_cursor_install,
    CursorDetect,
};
pub use managed::content_is_managed;
