//! Domain types for AST / regex extraction.

use serde::Serialize;

/// Soft cap on source size (2 MiB), aligned with persisted-contract limits.
pub const MAX_SOURCE_BYTES: usize = 2 * 1024 * 1024;

/// Supported languages for the AST engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    TypeScript,
    Tsx,
    JavaScript,
    Python,
    Php,
    Go,
    Ruby,
    Rust,
}

impl Language {
    /// Stable lowercase id used in docs and warnings.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TypeScript => "typescript",
            Self::Tsx => "tsx",
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Php => "php",
            Self::Go => "go",
            Self::Ruby => "ruby",
            Self::Rust => "rust",
        }
    }
}

/// Provenance of an extracted symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    Ast,
    Regex,
}

/// HTTP endpoint discovered in source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpEndpoint {
    pub method: String,
    pub path: String,
    pub line: u32,
    pub source: SourceKind,
}

/// Type-like entity (class, struct, interface, …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entity {
    pub name: String,
    pub kind: String,
    pub line: u32,
    pub source: SourceKind,
}

/// Deterministic extraction result for one source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DataModel {
    pub language: Language,
    pub endpoints: Vec<HttpEndpoint>,
    pub entities: Vec<Entity>,
    pub warnings: Vec<String>,
}
