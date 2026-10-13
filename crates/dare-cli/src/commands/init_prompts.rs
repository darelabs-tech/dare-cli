//! Interactive `dare init` prompts — same questions, order, conditions, labels,
//! values and defaults as npm `@dewtech/dare-cli@3.18.1` `commands/init.ts`
//! (cycle rust-init-npm-parity-011).

use std::fmt;

use dare_core::{CoreError, CoreResult};
use dare_scaffold::project::{
    default_crate_prefix, validate_crate_prefix, validate_npm_project_name, BackendStack,
    FrontendStack, GraphragBackend, Ide, McpFeature, McpLanguage, McpTransport,
    RustWorkspaceLayout, Structure, ToolchainMode,
};
use dialoguer::console::style;
use dialoguer::theme::Theme;
use dialoguer::{Confirm, Input, MultiSelect, Select};

pub const MSG_INTERACTIVE_REQUIRES_TTY: &str =
    "interactive mode requires a TTY (use --non-interactive with --stack or --mcp)";
const MSG_PROMPT_CANCELED: &str = "prompt canceled";

/// Abstract prompt backend (dialoguer in production, scripted in tests).
pub trait Prompter {
    fn input(
        &mut self,
        message: &str,
        default: &str,
        validate: &dyn Fn(&str) -> Result<(), String>,
    ) -> CoreResult<String>;
    fn select(&mut self, message: &str, items: &[&str], default: usize) -> CoreResult<usize>;
    fn checkbox(
        &mut self,
        message: &str,
        items: &[&str],
        checked: &[bool],
    ) -> CoreResult<Vec<usize>>;
    fn confirm(&mut self, message: &str, default: bool) -> CoreResult<bool>;
}

/// Answers of the interactive flow (npm `answers` object).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitAnswers {
    pub name: String,
    pub structure: Structure,
    pub backend: Option<BackendStack>,
    pub frontend: Option<FrontendStack>,
    pub mcp_language: Option<McpLanguage>,
    pub mcp_transport: Option<McpTransport>,
    pub mcp_features: Option<Vec<McpFeature>>,
    pub rust_workspace_layout: Option<RustWorkspaceLayout>,
    pub crate_prefix: Option<String>,
    pub ide: Ide,
    pub graphrag: GraphragBackend,
    pub mcp: bool,
    pub toolchain: ToolchainMode,
}

fn choose<T: Copy>(
    p: &mut dyn Prompter,
    message: &str,
    choices: &[(&str, T)],
    default: usize,
) -> CoreResult<T> {
    let labels: Vec<&str> = choices.iter().map(|(l, _)| *l).collect();
    let idx = p.select(message, &labels, default)?;
    choices
        .get(idx)
        .map(|(_, v)| *v)
        .ok_or_else(|| CoreError::invalid_input(MSG_PROMPT_CANCELED))
}

pub const STRUCTURE_CHOICES: &[(&str, Structure)] = &[
    ("🏗️  Monorepo (backend + frontend)", Structure::Monorepo),
    ("⚙️  Backend only", Structure::Backend),
    ("🎨 Frontend only", Structure::Frontend),
    ("🏛️  MVC (Laravel ou Rails — full-stack)", Structure::Mvc),
    ("🔌 MCP Server", Structure::McpServer),
];

pub const MVC_CHOICES: &[(&str, BackendStack)] = &[
    ("💎 Ruby / Rails 8", BackendStack::RubyRails8),
    ("🐘 PHP / Laravel", BackendStack::PhpLaravel),
];

pub const MCP_LANGUAGE_CHOICES: &[(&str, McpLanguage)] = &[
    ("🟦 TypeScript / Node.js", McpLanguage::NodeTs),
    ("🐍 Python", McpLanguage::Python),
    ("🦀 Rust (beta)", McpLanguage::Rust),
    ("🐹 Go (beta)", McpLanguage::Go),
];

pub const MCP_TRANSPORT_CHOICES: &[(&str, McpTransport)] = &[
    ("📟 stdio  (CLI tools, local agents)", McpTransport::Stdio),
    ("🌐 SSE   (web integrations, remote)", McpTransport::Sse),
    ("⚡ HTTP Stream (streamable HTTP)", McpTransport::HttpStream),
];

pub const MCP_FEATURE_CHOICES: &[(&str, McpFeature, bool)] = &[
    ("🔧 Tools     (callable functions)", McpFeature::Tools, true),
    (
        "📦 Resources (readable context)",
        McpFeature::Resources,
        false,
    ),
    (
        "💬 Prompts   (prompt templates)",
        McpFeature::Prompts,
        false,
    ),
];

pub const BACKEND_CHOICES: &[(&str, BackendStack)] = &[
    ("💎 Ruby / Rails 8", BackendStack::RubyRails8),
    ("🦀 Rust / Axum", BackendStack::RustAxum),
    ("🟢 Node.js / NestJS", BackendStack::NodeNestjs),
    ("🐍 Python / FastAPI", BackendStack::PythonFastapi),
    ("🐘 PHP / Laravel", BackendStack::PhpLaravel),
    ("🐹 Go / Gin", BackendStack::GoGin),
    (
        "🐹 Go / stdlib (no framework, net/http only)",
        BackendStack::GoStdlib,
    ),
];

pub const FRONTEND_CHOICES: &[(&str, FrontendStack)] = &[
    ("⚛️  React 18+ (TypeScript)", FrontendStack::React),
    ("💚 Vue 3+ (Composition API)", FrontendStack::Vue),
    (
        "🦀 Leptos fullstack (Rust SSR + WASM)",
        FrontendStack::RustLeptos,
    ),
    (
        "🦀 Leptos CSR-only (Rust WASM + trunk)",
        FrontendStack::RustLeptosCsr,
    ),
    ("🚫 None (backend only)", FrontendStack::None),
];

pub const LAYOUT_CHOICES: &[(&str, RustWorkspaceLayout)] = &[
    (
        "📦 Single-crate  — crates/server + crates/web  (app simples, recomendado para começar)",
        RustWorkspaceLayout::Single,
    ),
    (
        "🏗️  Multi-crate   — {name}-core / {name}-server / {name}-web / {name}-cli  (produto / plataforma)",
        RustWorkspaceLayout::Multi,
    ),
];

pub const IDE_CHOICES: &[(&str, Ide)] = &[
    ("🤖 Claude Code", Ide::ClaudeCode),
    ("⌨️  Codex CLI", Ide::Codex),
    ("🖱️  Cursor", Ide::Cursor),
    ("🚀 Antigravity", Ide::Antigravity),
    ("🔀 Cursor + Antigravity (Hybrid)", Ide::Hybrid),
    ("🔀 Claude Code + Cursor (Hybrid)", Ide::ClaudeHybrid),
];

pub const GRAPHRAG_CHOICES: &[(&str, GraphragBackend)] = &[
    (
        "🗄️  SQLite (recommended - fast, local)",
        GraphragBackend::Sqlite,
    ),
    (
        "📄 JSON Graph (simple, no dependencies)",
        GraphragBackend::Json,
    ),
    ("🐳 Neo4j Docker (enterprise)", GraphragBackend::Neo4j),
];

pub const TOOLCHAIN_CHOICES: &[(&str, ToolchainMode)] = &[
    (
        "🤖 Auto — use native if on PATH, else Docker (recommended)",
        ToolchainMode::Auto,
    ),
    (
        "🔧 Native only — require the CLI on PATH (faster, no Docker pulls)",
        ToolchainMode::Native,
    ),
    (
        "🐳 Docker only — always use the official image (hermetic, no host install)",
        ToolchainMode::Docker,
    ),
];

/// npm `validate` for `Project name:` — returns the message without `Error: `.
fn validate_name_prompt(input: &str) -> Result<(), String> {
    validate_npm_project_name(input).map(|_| ()).map_err(|e| {
        let msg = e.message();
        msg.strip_prefix("Error: ").unwrap_or(msg).to_string()
    })
}

/// The interactive question tree of `commands/init.ts` (lines 37–240).
pub fn prompt_init_answers(
    p: &mut dyn Prompter,
    project_name: Option<&str>,
) -> CoreResult<InitAnswers> {
    let name = match project_name {
        Some(n) => n.to_string(),
        None => p.input("Project name:", "my-dare-project", &validate_name_prompt)?,
    };

    let structure = choose(p, "Project structure:", STRUCTURE_CHOICES, 0)?;

    let mut backend = None;
    if structure == Structure::Mvc {
        backend = Some(choose(p, "MVC framework:", MVC_CHOICES, 0)?);
    }

    let (mut mcp_language, mut mcp_transport, mut mcp_features) = (None, None, None);
    if structure == Structure::McpServer {
        mcp_language = Some(choose(p, "MCP server language:", MCP_LANGUAGE_CHOICES, 0)?);
        mcp_transport = Some(choose(p, "Transport type:", MCP_TRANSPORT_CHOICES, 0)?);
        let labels: Vec<&str> = MCP_FEATURE_CHOICES.iter().map(|(l, _, _)| *l).collect();
        let checked: Vec<bool> = MCP_FEATURE_CHOICES.iter().map(|(_, _, c)| *c).collect();
        let picked = loop {
            let picked = p.checkbox("MCP capabilities to include:", &labels, &checked)?;
            if !picked.is_empty() {
                break picked;
            }
            // inquirer `validate`: re-asks with the error message.
            eprintln!(">> Select at least one capability");
        };
        mcp_features = Some(
            picked
                .into_iter()
                .filter_map(|i| MCP_FEATURE_CHOICES.get(i).map(|(_, f, _)| *f))
                .collect(),
        );
    }

    if !matches!(
        structure,
        Structure::Frontend | Structure::McpServer | Structure::Mvc
    ) {
        backend = Some(choose(p, "Backend stack:", BACKEND_CHOICES, 0)?);
    }

    let mut frontend = None;
    if !matches!(
        structure,
        Structure::Backend | Structure::McpServer | Structure::Mvc
    ) {
        frontend = Some(choose(p, "Frontend stack:", FRONTEND_CHOICES, 0)?);
    }

    let rust_combo = structure == Structure::Monorepo
        && backend == Some(BackendStack::RustAxum)
        && frontend.is_some_and(FrontendStack::is_leptos);
    let mut rust_workspace_layout = None;
    let mut crate_prefix = None;
    if rust_combo {
        let layout = choose(p, "Cargo workspace layout:", LAYOUT_CHOICES, 0)?;
        rust_workspace_layout = Some(layout);
        if layout == RustWorkspaceLayout::Multi {
            let default = default_crate_prefix(&name);
            let v = |s: &str| validate_crate_prefix(s).map_err(str::to_string);
            crate_prefix = Some(p.input(
                "Short crate prefix (e.g. \"ars\" → ars-core / ars-server / ars-web / ars-cli):",
                &default,
                &v,
            )?);
        }
    }

    let ide = choose(p, "Primary IDE / Agent:", IDE_CHOICES, 0)?;
    let graphrag = choose(p, "GraphRAG backend:", GRAPHRAG_CHOICES, 0)?;
    let mcp = p.confirm("Enable DARE MCP Server for context queries?", true)?;
    let toolchain = choose(
        p,
        "Toolchain for scaffolding (composer / npm / cargo / python / go):",
        TOOLCHAIN_CHOICES,
        0,
    )?;

    Ok(InitAnswers {
        name,
        structure,
        backend,
        frontend,
        mcp_language,
        mcp_transport,
        mcp_features,
        rust_workspace_layout,
        crate_prefix,
        ide,
        graphrag,
        mcp,
        toolchain,
    })
}

// ─── dialoguer backend with an inquirer-like theme ──────────────────────────

/// Mimics inquirer.js: `? Message: (Use arrow keys)`, `❯` pointer, cyan answers.
pub struct InquirerTheme;

impl Theme for InquirerTheme {
    fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        write!(f, "{} {}", style("?").green(), style(prompt).bold())
    }

    fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
        write!(f, "{} {}", style(">>").red(), err)
    }

    fn format_select_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.format_prompt(f, prompt)?;
        write!(f, " {}", style("(Use arrow keys)").dim())
    }

    fn format_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        self.format_prompt(f, prompt)?;
        write!(f, " {}", style(sel).cyan())
    }

    fn format_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
    ) -> fmt::Result {
        if active {
            write!(f, "{}", style(format!("❯ {text}")).cyan())
        } else {
            write!(f, "  {text}")
        }
    }

    fn format_multi_select_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.format_prompt(f, prompt)?;
        write!(
            f,
            " {}",
            style("(Press <space> to select, <a> to toggle all, <enter> to proceed)").dim()
        )
    }

    fn format_multi_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        checked: bool,
        active: bool,
    ) -> fmt::Result {
        let pointer = if active {
            style("❯").cyan().to_string()
        } else {
            " ".to_string()
        };
        let mark = if checked {
            style("◉").green().to_string()
        } else {
            "◯".to_string()
        };
        write!(f, "{pointer}{mark} {text}")
    }

    fn format_multi_select_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        self.format_prompt(f, prompt)?;
        write!(f, " {}", style(selections.join(", ")).cyan())
    }

    fn format_input_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<&str>,
    ) -> fmt::Result {
        self.format_prompt(f, prompt)?;
        match default {
            Some(d) => write!(f, " {} ", style(format!("({d})")).dim()),
            None => write!(f, " "),
        }
    }

    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        self.format_prompt(f, prompt)?;
        write!(f, " {}", style(sel).cyan())
    }

    fn format_confirm_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<bool>,
    ) -> fmt::Result {
        self.format_prompt(f, prompt)?;
        let hint = match default {
            Some(true) => "(Y/n)",
            Some(false) => "(y/N)",
            None => "(y/n)",
        };
        write!(f, " {} ", style(hint).dim())
    }

    fn format_confirm_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selection: Option<bool>,
    ) -> fmt::Result {
        self.format_prompt(f, prompt)?;
        let answer = match selection {
            Some(true) => "Yes",
            Some(false) => "No",
            None => "",
        };
        write!(f, " {}", style(answer).cyan())
    }
}

/// Production prompter (TTY).
pub struct DialoguerPrompter {
    theme: InquirerTheme,
}

impl Default for DialoguerPrompter {
    fn default() -> Self {
        Self {
            theme: InquirerTheme,
        }
    }
}

fn map_dialoguer_err(err: dialoguer::Error) -> CoreError {
    CoreError::invalid_input(format!("prompt failed: {err}"))
}

impl Prompter for DialoguerPrompter {
    fn input(
        &mut self,
        message: &str,
        default: &str,
        validate: &dyn Fn(&str) -> Result<(), String>,
    ) -> CoreResult<String> {
        Input::<String>::with_theme(&self.theme)
            .with_prompt(message)
            .default(default.to_string())
            .validate_with(|s: &String| validate(s))
            .interact_text()
            .map_err(map_dialoguer_err)
    }

    fn select(&mut self, message: &str, items: &[&str], default: usize) -> CoreResult<usize> {
        Select::with_theme(&self.theme)
            .with_prompt(message)
            .items(items)
            .default(default)
            .interact_opt()
            .map_err(map_dialoguer_err)?
            .ok_or_else(|| CoreError::invalid_input(MSG_PROMPT_CANCELED))
    }

    fn checkbox(
        &mut self,
        message: &str,
        items: &[&str],
        checked: &[bool],
    ) -> CoreResult<Vec<usize>> {
        MultiSelect::with_theme(&self.theme)
            .with_prompt(message)
            .items(items)
            .defaults(checked)
            .interact_opt()
            .map_err(map_dialoguer_err)?
            .ok_or_else(|| CoreError::invalid_input(MSG_PROMPT_CANCELED))
    }

    fn confirm(&mut self, message: &str, default: bool) -> CoreResult<bool> {
        Confirm::with_theme(&self.theme)
            .with_prompt(message)
            .default(default)
            .interact_opt()
            .map_err(map_dialoguer_err)?
            .ok_or_else(|| CoreError::invalid_input(MSG_PROMPT_CANCELED))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::VecDeque;

    /// Scripted answers; records every question asked.
    #[derive(Default)]
    pub(crate) struct ScriptedPrompter {
        pub answers: VecDeque<String>,
        pub asked: Vec<(String, Vec<String>)>,
    }

    impl ScriptedPrompter {
        pub fn new(answers: &[&str]) -> Self {
            Self {
                answers: answers.iter().map(|s| (*s).to_string()).collect(),
                asked: Vec::new(),
            }
        }

        fn next(&mut self, message: &str, items: &[&str]) -> String {
            self.asked.push((
                message.to_string(),
                items.iter().map(|s| (*s).to_string()).collect(),
            ));
            self.answers
                .pop_front()
                .unwrap_or_else(|| panic!("no scripted answer for '{message}'"))
        }

        pub fn messages(&self) -> Vec<&str> {
            self.asked.iter().map(|(m, _)| m.as_str()).collect()
        }
    }

    /// Answers are matched against labels by substring (selects) or used verbatim.
    impl Prompter for ScriptedPrompter {
        fn input(
            &mut self,
            message: &str,
            default: &str,
            validate: &dyn Fn(&str) -> Result<(), String>,
        ) -> CoreResult<String> {
            let a = self.next(message, &[]);
            let v = if a.is_empty() { default.to_string() } else { a };
            validate(&v).map_err(CoreError::invalid_input)?;
            Ok(v)
        }

        fn select(&mut self, message: &str, items: &[&str], _default: usize) -> CoreResult<usize> {
            let a = self.next(message, items);
            items
                .iter()
                .position(|i| i.contains(a.as_str()))
                .ok_or_else(|| CoreError::invalid_input(format!("no item matching '{a}'")))
        }

        fn checkbox(
            &mut self,
            message: &str,
            items: &[&str],
            _checked: &[bool],
        ) -> CoreResult<Vec<usize>> {
            let a = self.next(message, items);
            Ok(a.split(',')
                .filter_map(|want| items.iter().position(|i| i.contains(want.trim())))
                .collect())
        }

        fn confirm(&mut self, message: &str, _default: bool) -> CoreResult<bool> {
            Ok(self.next(message, &[]) == "y")
        }
    }

    #[test]
    fn monorepo_flow_asks_backend_then_frontend() {
        let mut p = ScriptedPrompter::new(&[
            "Monorepo",
            "NestJS",
            "Vue",
            "Claude Code",
            "SQLite",
            "y",
            "Auto",
        ]);
        let a = prompt_init_answers(&mut p, Some("api-nest-validations")).expect("answers");
        assert_eq!(
            p.messages(),
            [
                "Project structure:",
                "Backend stack:",
                "Frontend stack:",
                "Primary IDE / Agent:",
                "GraphRAG backend:",
                "Enable DARE MCP Server for context queries?",
                "Toolchain for scaffolding (composer / npm / cargo / python / go):",
            ]
        );
        assert_eq!(a.structure, Structure::Monorepo);
        assert_eq!(a.backend, Some(BackendStack::NodeNestjs));
        assert_eq!(a.frontend, Some(FrontendStack::Vue));
        assert_eq!(
            p.asked[0].1,
            STRUCTURE_CHOICES
                .iter()
                .map(|(l, _)| *l)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn frontend_only_skips_backend() {
        let mut p =
            ScriptedPrompter::new(&["Frontend only", "Vue", "Cursor", "JSON", "n", "Docker only"]);
        let a = prompt_init_answers(&mut p, Some("web")).expect("answers");
        assert!(!p.messages().contains(&"Backend stack:"));
        assert_eq!(a.backend, None);
        assert_eq!(a.toolchain, ToolchainMode::Docker);
    }

    #[test]
    fn mvc_and_mcp_branches() {
        let mut p = ScriptedPrompter::new(&["MVC", "Laravel", "Codex", "SQLite", "y", "Auto"]);
        let a = prompt_init_answers(&mut p, Some("shop")).expect("mvc");
        assert_eq!(p.messages()[1], "MVC framework:");
        assert_eq!(a.backend, Some(BackendStack::PhpLaravel));
        assert_eq!(a.frontend, None);

        let mut p = ScriptedPrompter::new(&[
            "MCP Server",
            "Python",
            "SSE",
            "Tools, Prompts",
            "Cursor",
            "SQLite",
            "y",
            "Auto",
        ]);
        let a = prompt_init_answers(&mut p, Some("srv")).expect("mcp");
        assert_eq!(
            a.mcp_features,
            Some(vec![McpFeature::Tools, McpFeature::Prompts])
        );
        assert_eq!(a.mcp_transport, Some(McpTransport::Sse));
        assert!(!p.messages().contains(&"Backend stack:"));
    }

    #[test]
    fn rust_combo_asks_layout_and_prefix() {
        let mut p = ScriptedPrompter::new(&[
            "",
            "Monorepo",
            "Rust / Axum",
            "CSR-only",
            "Multi-crate",
            "",
            "Cursor",
            "SQLite",
            "y",
            "Auto",
        ]);
        let a = prompt_init_answers(&mut p, None).expect("answers");
        assert_eq!(a.name, "my-dare-project");
        assert_eq!(a.rust_workspace_layout, Some(RustWorkspaceLayout::Multi));
        assert_eq!(a.crate_prefix.as_deref(), Some("mdp"));
    }

    #[test]
    fn name_validation_message_drops_error_prefix() {
        assert_eq!(
            validate_name_prompt("Bad Name").unwrap_err(),
            "project name may only contain lowercase letters, numbers, hyphens and underscores (got 'Bad Name')"
        );
    }
}
