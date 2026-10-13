//! Console output for `dare init` — same text as the npm CLI (chalk colors).

use std::io::Write;

/// chalk styles used by the npm generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Plain,
    Blue,
    BlueBold,
    Cyan,
    Green,
    Yellow,
    Gray,
    Red,
}

impl Tone {
    fn ansi(self) -> Option<&'static str> {
        match self {
            Tone::Plain => None,
            Tone::Blue => Some("\x1b[34m"),
            Tone::BlueBold => Some("\x1b[34m\x1b[1m"),
            Tone::Cyan => Some("\x1b[36m"),
            Tone::Green => Some("\x1b[32m"),
            Tone::Yellow => Some("\x1b[33m"),
            Tone::Gray => Some("\x1b[90m"),
            Tone::Red => Some("\x1b[31m"),
        }
    }
}

/// Sink for `console.log` lines.
pub trait Console {
    fn log(&mut self, tone: Tone, text: &str);
}

/// Writes to stdout (or stderr when stdout is reserved for `--json`).
pub struct StdConsole {
    color: bool,
    to_stderr: bool,
}

impl StdConsole {
    pub fn new(color: bool, to_stderr: bool) -> Self {
        Self { color, to_stderr }
    }
}

impl Console for StdConsole {
    fn log(&mut self, tone: Tone, text: &str) {
        let line = match (self.color, tone.ansi()) {
            (true, Some(code)) => {
                // chalk re-applies styles per line for multi-line strings.
                text.split('\n')
                    .map(|l| {
                        if l.is_empty() {
                            String::new()
                        } else {
                            format!("{code}{l}\x1b[0m")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            _ => text.to_string(),
        };
        if self.to_stderr {
            let _ = writeln!(std::io::stderr(), "{line}");
        } else {
            let _ = writeln!(std::io::stdout(), "{line}");
        }
    }
}

/// Captures lines (tests, `--json`).
#[derive(Debug, Default)]
pub struct MemoryConsole {
    pub lines: Vec<String>,
}

impl Console for MemoryConsole {
    fn log(&mut self, _tone: Tone, text: &str) {
        self.lines.push(text.to_string());
    }
}

/// npm `banner()` in stack-bootstrap.ts.
pub fn banner(console: &mut dyn Console, msg: &str) {
    console.log(Tone::BlueBold, &format!("\n📦 {msg}\n"));
}
