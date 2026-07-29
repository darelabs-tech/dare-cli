//! AI enrichment domain (microplano 024).

pub mod command_registry;
mod capabilities;
mod codex;
mod doctor;
mod inject;
mod mock;
pub mod prompt;
mod provider;
mod redact_log;
mod request;
pub mod run;
mod schema;
mod text_cli;

use std::time::Duration;

pub use capabilities::{
    default_program, default_timeout_secs, env_override_name, is_implemented,
    list_provider_capabilities, ProviderCapability, ProvidersReport, PROVIDER_ORDER,
    PROVIDERS_SCHEMA_VERSION,
};
pub use codex::{parse_argv_override, CodexCliProvider};
pub use command_registry::{sections_for_command, MSG_UNKNOWN_COMMAND};
pub use doctor::{
    diagnose_all, diagnose_provider, DoctorReport, DoctorStatus, ProviderDoctorEntry,
    DOCTOR_SCHEMA_VERSION,
};
pub use inject::{inject_enrichable, inject_sections};
pub use mock::MockProvider;
pub use prompt::{build_enrich_prompt, prompt_preview, PromptReport};
pub use provider::{resolve_provider, AiProvider, ProviderId};
pub use text_cli::{
    TextCliConfig, TextCliProvider, ANTIGRAVITY_CFG, CLAUDE_CFG, CURSOR_CFG,
};
pub use redact_log::{redact_prompt_for_log, redact_stderr_for_error};
pub use request::{EnrichRaw, EnrichRequest};
pub use run::{
    run_enrich, RunEnrichRequest, RunReport, AI_REPORT_SCHEMA, MSG_PROVIDER_NOT_IMPL,
    MSG_WRITE_NEEDS_MARKDOWN,
};
pub use schema::{parse_and_validate_sections, parse_and_validate_sections_with};

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

/// Serialize tests that mutate process env (parallel cargo test races).
#[cfg(test)]
pub(crate) fn with_env_lock<R>(f: impl FnOnce() -> R) -> R {
    use std::sync::{Mutex, OnceLock};
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().unwrap_or_else(|e| e.into_inner());
    f()
}
