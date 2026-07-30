//! `dare welcome` — banner + quick-start (microplano 016).
//!
//! Color banner uses DareLabs identity (white fill + electric blue outline) when TTY + color OK.

use std::io::IsTerminal;

/// ANSI: DareLabs electric blue (#1E6BFF) + bright white.
const C_BLUE: &str = "\x1b[38;2;30;107;255m";
const C_WHITE: &str = "\x1b[38;2;255;255;255m";
const C_MUTED: &str = "\x1b[38;2;148;163;184m";
const C_RESET: &str = "\x1b[0m";

/// Plain brand line under --no-color / NO_COLOR.
const BANNER_PLAIN: &str = "DARE\n";

/// Block “DARE” glyph rows — 6 × 52 cols. `█` fill white; box-drawing = blue contour.
const DARE_GLYPH: [&str; 6] = [
    "      ██████╗  █████╗ ██████╗ ███████╗              ",
    "      ██╔══██╗██╔══██╗██╔══██╗██╔════╝              ",
    "      ██║  ██║███████║██████╔╝█████╗                ",
    "      ██║  ██║██╔══██║██╔══██╗██╔══╝                ",
    "      ██████╔╝██║  ██║██║  ██║███████╗              ",
    "      ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝              ",
];

/// Monochrome twin of the colored banner (docs / visual reference).
#[allow(dead_code)]
const BANNER_ART: &str = concat!(
    "╔════════════════════════════════════════════════════╗\n",
    "║                                                    ║\n",
    "║      ██████╗  █████╗ ██████╗ ███████╗              ║\n",
    "║      ██╔══██╗██╔══██╗██╔══██╗██╔════╝              ║\n",
    "║      ██║  ██║███████║██████╔╝█████╗                ║\n",
    "║      ██║  ██║██╔══██║██╔══██╗██╔══╝                ║\n",
    "║      ██████╔╝██║  ██║██║  ██║███████╗              ║\n",
    "║      ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝              ║\n",
    "║                                                    ║\n",
    "║  Discover · Design · Architect · Review · Execute  ║\n",
    "║                                                    ║\n",
    "╚════════════════════════════════════════════════════╝\n",
);

const TAGLINE: &str = "Native Rust rewrite — Design → Architecture → Review → Execute\n\n";

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
    /// When set, overrides `NO_COLOR` / `no_color` for color path (tests).
    pub force_color: Option<bool>,
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

fn color_allowed(opts: &WelcomeOptions) -> bool {
    if let Some(force) = opts.force_color {
        return force;
    }
    !opts.no_color && std::env::var_os("NO_COLOR").is_none()
}

/// Paint figlet-style glyphs: solid `█` white, contour box-drawing blue.
fn paint_dare_glyph(line: &str) -> String {
    let mut out = String::with_capacity(line.len() * 8);
    for ch in line.chars() {
        if ch == '█' {
            out.push_str(C_WHITE);
            out.push(ch);
            out.push_str(C_RESET);
        } else if "╔╗╚╝║═".contains(ch) {
            out.push_str(C_BLUE);
            out.push(ch);
            out.push_str(C_RESET);
        } else {
            out.push(ch);
        }
    }
    out
}

fn frame_row(inner: &str) -> String {
    format!("{C_BLUE}║{C_RESET}{inner}{C_BLUE}║{C_RESET}\n")
}

/// Colored banner: white fill + blue letter contours + blue frame (no LABS).
fn banner_colored() -> String {
    let blank = " ".repeat(52);
    let method = format!(
        "  {C_MUTED}Discover · Design · Architect · Review · Execute{C_RESET}  ",
    );

    let mut out = String::new();
    out.push_str(&format!(
        "{C_BLUE}╔════════════════════════════════════════════════════╗{C_RESET}\n"
    ));
    out.push_str(&frame_row(&blank));
    for row in DARE_GLYPH {
        out.push_str(&frame_row(&paint_dare_glyph(row)));
    }
    out.push_str(&frame_row(&blank));
    out.push_str(&frame_row(&method));
    out.push_str(&frame_row(&blank));
    out.push_str(&format!(
        "{C_BLUE}╚════════════════════════════════════════════════════╝{C_RESET}\n"
    ));
    out
}

/// Render welcome text (human). Never mentions nonexistent `dare new`.
pub fn render_welcome(opts: &WelcomeOptions) -> String {
    let mut out = String::new();
    if should_show_banner(opts) {
        if color_allowed(opts) {
            out.push_str(&banner_colored());
        } else {
            out.push_str(BANNER_PLAIN);
        }
        out.push_str(TAGLINE);
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
        assert!(!text.contains('█'));
        assert!(!text.starts_with("DARE\n"));
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
        assert!(!text.contains('█'));
        assert!(text.contains("dare design"));
    }

    #[test]
    fn snapshot_human_tty_no_color() {
        let text = render_welcome(&WelcomeOptions {
            stdout_is_tty: Some(true),
            no_color: true,
            ..Default::default()
        });
        assert_eq!(text, format!("{BANNER_PLAIN}{TAGLINE}{QUICK_START}"));
    }

    #[test]
    fn snapshot_no_tty() {
        let text = render_welcome(&WelcomeOptions {
            stdout_is_tty: Some(false),
            ..Default::default()
        });
        assert_eq!(text, QUICK_START);
    }

    #[test]
    fn colored_banner_has_darelabs_palette_and_method_line() {
        let text = render_welcome(&WelcomeOptions {
            stdout_is_tty: Some(true),
            no_color: false,
            force_color: Some(true),
            ..Default::default()
        });
        assert!(text.contains("30;107;255")); // DareLabs blue
        assert!(text.contains('█'));
        assert!(!text.contains("L A B S"));
        assert!(!text.contains("LABS"));
        assert!(!text.contains('▲'));
        assert!(text.contains("Discover · Design · Architect · Review · Execute"));
        assert!(text.contains(C_RESET));
        assert!(!text.contains("dare new"));
    }

    #[test]
    fn banner_art_constant_is_well_formed() {
        assert!(BANNER_ART.contains('█'));
        assert!(!BANNER_ART.contains("L A B S"));
        assert!(BANNER_ART.contains("Discover"));
    }

    #[test]
    fn dare_glyph_rows_are_52_cols() {
        for row in DARE_GLYPH {
            assert_eq!(row.chars().count(), 52, "bad width: {row:?}");
        }
    }
}
