//! Enrich prompt builder and redacted preview (BLUEPRINT-050 §4.4 / §5.2).

use serde::{Deserialize, Serialize};

use crate::redact_log::redact_prompt_for_log;
use crate::request::EnrichRequest;

/// Frozen JSON schema version for [`PromptReport`].
pub const PROMPT_SCHEMA_VERSION: u32 = 1;

const MARKDOWN_PROMPT_MAX: usize = 32 * 1024;

/// Prompt preview report (`schemaVersion` 1, camelCase JSON).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptReport {
    pub schema_version: u32,
    pub command: String,
    pub provider: String,
    pub prompt_preview: String,
    pub prompt_chars: usize,
    pub env_leaked: bool,
}

/// Build the enrich prompt text for the given section ids (shared JSON schema hint).
///
/// Does **not** read environment variables (`DARE_*_COMMAND`, `PATH`, tokens, etc.).
pub fn build_enrich_prompt(req: &EnrichRequest, section_ids: &[&str]) -> String {
    let markdown = truncate_bytes(&req.current_markdown, MARKDOWN_PROMPT_MAX);
    let schema_inner = section_ids
        .iter()
        .map(|id| format!("\"{id}\":\"...\""))
        .collect::<Vec<_>>()
        .join(",");
    let sections_hint = section_ids.join(", ");

    format!(
        "You are enriching a DARE {command} document.\n\
         Respond with a single JSON object matching this schema:\n\
         {{\"sections\":{{{schema_inner}}}}}\n\
         Required section ids: {sections_hint}\n\
         Each section value must be a non-empty markdown string (tables where appropriate).\n\
         Do not include AGENT marker comments in section bodies.\n\
         Do not include secrets or credentials.\n\n\
         Command: {command}\n\
         Title: {title}\n\
         Description: {description}\n\n\
         Current markdown (may be truncated):\n\
         {markdown}\n",
        command = req.command,
        title = req.title,
        description = req.description,
        markdown = markdown,
        schema_inner = schema_inner,
        sections_hint = sections_hint,
    )
}

/// Build a redacted prompt preview report. `envLeaked` is always `false`.
///
/// `provider` is left empty for the CLI to fill; this helper never reads env.
pub fn prompt_preview(req: &EnrichRequest, section_ids: &[&str]) -> PromptReport {
    let full = build_enrich_prompt(req, section_ids);
    let prompt_chars = full.chars().count();
    PromptReport {
        schema_version: PROMPT_SCHEMA_VERSION,
        command: req.command.clone(),
        provider: String::new(),
        prompt_preview: redact_prompt_for_log(&full),
        prompt_chars,
        env_leaked: false,
    }
}

fn truncate_bytes(input: &str, max_bytes: usize) -> &str {
    if input.len() <= max_bytes {
        return input;
    }
    let mut end = max_bytes;
    while end > 0 && !input.is_char_boundary(end) {
        end -= 1;
    }
    &input[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ENRICHABLE, ENV_CODEX, PROMPT_LOG_MAX};

    fn sample_req(markdown: &str) -> EnrichRequest {
        EnrichRequest {
            command: "design".to_string(),
            title: "Payments API".to_string(),
            description: "Stripe checkout".to_string(),
            current_markdown: markdown.to_string(),
            cwd: None,
        }
    }

    #[test]
    fn prompt_no_env_leak() {
        crate::with_env_lock(|| {
            let secret = "super-secret-codex-override-xyz-9f3a";
            let prev_codex = std::env::var(ENV_CODEX).ok();
            let prev_path = std::env::var("PATH").ok();
            std::env::set_var(ENV_CODEX, format!("{secret} --flag"));
            std::env::set_var("PATH", format!("/leaked/path/bin:{secret}"));

            let report = prompt_preview(&sample_req("# md"), ENRICHABLE);

            assert!(!report.env_leaked);
            assert!(!report.prompt_preview.contains(secret));
            assert!(!report.prompt_preview.contains("/leaked/path/bin"));
            let full = build_enrich_prompt(&sample_req("# md"), ENRICHABLE);
            assert!(!full.contains(secret));
            assert!(!full.contains("/leaked/path/bin"));

            match prev_codex {
                Some(v) => std::env::set_var(ENV_CODEX, v),
                None => std::env::remove_var(ENV_CODEX),
            }
            match prev_path {
                Some(v) => std::env::set_var("PATH", v),
                None => std::env::remove_var("PATH"),
            }
        });
    }

    #[test]
    fn prompt_truncates() {
        let long = "x".repeat(PROMPT_LOG_MAX + 500);
        let report = prompt_preview(&sample_req(&long), ENRICHABLE);
        assert!(report.prompt_chars > PROMPT_LOG_MAX);
        assert!(report.prompt_preview.chars().count() <= PROMPT_LOG_MAX + 1);
        assert!(!report.env_leaked);
        assert_eq!(report.schema_version, PROMPT_SCHEMA_VERSION);
    }

    #[test]
    fn prompt_includes_command_and_sections_hint() {
        let full = build_enrich_prompt(&sample_req("# body"), ENRICHABLE);
        assert!(full.contains("Command: design"));
        for id in ENRICHABLE {
            assert!(full.contains(id), "missing section id {id}");
        }
        let report = prompt_preview(&sample_req("# body"), ENRICHABLE);
        assert_eq!(report.command, "design");
        assert!(report.prompt_preview.contains("design") || report.command == "design");
    }
}
