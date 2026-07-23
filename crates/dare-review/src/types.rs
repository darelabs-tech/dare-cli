//! Finding types and CLI enums.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }

    pub fn github_token(self) -> &'static str {
        self.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailOn {
    Error,
    Warning,
    Never,
}

impl FailOn {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "error" => Some(FailOn::Error),
            "warning" => Some(FailOn::Warning),
            "never" => Some(FailOn::Never),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            FailOn::Error => "error",
            FailOn::Warning => "warning",
            FailOn::Never => "never",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
    Github,
}

impl OutputFormat {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "human" => Some(OutputFormat::Human),
            "json" => Some(OutputFormat::Json),
            "github" => Some(OutputFormat::Github),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            OutputFormat::Human => "human",
            OutputFormat::Json => "json",
            OutputFormat::Github => "github",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub path: String,
    pub line: u32,
    pub col: u32,
    pub severity: Severity,
    pub rule_id: String,
    pub message: String,
}
