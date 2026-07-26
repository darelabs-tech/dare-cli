//! Optional YAML frontmatter for steering markdown files.

use serde::Deserialize;

use crate::PRIORITY_DEFAULT;

/// Parsed steering markdown (frontmatter + body).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSteering {
    pub scope: String,
    pub glob: Option<String>,
    pub priority: i32,
    pub body: String,
}

/// Outcome of parsing a steering file; invalid scope yields skip + warning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseOutcome {
    Ok(ParsedSteering),
    Skip { warning: String },
}

#[derive(Debug, Deserialize)]
struct FrontmatterFields {
    #[serde(default = "default_scope")]
    scope: String,
    #[serde(default)]
    glob: Option<String>,
    #[serde(default = "default_priority")]
    priority: i32,
}

fn default_scope() -> String {
    "project".to_string()
}

fn default_priority() -> i32 {
    PRIORITY_DEFAULT
}

/// Parse markdown with optional leading `---` YAML `---` frontmatter.
pub fn parse_steering_markdown(content: &str) -> ParseOutcome {
    let (fields, body) = match split_frontmatter(content) {
        Some((yaml, body)) => {
            let fields: FrontmatterFields = match serde_yaml::from_str(yaml) {
                Ok(f) => f,
                Err(e) => {
                    return ParseOutcome::Skip {
                        warning: format!("invalid steering frontmatter: {e}"),
                    };
                }
            };
            (fields, body.to_string())
        }
        None => (
            FrontmatterFields {
                scope: default_scope(),
                glob: None,
                priority: default_priority(),
            },
            content.to_string(),
        ),
    };

    match fields.scope.as_str() {
        "project" | "glob" => ParseOutcome::Ok(ParsedSteering {
            scope: fields.scope,
            glob: fields.glob,
            priority: fields.priority,
            body,
        }),
        other => ParseOutcome::Skip {
            warning: format!("invalid steering scope: {other}"),
        },
    }
}

fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let trimmed = content.strip_prefix('\u{feff}').unwrap_or(content);
    let rest = if let Some(r) = trimmed.strip_prefix("---\n") {
        r
    } else if let Some(r) = trimmed.strip_prefix("---\r\n") {
        r
    } else {
        return None;
    };

    // Closing fence on its own line
    if let Some(idx) = rest.find("\n---\n") {
        let yaml = &rest[..idx];
        let body = &rest[idx + "\n---\n".len()..];
        return Some((yaml, body));
    }
    if let Some(idx) = rest.find("\n---\r\n") {
        let yaml = &rest[..idx];
        let body = &rest[idx + "\n---\r\n".len()..];
        return Some((yaml, body));
    }
    // Closing at EOF: `\n---`
    if let Some(stripped) = rest.strip_suffix("\n---") {
        return Some((stripped, ""));
    }
    if let Some(stripped) = rest.strip_suffix("\r\n---") {
        return Some((stripped, ""));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults_without_frontmatter() {
        let out = parse_steering_markdown("hello");
        match out {
            ParseOutcome::Ok(p) => {
                assert_eq!(p.scope, "project");
                assert_eq!(p.priority, PRIORITY_DEFAULT);
                assert!(p.glob.is_none());
                assert_eq!(p.body, "hello");
            }
            ParseOutcome::Skip { warning } => panic!("unexpected skip: {warning}"),
        }
    }

    #[test]
    fn invalid_scope_skips() {
        let md = "---\nscope: weird\n---\nbody\n";
        match parse_steering_markdown(md) {
            ParseOutcome::Skip { warning } => {
                assert!(warning.contains("invalid steering scope"));
            }
            ParseOutcome::Ok(_) => panic!("expected skip"),
        }
    }
}
