//! Redaction helpers for enrichment logs and error messages.

use dare_core::redact;

use crate::PROMPT_LOG_MAX;

const STDERR_ERROR_CAP: usize = 512;

pub fn redact_prompt_for_log(prompt: &str) -> String {
    let truncated = truncate_chars(prompt, PROMPT_LOG_MAX);
    redact(&truncated)
}

pub fn redact_stderr_for_error(stderr: &str) -> String {
    let truncated = truncate_chars(stderr, STDERR_ERROR_CAP);
    redact(&truncated)
}

fn truncate_chars(input: &str, max: usize) -> String {
    if input.chars().count() <= max {
        return input.to_string();
    }
    let prefix: String = input.chars().take(max).collect();
    format!("{prefix}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_stderr_hides_api_key_in_error_display() {
        let raw = "failed: api_key=secret value here";
        let redacted = redact_stderr_for_error(raw);
        assert!(!redacted.contains("secret"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn redact_prompt_truncates_to_prompt_log_max() {
        let prompt = "x".repeat(PROMPT_LOG_MAX + 50);
        let logged = redact_prompt_for_log(&prompt);
        assert!(logged.chars().count() <= PROMPT_LOG_MAX + 1);
    }
}
