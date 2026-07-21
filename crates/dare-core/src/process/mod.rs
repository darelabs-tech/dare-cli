//! Secure process execution: argv-only spawn, env denylist, timeout, mock.

mod command;
mod env;
mod kill;
mod mock;
mod output;
mod runner;

pub use command::{CancelFlag, CwdSpec, SafeCommand};
pub use env::{env_key_is_denied, sanitize_env};
pub use mock::MockProcessRunner;
pub use output::{truncate_chars, ProcessOutput, DEFAULT_STREAM_LIMIT};
pub use runner::{ProcessRunner, SystemProcessRunner};
