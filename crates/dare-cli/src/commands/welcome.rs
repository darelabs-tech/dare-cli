//! `dare welcome` — banner + quick-start (microplano 016).

use std::io::IsTerminal;

const BANNER: &str = r#"
 ____    _    ____  _____
|  _ \  / \  |  _ \| ____|
| | | |/ _ \ | |_) |  _|
| |_| / ___ \|  _ <| |___
|____/_/   \_\_| \_\_____|
"#;

const BANNER_PLAIN: &str = "DARE Framework\n";

const QUICK_START: &str = r#"Quick start — Design → Architecture → Review → Execute

  1. dare design          # /dare-design — requisitos em DARE/DESIGN.md
  2. dare blueprint       # /dare-blueprint — arquitetura + tasks
  3. dare tasks           # /dare-tasks — TASKS.md + dare-dag.yaml
  4. dare execute         # /dare-dag-run-parallel — Ralph Loop

Also useful:
  dare info               # instalação e caminhos
  dare harness claude detect
  dare assets verify

Docs: CLAUDE.md · DARE/ · https://github.com/dewtech/dare-cli
"#;

#[derive(Debug, Clone, Default)]
pub struct WelcomeOptions {
    pub no_banner: bool,
    /// When set, overrides TTY detection (tests).
    pub stdout_is_tty: Option<bool>,
    pub no_color: bool,
}

fn env_no_banner() -> bool {
    matches!(
        std::env::var("DARE_NO_BANNER").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    )
}

fn detect_tty(opts: &WelcomeOptions) -> bool {
    opts.stdout_is_tty
        .unwrap_or_else(|| std::io::stdout().is_terminal())
}

fn should_show_banner(opts: &WelcomeOptions) -> bool {
    if opts.no_banner || env_no_banner() {
        return false;
    }
    detect_tty(opts)
}

/// Render welcome text (human). Never mentions nonexistent `dare new`.
pub fn render_welcome(opts: &WelcomeOptions) -> String {
    let mut out = String::new();
    if should_show_banner(opts) {
        if opts.no_color || std::env::var_os("NO_COLOR").is_some() {
            out.push_str(BANNER_PLAIN);
        } else {
            out.push_str(BANNER.trim_start_matches('\n'));
            out.push('\n');
        }
        out.push_str("Native Rust rewrite — Design → Architecture → Review → Execute\n\n");
    }
    out.push_str(QUICK_START);
    debug_assert!(
        !out.contains("dare new"),
        "welcome must not mention nonexistent dare new"
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_tty_skips_banner() {
        let text = render_welcome(&WelcomeOptions {
            stdout_is_tty: Some(false),
            ..Default::default()
        });
        assert!(!text.contains("____"));
        assert!(text.contains("Quick start"));
        assert!(!text.contains("dare new"));
    }

    #[test]
    fn no_banner_flag() {
        let text = render_welcome(&WelcomeOptions {
            no_banner: true,
            stdout_is_tty: Some(true),
            ..Default::default()
        });
        assert!(!text.contains("____"));
        assert!(text.contains("dare design"));
    }

    #[test]
    fn snapshot_human_tty_no_color() {
        let text = render_welcome(&WelcomeOptions {
            stdout_is_tty: Some(true),
            no_color: true,
            ..Default::default()
        });
        assert_eq!(
            text,
            format!(
                "{BANNER_PLAIN}Native Rust rewrite — Design → Architecture → Review → Execute\n\n{QUICK_START}"
            )
        );
    }

    #[test]
    fn snapshot_no_tty() {
        let text = render_welcome(&WelcomeOptions {
            stdout_is_tty: Some(false),
            ..Default::default()
        });
        assert_eq!(text, QUICK_START);
    }
}
