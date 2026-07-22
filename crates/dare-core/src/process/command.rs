//! SafeCommand builder (argv only — no shell API).

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use crate::path::{ProjectRoot, SafeRelativePath};

pub type CancelFlag = Arc<AtomicBool>;

#[derive(Debug, Clone)]
pub struct CwdSpec {
    pub root: ProjectRoot,
    pub rel: SafeRelativePath,
}

#[derive(Debug, Clone)]
pub struct SafeCommand {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) cwd: Option<CwdSpec>,
    pub(crate) extra_env: Vec<(String, String)>,
    pub(crate) clear_env: bool,
    pub(crate) timeout: Option<Duration>,
    pub(crate) stdout_limit: usize,
    pub(crate) stderr_limit: usize,
    pub(crate) cancel: Option<CancelFlag>,
    pub(crate) stdin: Option<Vec<u8>>,
}

impl SafeCommand {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            extra_env: Vec::new(),
            clear_env: false,
            timeout: None,
            stdout_limit: super::DEFAULT_STREAM_LIMIT,
            stderr_limit: super::DEFAULT_STREAM_LIMIT,
            cancel: None,
            stdin: None,
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn cwd(mut self, root: ProjectRoot, rel: SafeRelativePath) -> Self {
        self.cwd = Some(CwdSpec { root, rel });
        self
    }

    pub fn env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.extra_env.push((key.into(), val.into()));
        self
    }

    pub fn clear_env(mut self, clear: bool) -> Self {
        self.clear_env = clear;
        self
    }

    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = Some(d);
        self
    }

    pub fn stdout_limit(mut self, n: usize) -> Self {
        self.stdout_limit = n;
        self
    }

    pub fn stderr_limit(mut self, n: usize) -> Self {
        self.stderr_limit = n;
        self
    }

    pub fn cancel_flag(mut self, flag: CancelFlag) -> Self {
        self.cancel = Some(flag);
        self
    }

    pub fn stdin(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.stdin = Some(bytes.into());
        self
    }

    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn arg_list(&self) -> &[String] {
        &self.args
    }

    pub fn stdin_bytes(&self) -> Option<&[u8]> {
        self.stdin.as_deref()
    }
}
