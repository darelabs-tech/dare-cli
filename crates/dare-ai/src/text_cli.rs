//! Shared terminal-first CLI enrich providers (Claude / Cursor / Antigravity).
//!
//! Mirrors [`CodexCliProvider`](crate::CodexCliProvider): `SafeCommand` argv-only spawn,
//! `DARE_*_COMMAND` overrides via [`parse_argv_override`](crate::parse_argv_override),
//! stdin prompt, `ENRICH_TIMEOUT`, stdout/stderr caps. No shell.

use std::sync::Arc;

use dare_core::{
    CoreError, CoreResult, ProcessRunner, ProjectRoot, SafeCommand, SafeRelativePath,
    SystemProcessRunner,
};

use crate::codex::parse_argv_override;
use crate::provider::{AiProvider, ProviderId};
use crate::redact_log::{redact_prompt_for_log, redact_stderr_for_error};
use crate::request::{EnrichRaw, EnrichRequest};
use crate::{ENRICH_TIMEOUT, ENV_ANTIGRAVITY, ENV_CLAUDE, ENV_CURSOR, STDERR_CAP, STDOUT_CAP};

const MARKDOWN_PROMPT_MAX: usize = 32 * 1024;

/// Static defaults for a text CLI enrich provider.
pub struct TextCliConfig {
    pub id: ProviderId,
    pub env_key: &'static str,
    pub default_program: &'static str,
    pub default_args: &'static [&'static str],
}

/// Claude Code — default `claude -p --output-format text`.
pub const CLAUDE_CFG: TextCliConfig = TextCliConfig {
    id: ProviderId::ClaudeCode,
    env_key: ENV_CLAUDE,
    default_program: "claude",
    default_args: &["-p", "--output-format", "text"],
};

/// Cursor CLI — default `cursor --print`.
pub const CURSOR_CFG: TextCliConfig = TextCliConfig {
    id: ProviderId::CursorCli,
    env_key: ENV_CURSOR,
    default_program: "cursor",
    default_args: &["--print"],
};

/// Antigravity CLI — default `antigravity agent --print`.
pub const ANTIGRAVITY_CFG: TextCliConfig = TextCliConfig {
    id: ProviderId::AntigravityCli,
    env_key: ENV_ANTIGRAVITY,
    default_program: "antigravity",
    default_args: &["agent", "--print"],
};

/// Generic CLI enrich provider parameterized by [`TextCliConfig`].
pub struct TextCliProvider {
    id: ProviderId,
    program: String,
    base_args: Vec<String>,
    runner: Arc<dyn ProcessRunner>,
}

impl TextCliProvider {
    pub fn from_env(config: &TextCliConfig) -> CoreResult<Self> {
        Self::from_env_with_runner(config, Arc::new(SystemProcessRunner))
    }

    pub fn from_env_with_runner(
        config: &TextCliConfig,
        runner: Arc<dyn ProcessRunner>,
    ) -> CoreResult<Self> {
        let (program, base_args) = match std::env::var(config.env_key) {
            Ok(val) => parse_argv_override(&val)?,
            Err(_) => (
                config.default_program.to_string(),
                config
                    .default_args
                    .iter()
                    .map(|s| (*s).to_string())
                    .collect(),
            ),
        };
        Ok(Self {
            id: config.id,
            program,
            base_args,
            runner,
        })
    }

    fn build_prompt(req: &EnrichRequest) -> String {
        let markdown = truncate_bytes(&req.current_markdown, MARKDOWN_PROMPT_MAX);
        format!(
            "You are enriching a DARE design document section.\n\
             Respond with a single JSON object matching this schema:\n\
             {{\"sections\":{{\"description\":\"...\",\"objectives\":\"...\",\
             \"functional-requirements\":\"...\",\"stack\":\"...\"}}}}\n\
             Each section value must be a non-empty markdown string (tables where appropriate).\n\
             Do not include AGENT marker comments in section bodies.\n\
             Do not include secrets or credentials.\n\n\
             Command: {}\n\
             Title: {}\n\
             Description: {}\n\n\
             Current markdown (may be truncated):\n\
             {}\n",
            req.command, req.title, req.description, markdown
        )
    }

    fn resolve_cwd(req: &EnrichRequest) -> CoreResult<(ProjectRoot, SafeRelativePath)> {
        req.cwd
            .clone()
            .ok_or_else(|| CoreError::invalid_input("enrich requires project cwd"))
    }
}

impl AiProvider for TextCliProvider {
    fn id(&self) -> ProviderId {
        self.id
    }

    fn enrich(&self, req: &EnrichRequest) -> CoreResult<EnrichRaw> {
        let (root, rel) = Self::resolve_cwd(req)?;
        let prompt = Self::build_prompt(req);
        let _log_preview = redact_prompt_for_log(&prompt);

        let cmd = SafeCommand::new(&self.program)
            .args(self.base_args.iter().cloned())
            .stdin(prompt.into_bytes())
            .timeout(ENRICH_TIMEOUT)
            .stdout_limit(STDOUT_CAP)
            .stderr_limit(STDERR_CAP)
            .cwd(root, rel);

        let output = self.runner.run(&cmd)?;

        if output.timed_out || output.exit_code == 124 {
            return Err(CoreError::internal("provider timed out"));
        }

        if output.exit_code != 0 {
            let detail = redact_stderr_for_error(&output.stderr);
            return Err(CoreError::internal(format!(
                "provider exited with code {}: {}",
                output.exit_code, detail
            )));
        }

        Ok(EnrichRaw {
            stdout: output.stdout,
            stderr_redacted: redact_stderr_for_error(&output.stderr),
            exit_code: output.exit_code,
        })
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
    use dare_core::{ErrorKind, MockProcessRunner, ProcessOutput};
    use std::sync::Mutex;
    use tempfile::tempdir;

    struct RecordingRunner {
        inner: MockProcessRunner,
        last_cmd: Mutex<Option<SafeCommand>>,
    }

    impl RecordingRunner {
        fn new(inner: MockProcessRunner) -> Self {
            Self {
                inner,
                last_cmd: Mutex::new(None),
            }
        }

        fn last_cmd(&self) -> Option<SafeCommand> {
            self.last_cmd.lock().expect("lock").clone()
        }
    }

    impl ProcessRunner for RecordingRunner {
        fn run(&self, cmd: &SafeCommand) -> CoreResult<ProcessOutput> {
            *self.last_cmd.lock().expect("lock") = Some(cmd.clone());
            self.inner.run(cmd)
        }
    }

    fn sample_request(root: ProjectRoot) -> EnrichRequest {
        let rel = SafeRelativePath::new("sub").expect("sub rel");
        EnrichRequest {
            command: "design".into(),
            title: "Sample API".into(),
            description: "Payment API".into(),
            current_markdown: "# Design\n".into(),
            cwd: Some((root, rel)),
        }
    }

    fn ok_stdout() -> ProcessOutput {
        ProcessOutput {
            exit_code: 0,
            stdout: r#"{"sections":{"description":"d","objectives":"o","functional-requirements":"f","stack":"s"}}"#
                .into(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        }
    }

    #[test]
    fn resolve_claude_from_env_override() {
        crate::with_env_lock(|| {
            std::env::set_var(ENV_CLAUDE, "/opt/claude-bin -p --extra");
            let mock = MockProcessRunner::new();
            let result = TextCliProvider::from_env_with_runner(
                &CLAUDE_CFG,
                Arc::new(mock) as Arc<dyn ProcessRunner>,
            );
            std::env::remove_var(ENV_CLAUDE);
            let provider = result.expect("from_env");
            assert_eq!(provider.id(), ProviderId::ClaudeCode);
            assert_eq!(provider.program, "/opt/claude-bin");
            assert_eq!(provider.base_args, vec!["-p", "--extra"]);
        });
    }

    #[test]
    fn resolve_cursor_ok_with_fake_runner() {
        crate::with_env_lock(|| {
            std::env::remove_var(ENV_CURSOR);
            let dir = tempdir().expect("temp");
            let _ = std::fs::create_dir_all(dir.path().join("sub"));
            let root = ProjectRoot::new(dir.path()).expect("root");
            let req = sample_request(root);

            let mock = MockProcessRunner::new();
            mock.push(ok_stdout());
            let recording = Arc::new(RecordingRunner::new(mock));
            let provider = TextCliProvider::from_env_with_runner(
                &CURSOR_CFG,
                Arc::clone(&recording) as Arc<dyn ProcessRunner>,
            )
            .unwrap();
            assert_eq!(provider.id(), ProviderId::CursorCli);

            provider.enrich(&req).expect("enrich ok");
            let cmd = recording.last_cmd().expect("command recorded");
            assert_eq!(cmd.program(), "cursor");
            assert_eq!(cmd.arg_list(), &["--print"]);
        });
    }

    #[test]
    fn resolve_antigravity_default_args() {
        crate::with_env_lock(|| {
            std::env::remove_var(ENV_ANTIGRAVITY);
            let dir = tempdir().expect("temp");
            let _ = std::fs::create_dir_all(dir.path().join("sub"));
            let root = ProjectRoot::new(dir.path()).expect("root");
            let req = sample_request(root);

            let mock = MockProcessRunner::new();
            mock.push(ok_stdout());
            let recording = Arc::new(RecordingRunner::new(mock));
            let provider = TextCliProvider::from_env_with_runner(
                &ANTIGRAVITY_CFG,
                Arc::clone(&recording) as Arc<dyn ProcessRunner>,
            )
            .unwrap();
            assert_eq!(provider.id(), ProviderId::AntigravityCli);

            provider.enrich(&req).expect("enrich ok");
            let cmd = recording.last_cmd().expect("command recorded");
            assert_eq!(cmd.program(), "antigravity");
            assert_eq!(cmd.arg_list(), &["agent", "--print"]);
        });
    }

    #[test]
    fn empty_override_is_invalid_input() {
        crate::with_env_lock(|| {
            std::env::set_var(ENV_CLAUDE, "   ");
            let result = TextCliProvider::from_env_with_runner(
                &CLAUDE_CFG,
                Arc::new(MockProcessRunner::new()) as Arc<dyn ProcessRunner>,
            );
            std::env::remove_var(ENV_CLAUDE);
            let err = result.err().expect("empty override");
            assert_eq!(err.kind(), ErrorKind::InvalidInput);
        });
    }
}
