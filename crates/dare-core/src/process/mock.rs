//! Mock ProcessRunner — never spawns OS processes.

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::error::{CoreError, CoreResult};

use super::command::SafeCommand;
use super::output::ProcessOutput;
use super::runner::ProcessRunner;

enum MockResponse {
    Ok(ProcessOutput),
    Err(CoreError),
}

struct MockRule {
    program: Option<String>,
    response: MockResponse,
}

pub struct MockProcessRunner {
    rules: Mutex<VecDeque<MockRule>>,
}

impl Default for MockProcessRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl MockProcessRunner {
    pub fn new() -> Self {
        Self {
            rules: Mutex::new(VecDeque::new()),
        }
    }

    pub fn push(&self, output: ProcessOutput) {
        self.rules.lock().expect("mock lock").push_back(MockRule {
            program: None,
            response: MockResponse::Ok(output),
        });
    }

    pub fn push_err(&self, err: CoreError) {
        self.rules.lock().expect("mock lock").push_back(MockRule {
            program: None,
            response: MockResponse::Err(err),
        });
    }

    pub fn when_program(&self, program: &str, output: ProcessOutput) {
        self.rules.lock().expect("mock lock").push_back(MockRule {
            program: Some(program.to_string()),
            response: MockResponse::Ok(output),
        });
    }
}

impl ProcessRunner for MockProcessRunner {
    fn run(&self, cmd: &SafeCommand) -> CoreResult<ProcessOutput> {
        let mut q = self.rules.lock().expect("mock lock");
        let idx = q.iter().position(|r| match &r.program {
            None => true,
            Some(p) => p == cmd.program(),
        });
        let Some(i) = idx else {
            return Err(CoreError::internal(
                "mock process runner: no response queued",
            ));
        };
        let rule = q.remove(i).expect("index valid");
        match rule.response {
            MockResponse::Ok(o) => Ok(o),
            MockResponse::Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_runner_returns_queued_output_without_spawn() {
        let mock = MockProcessRunner::new();
        mock.push(ProcessOutput {
            exit_code: 0,
            stdout: "ok".into(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        });
        let out = mock.run(&SafeCommand::new("never-runs")).expect("mock ok");
        assert_eq!(out.stdout, "ok");
        assert_eq!(out.exit_code, 0);
    }
}
