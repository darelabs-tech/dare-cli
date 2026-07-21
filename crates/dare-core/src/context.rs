//! Execution context for a single CLI invocation.

use std::time::Instant;

use uuid::Uuid;

/// ANSI color policy for human output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Never,
    Always,
}

/// Per-invocation runtime context (correlation, output mode, TTY).
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub correlation_id: String,
    pub json: bool,
    pub color: ColorMode,
    pub stdout_is_terminal: bool,
    pub stderr_is_terminal: bool,
    pub started_at: Instant,
}

impl ExecutionContext {
    /// Build from CLI flags + environment.
    /// `NO_COLOR` set to any value ⇒ [`ColorMode::Never`].
    /// `--no-color` ⇒ Never. Else Auto.
    pub fn from_cli(json: bool, no_color: bool) -> Self {
        let color = if no_color || std::env::var_os("NO_COLOR").is_some() {
            ColorMode::Never
        } else {
            ColorMode::Auto
        };
        Self {
            correlation_id: Uuid::new_v4().to_string(),
            json,
            color,
            stdout_is_terminal: atty_stdout(),
            stderr_is_terminal: atty_stderr(),
            started_at: Instant::now(),
        }
    }

    /// Test/injectable constructor.
    pub fn new_for_test(
        json: bool,
        color: ColorMode,
        stdout_is_terminal: bool,
        stderr_is_terminal: bool,
    ) -> Self {
        Self {
            correlation_id: Uuid::new_v4().to_string(),
            json,
            color,
            stdout_is_terminal,
            stderr_is_terminal,
            started_at: Instant::now(),
        }
    }

    pub fn color_enabled_for_stdout(&self) -> bool {
        match self.color {
            ColorMode::Never => false,
            ColorMode::Always => true,
            ColorMode::Auto => self.stdout_is_terminal,
        }
    }

    pub fn color_enabled_for_stderr(&self) -> bool {
        match self.color {
            ColorMode::Never => false,
            ColorMode::Always => true,
            ColorMode::Auto => self.stderr_is_terminal,
        }
    }
}

fn atty_stdout() -> bool {
    anstream_is_terminal(true)
}

fn atty_stderr() -> bool {
    anstream_is_terminal(false)
}

fn anstream_is_terminal(stdout: bool) -> bool {
    // Avoid depending on anstream in core: use std heuristic via is_terminal on stdio.
    use std::io::IsTerminal;
    if stdout {
        std::io::stdout().is_terminal()
    } else {
        std::io::stderr().is_terminal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_cli_no_color_flag_never() {
        let ctx = ExecutionContext::from_cli(false, true);
        assert_eq!(ctx.color, ColorMode::Never);
        assert!(!ctx.color_enabled_for_stdout());
        assert!(!ctx.color_enabled_for_stderr());
    }

    #[test]
    fn correlation_id_is_uuid_v4_shape() {
        let ctx = ExecutionContext::new_for_test(false, ColorMode::Never, false, false);
        let parts: Vec<_> = ctx.correlation_id.split('-').collect();
        assert_eq!(parts.len(), 5, "{}", ctx.correlation_id);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }

    #[test]
    fn auto_respects_tty_flags() {
        let ctx = ExecutionContext::new_for_test(true, ColorMode::Auto, true, false);
        assert!(ctx.color_enabled_for_stdout());
        assert!(!ctx.color_enabled_for_stderr());
    }
}
