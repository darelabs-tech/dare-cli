//! `ProjectConfig` — mirror of npm `project-generator.ts` `ProjectConfig` (cycle 011).
//!
//! Every enum serializes to the exact `value` strings used by the npm
//! `dare init` prompts so `dare.config.json` stays byte-compatible.

use std::path::{Component, Path, PathBuf};

use dare_core::{CoreError, CoreResult};

macro_rules! npm_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant),+];

            /// npm `value` string.
            pub fn as_str(self) -> &'static str {
                match self {
                    $($name::$variant => $value),+
                }
            }

            /// Parse an npm `value` string (exact match).
            pub fn parse(input: &str) -> Option<Self> {
                match input {
                    $($value => Some($name::$variant),)+
                    _ => None,
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

npm_enum!(
    /// `Project structure:` prompt.
    Structure {
        Monorepo => "monorepo",
        Backend => "backend",
        Frontend => "frontend",
        Mvc => "mvc",
        McpServer => "mcp-server",
    }
);

npm_enum!(
    /// `Backend stack:` / `MVC framework:` / `--stack`.
    BackendStack {
        RubyRails8 => "ruby-rails-8",
        RustAxum => "rust-axum",
        NodeNestjs => "node-nestjs",
        PythonFastapi => "python-fastapi",
        PhpLaravel => "php-laravel",
        GoGin => "go-gin",
        GoStdlib => "go-stdlib",
    }
);

npm_enum!(
    /// `Frontend stack:` prompt.
    FrontendStack {
        React => "react",
        Vue => "vue",
        RustLeptos => "rust-leptos",
        RustLeptosCsr => "rust-leptos-csr",
        None => "none",
    }
);

npm_enum!(
    /// `MCP server language:` / `--mcp`.
    McpLanguage {
        NodeTs => "node-ts",
        Python => "python",
        Rust => "rust",
        Go => "go",
    }
);

npm_enum!(
    /// `Transport type:` (config uses the historical `http-stream` label).
    McpTransport {
        Stdio => "stdio",
        Sse => "sse",
        HttpStream => "http-stream",
    }
);

npm_enum!(
    /// `MCP capabilities to include:`.
    McpFeature {
        Tools => "tools",
        Resources => "resources",
        Prompts => "prompts",
    }
);

npm_enum!(
    /// `Primary IDE / Agent:`.
    Ide {
        ClaudeCode => "claude-code",
        Codex => "codex",
        Cursor => "cursor",
        Antigravity => "antigravity",
        Hybrid => "hybrid",
        ClaudeHybrid => "claude-hybrid",
    }
);

npm_enum!(
    /// `GraphRAG backend:`.
    GraphragBackend {
        Sqlite => "sqlite",
        Json => "json",
        Neo4j => "neo4j",
    }
);

npm_enum!(
    /// `Toolchain for scaffolding` / `--toolchain`.
    ToolchainMode {
        Auto => "auto",
        Native => "native",
        Docker => "docker",
    }
);

npm_enum!(
    /// `Cargo workspace layout:`.
    RustWorkspaceLayout {
        Single => "single",
        Multi => "multi",
    }
);

impl BackendStack {
    /// Stacks accepted by `--fullstack` (structure `mvc`).
    pub fn is_mvc_capable(self) -> bool {
        matches!(self, BackendStack::RubyRails8 | BackendStack::PhpLaravel)
    }
}

impl FrontendStack {
    pub fn is_leptos(self) -> bool {
        matches!(
            self,
            FrontendStack::RustLeptos | FrontendStack::RustLeptosCsr
        )
    }
}

impl McpTransport {
    /// Scaffolder transport (`http-stream` → `http`), as in `runStackBootstrap`.
    pub fn scaffold_value(self) -> &'static str {
        match self {
            McpTransport::Stdio => "stdio",
            McpTransport::Sse => "sse",
            McpTransport::HttpStream => "http",
        }
    }
}

/// Resolved `dare init` answers — same fields as npm `ProjectConfig`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectConfig {
    pub name: String,
    pub structure: Structure,
    pub backend: Option<BackendStack>,
    pub frontend: Option<FrontendStack>,
    pub mcp_transport: Option<McpTransport>,
    pub mcp_language: Option<McpLanguage>,
    pub mcp_features: Option<Vec<McpFeature>>,
    pub ide: Ide,
    pub graphrag: GraphragBackend,
    pub mcp: bool,
    pub toolchain: ToolchainMode,
    pub rust_workspace_layout: Option<RustWorkspaceLayout>,
    pub crate_prefix: Option<String>,
    pub output_dir: PathBuf,
}

impl ProjectConfig {
    /// monorepo + rust-axum + rust-leptos* → unified Cargo workspace.
    pub fn is_rust_monorepo(&self) -> bool {
        self.structure == Structure::Monorepo
            && self.backend == Some(BackendStack::RustAxum)
            && self.frontend.is_some_and(FrontendStack::is_leptos)
    }

    pub fn layout(&self) -> RustWorkspaceLayout {
        self.rust_workspace_layout
            .unwrap_or(RustWorkspaceLayout::Single)
    }
}

/// npm `PROJECT_NAME_RE` (`^[a-z0-9-_]+$`).
pub fn is_valid_npm_project_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

fn simple_name_error(name: &str) -> String {
    format!(
        "Error: project name must be a simple directory name under the current folder (got '{name}')"
    )
}

fn regex_name_error(name: &str) -> String {
    format!(
        "Error: project name may only contain lowercase letters, numbers, hyphens and underscores (got '{name}')"
    )
}

pub const MSG_CWD_ESCAPE: &str =
    "Error: project directory must stay inside the current working directory";

/// `path.posix.normalize(name)` starts with a `..` segment (npm `nameHasTraversal`).
fn name_has_traversal(name: &str) -> bool {
    let mut stack: Vec<&str> = Vec::new();
    let mut leading_parents = 0usize;
    for seg in name.split(['/', '\\']) {
        match seg {
            "" | "." => {}
            ".." => {
                if stack.pop().is_none() {
                    leading_parents += 1;
                }
            }
            other => stack.push(other),
        }
    }
    leading_parents > 0
}

fn is_absolute_like(name: &str) -> bool {
    let p = Path::new(name);
    p.is_absolute() || p.has_root() || name.starts_with('/') || name.starts_with('\\')
}

/// Port of npm `validateProjectName` — returns the sanitized (trimmed) name.
/// Errors carry the exact npm message (with the `Error: ` prefix).
pub fn validate_npm_project_name(name: &str) -> CoreResult<String> {
    if name == "." || name == ".." || is_absolute_like(name) || name_has_traversal(name) {
        return Err(CoreError::invalid_input(simple_name_error(name)));
    }
    let trimmed = name.trim();
    if !is_valid_npm_project_name(trimmed) {
        return Err(CoreError::invalid_input(regex_name_error(name)));
    }
    Ok(trimmed.to_string())
}

/// Lexically normalize `path` (resolve `.`/`..` without touching the FS).
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in path.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Port of `assertWithinCwd`.
pub fn assert_within_cwd(cwd: &Path, target: &Path) -> CoreResult<()> {
    let cwd_n = normalize(cwd);
    let target_n = normalize(&cwd.join(target));
    if target_n.starts_with(&cwd_n) {
        Ok(())
    } else {
        Err(CoreError::invalid_input(MSG_CWD_ESCAPE))
    }
}

/// Port of `resolveProjectOutputDir`.
pub fn resolve_project_output_dir(cwd: &Path, name: &str) -> CoreResult<PathBuf> {
    let out = normalize(&cwd.join(name));
    assert_within_cwd(cwd, &out)?;
    Ok(out)
}

/// npm slug: lowercase, `[^a-z0-9-]+` → `-`, trim `-`.
pub fn npm_slug(name: &str) -> String {
    let lower = name.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut in_run = false;
    for ch in lower.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' {
            out.push(ch);
            in_run = false;
        } else if !in_run {
            out.push('-');
            in_run = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Default crate prefix offered by the `Short crate prefix` prompt.
pub fn default_crate_prefix(name: &str) -> String {
    let slug = npm_slug(name);
    let initials: String = slug.split('-').filter_map(|p| p.chars().next()).collect();
    if initials.chars().count() >= 2 {
        initials
    } else if !slug.is_empty() {
        slug.chars().take(6).collect()
    } else {
        "app".to_string()
    }
}

/// Validation of the `Short crate prefix` prompt (returns npm message on error).
pub fn validate_crate_prefix(input: &str) -> Result<(), &'static str> {
    let t = input.trim();
    if t.is_empty() {
        return Err("Prefix cannot be empty");
    }
    let mut chars = t.chars();
    let first_ok = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    let rest_ok = chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if first_ok && rest_ok {
        Ok(())
    } else {
        Err("Use only lowercase letters, numbers, hyphens (must start with a letter)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_values_match_npm() {
        assert_eq!(Structure::McpServer.as_str(), "mcp-server");
        assert_eq!(
            BackendStack::parse("go-stdlib"),
            Some(BackendStack::GoStdlib)
        );
        assert_eq!(FrontendStack::RustLeptosCsr.as_str(), "rust-leptos-csr");
        assert_eq!(McpTransport::HttpStream.as_str(), "http-stream");
        assert_eq!(McpTransport::HttpStream.scaffold_value(), "http");
        assert_eq!(Ide::ClaudeHybrid.as_str(), "claude-hybrid");
        assert_eq!(ToolchainMode::parse("docker"), Some(ToolchainMode::Docker));
        assert_eq!(BackendStack::parse("rails"), None);
    }

    #[test]
    fn project_name_validation_matches_npm_messages() {
        assert_eq!(
            validate_npm_project_name("api-nest_1").unwrap(),
            "api-nest_1"
        );
        let err = validate_npm_project_name("..").unwrap_err();
        assert_eq!(err.message(), simple_name_error(".."));
        let err = validate_npm_project_name("a/../../b").unwrap_err();
        assert_eq!(err.message(), simple_name_error("a/../../b"));
        // npm normalizes `a/../b` to `b` (no traversal) and then fails the regex.
        let err = validate_npm_project_name("a/../b").unwrap_err();
        assert_eq!(err.message(), regex_name_error("a/../b"));
        let err = validate_npm_project_name("MyApp").unwrap_err();
        assert_eq!(
            err.message(),
            "Error: project name may only contain lowercase letters, numbers, hyphens and underscores (got 'MyApp')"
        );
        assert!(validate_npm_project_name("   ").is_err());
    }

    #[test]
    fn output_dir_stays_within_cwd() {
        let cwd = Path::new("/work/projects");
        assert_eq!(
            resolve_project_output_dir(cwd, "demo").unwrap(),
            PathBuf::from("/work/projects/demo")
        );
        assert_eq!(
            assert_within_cwd(cwd, Path::new("../escape"))
                .unwrap_err()
                .message(),
            MSG_CWD_ESCAPE
        );
    }

    #[test]
    fn crate_prefix_defaults_and_validation() {
        assert_eq!(npm_slug("Api Rust--Service!"), "api-rust--service");
        assert_eq!(default_crate_prefix("acme-rust-store"), "ars");
        assert_eq!(default_crate_prefix("platform"), "platfo");
        assert_eq!(default_crate_prefix("!!!"), "app");
        assert!(validate_crate_prefix("ars").is_ok());
        assert_eq!(validate_crate_prefix(" "), Err("Prefix cannot be empty"));
        assert!(validate_crate_prefix("1ab").is_err());
    }

    #[test]
    fn rust_monorepo_detection() {
        let cfg = ProjectConfig {
            name: "x".into(),
            structure: Structure::Monorepo,
            backend: Some(BackendStack::RustAxum),
            frontend: Some(FrontendStack::RustLeptosCsr),
            mcp_transport: None,
            mcp_language: None,
            mcp_features: None,
            ide: Ide::Cursor,
            graphrag: GraphragBackend::Sqlite,
            mcp: false,
            toolchain: ToolchainMode::Auto,
            rust_workspace_layout: None,
            crate_prefix: None,
            output_dir: PathBuf::from("x"),
        };
        assert!(cfg.is_rust_monorepo());
        assert_eq!(cfg.layout(), RustWorkspaceLayout::Single);
    }
}
