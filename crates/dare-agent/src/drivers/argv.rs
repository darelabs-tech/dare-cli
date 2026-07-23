//! Env argv override parsing for agent CLI drivers (microplano 031).

use dare_core::{CoreError, CoreResult};

pub const ENV_CODEX: &str = "DARE_CODEX_COMMAND";
pub const ENV_CLAUDE: &str = "DARE_CLAUDE_COMMAND";
pub const ENV_CURSOR: &str = "DARE_CURSOR_COMMAND";
pub const ENV_ANTIGRAVITY: &str = "DARE_ANTIGRAVITY_COMMAND";

/// Parse a whitespace-separated command override into `(program, args)`.
///
/// Empty / whitespace-only input → [`CoreError::InvalidInput`].
pub fn parse_argv_override(env_val: &str) -> CoreResult<(String, Vec<String>)> {
    let trimmed = env_val.trim();
    if trimmed.is_empty() {
        return Err(CoreError::invalid_input(
            "command override must not be empty",
        ));
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let program = parts[0].to_string();
    let args = parts[1..].iter().map(|s| (*s).to_string()).collect();
    Ok((program, args))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_argv_override_empty_errs() {
        let err = parse_argv_override("").unwrap_err();
        assert!(
            err.to_string()
                .contains("command override must not be empty"),
            "msg={}",
            err
        );
        let err = parse_argv_override("   \t  ").unwrap_err();
        assert!(err
            .to_string()
            .contains("command override must not be empty"));
    }

    #[test]
    fn parse_argv_override_codex_exec_json() {
        let (program, args) = parse_argv_override("codex exec --json").unwrap();
        assert_eq!(program, "codex");
        assert_eq!(args, vec!["exec".to_string(), "--json".to_string()]);
    }
}
