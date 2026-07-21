//! Human / JSON output renderer (microplano 004).

use std::io::{self, Write};

use dare_core::{redact, to_canonical_json_string, CoreError, ExecutionContext};
use serde_json::{json, Value};

pub struct OutputRenderer<'a> {
    pub ctx: &'a ExecutionContext,
}

impl<'a> OutputRenderer<'a> {
    pub fn new(ctx: &'a ExecutionContext) -> Self {
        Self { ctx }
    }

    /// JSON: success envelope to stdout. Human: message to stdout.
    #[allow(dead_code)] // public API for upcoming commands (005+)
    pub fn write_success(&self, human_message: &str, data: Value) -> Result<(), CoreError> {
        if self.ctx.json {
            let envelope = json!({
                "correlation_id": self.ctx.correlation_id,
                "data": data,
                "ok": true,
            });
            let line = to_canonical_json_string(&envelope)?;
            writeln!(io::stdout(), "{line}").map_err(|e| CoreError::io(e.to_string()))?;
        } else {
            let msg = if self.ctx.color_enabled_for_stdout() {
                human_message.to_string()
            } else {
                strip_ansi(human_message)
            };
            writeln!(io::stdout(), "{msg}").map_err(|e| CoreError::io(e.to_string()))?;
        }
        Ok(())
    }

    /// JSON: error envelope to stdout. Human: message to stderr.
    /// Returns process exit code.
    pub fn write_error(&self, err: &CoreError) -> i32 {
        let message = redact(err.message());
        if self.ctx.json {
            let envelope = json!({
                "correlation_id": self.ctx.correlation_id,
                "error": {
                    "kind": err.kind().as_str(),
                    "message": message,
                },
                "ok": false,
            });
            match to_canonical_json_string(&envelope) {
                Ok(line) => {
                    let _ = writeln!(io::stdout(), "{line}");
                }
                Err(e) => {
                    let _ = writeln!(io::stderr(), "internal json error: {}", e.message());
                }
            }
        } else {
            let line = if self.ctx.color_enabled_for_stderr() {
                format!("error: {message}")
            } else {
                format!("error: {}", strip_ansi(&message))
            };
            let _ = writeln!(io::stderr(), "{line}");
        }
        err.exit_code()
    }
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for x in chars.by_ref() {
                    if x.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}
