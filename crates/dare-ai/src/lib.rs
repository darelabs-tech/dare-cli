//! AI enrichment domain (microplano 024).

mod codex;
mod inject;
mod mock;
mod provider;
mod redact_log;
mod request;
mod schema;

use std::time::Duration;

pub use codex::{parse_argv_override, CodexCliProvider};
pub use inject::inject_enrichable;
pub use mock::MockProvider;
pub use provider::{resolve_provider, AiProvider, ProviderId};
pub use redact_log::{redact_prompt_for_log, redact_stderr_for_error};
pub use request::{EnrichRaw, EnrichRequest};
pub use schema::parse_and_validate_sections;

pub const ENRICH_TIMEOUT: Duration = Duration::from_secs(20 * 60);
pub const STDOUT_CAP: usize = 1_048_576;
pub const STDERR_CAP: usize = 65_536;
pub const BODY_MAX: usize = 65_536;
pub const PROMPT_LOG_MAX: usize = 256;
pub const ENRICHABLE: &[&str] = &[
    "description",
    "objectives",
    "functional-requirements",
    "stack",
];
pub const MARKER_BEGIN: &str = "<!-- AGENT:BEGIN section=\"";
pub const MARKER_END_PREFIX: &str = "<!-- AGENT:END section=\"";

pub const ENV_CODEX: &str = "DARE_CODEX_COMMAND";
pub const ENV_CLAUDE: &str = "DARE_CLAUDE_COMMAND";
pub const ENV_CURSOR: &str = "DARE_CURSOR_COMMAND";
pub const ENV_ANTIGRAVITY: &str = "DARE_ANTIGRAVITY_COMMAND";
