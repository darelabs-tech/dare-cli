//! Codex CLI provider — argv-only spawn via `SafeCommand`.

use std::sync::Arc;

use dare_core::{
    CoreError, CoreResult, ProcessRunner, ProjectRoot, SafeCommand, SafeRelativePath,
    SystemProcessRunner,
};

use crate::provider::{AiProvider, ProviderId};
use crate::redact_log::{redact_prompt_for_log, redact_stderr_for_error};
use crate::request::{EnrichRaw, EnrichRequest};
use crate::{ENRICH_TIMEOUT, ENV_CODEX, STDERR_CAP, STDOUT_CAP};

const MARKDOWN_PROMPT_MAX: usize = 32 * 1024;

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

pub struct CodexCliProvider {
    program: String,
    base_args: Vec<String>,
    runner: Arc<dyn ProcessRunner>,
}

impl CodexCliProvider {
    pub fn from_env() -> CoreResult<Self> {
        Self::from_env_with_runner(Arc::new(SystemProcessRunner))
    }

    pub fn from_env_with_runner(runner: Arc<dyn ProcessRunner>) -> CoreResult<Self> {
        let (program, base_args) = match std::env::var(ENV_CODEX) {
            Ok(val) => parse_argv_override(&val)?,
            Err(_) => ("codex".to_string(), vec!["exec".to_string()]),
        };
        Ok(Self {
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

impl AiProvider for CodexCliProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Codex
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

    #[test]
    fn argv_override_split() {
        let (program, args) =
            parse_argv_override("  /usr/bin/codex  exec  --model  gpt-4  ").unwrap();
        assert_eq!(program, "/usr/bin/codex");
        assert_eq!(args, vec!["exec", "--model", "gpt-4"]);

        let err = parse_argv_override("   ").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn codex_default_program_is_codex() {
        std::env::remove_var(ENV_CODEX);
        let dir = tempdir().expect("temp");
        let _ = std::fs::create_dir_all(dir.path().join("sub"));
        let root = ProjectRoot::new(dir.path()).expect("root");
        let req = sample_request(root);

        let mock = MockProcessRunner::new();
        mock.push(ProcessOutput {
            exit_code: 0,
            stdout: r#"{"sections":{"description":"d","objectives":"o","functional-requirements":"f","stack":"s"}}"#
                .into(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        });

        let recording = Arc::new(RecordingRunner::new(mock));
        let provider = CodexCliProvider::from_env_with_runner(
            Arc::clone(&recording) as Arc<dyn ProcessRunner>
        )
        .unwrap();
        assert_eq!(provider.id(), ProviderId::Codex);

        provider.enrich(&req).expect("enrich ok");
        let cmd = recording.last_cmd().expect("command recorded");
        assert_eq!(cmd.program(), "codex");
        assert_eq!(cmd.arg_list(), &["exec"]);
    }

    #[test]
    fn codex_builds_command_with_mock_runner() {
        std::env::remove_var(ENV_CODEX);
        let dir = tempdir().expect("temp");
        let _ = std::fs::create_dir_all(dir.path().join("sub"));
        let root = ProjectRoot::new(dir.path()).expect("root");
        let req = sample_request(root);

        let mock = MockProcessRunner::new();
        mock.push(ProcessOutput {
            exit_code: 0,
            stdout: r#"{"sections":{"description":"d","objectives":"o","functional-requirements":"f","stack":"s"}}"#
                .into(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        });

        let recording = Arc::new(RecordingRunner::new(mock));
        let provider = CodexCliProvider::from_env_with_runner(
            Arc::clone(&recording) as Arc<dyn ProcessRunner>
        )
        .unwrap();

        let raw = provider.enrich(&req).expect("enrich ok");
        assert_eq!(raw.exit_code, 0);
        assert!(raw.stdout.contains("sections"));

        let cmd = recording.last_cmd().expect("command recorded");
        assert_eq!(cmd.program(), "codex");
        assert_eq!(cmd.arg_list(), &["exec"]);
        assert!(cmd.stdin_bytes().is_some());
        let stdin = cmd.stdin_bytes().unwrap();
        assert!(std::str::from_utf8(stdin).unwrap().contains("Payment API"));
    }

    #[test]
    fn codex_timeout_returns_err() {
        std::env::remove_var(ENV_CODEX);
        let dir = tempdir().expect("temp");
        let _ = std::fs::create_dir_all(dir.path().join("sub"));
        let root = ProjectRoot::new(dir.path()).expect("root");
        let req = sample_request(root);

        let mock = MockProcessRunner::new();
        mock.push(ProcessOutput {
            exit_code: 124,
            stdout: String::new(),
            stderr: "timed out".into(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: true,
            cancelled: false,
        });

        let provider =
            CodexCliProvider::from_env_with_runner(Arc::new(mock) as Arc<dyn ProcessRunner>)
                .unwrap();

        let err = provider.enrich(&req).expect_err("timeout");
        assert_eq!(err.kind(), ErrorKind::Internal);
        assert!(err.to_string().contains("timed out"));
    }

    #[test]
    fn codex_nonzero_exit_redacts_stderr() {
        std::env::remove_var(ENV_CODEX);
        let dir = tempdir().expect("temp");
        let _ = std::fs::create_dir_all(dir.path().join("sub"));
        let root = ProjectRoot::new(dir.path()).expect("root");
        let req = sample_request(root);

        let mock = MockProcessRunner::new();
        mock.push(ProcessOutput {
            exit_code: 1,
            stdout: String::new(),
            stderr: "error api_key=topsecret".into(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        });

        let provider =
            CodexCliProvider::from_env_with_runner(Arc::new(mock) as Arc<dyn ProcessRunner>)
                .unwrap();

        let err = provider.enrich(&req).expect_err("nonzero");
        let msg = err.to_string();
        assert!(!msg.contains("topsecret"));
        assert!(msg.contains("[REDACTED]"));
    }
}
