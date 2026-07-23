//! Regex fallback extractors (always available, even without grammars).

use regex::Regex;
use std::sync::OnceLock;

use crate::model::{Entity, HttpEndpoint, Language, SourceKind};

/// Extract endpoints and entities via language-aware regex heuristics.
pub fn extract_regex(lang: Language, source: &str) -> (Vec<HttpEndpoint>, Vec<Entity>) {
    let endpoints = regex_endpoints(lang, source);
    let entities = regex_entities(lang, source);
    (endpoints, entities)
}

fn regex_endpoints(lang: Language, source: &str) -> Vec<HttpEndpoint> {
    let mut out = Vec::new();
    for (re, method_idx, path_idx, default_method) in endpoint_patterns(lang) {
        for caps in re.captures_iter(source) {
            let method = if let Some(m) = default_method {
                m.to_string()
            } else {
                caps.get(method_idx)
                    .map(|m| m.as_str().to_ascii_uppercase())
                    .unwrap_or_else(|| "GET".to_string())
            };
            let path = caps
                .get(path_idx)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            if path.is_empty() {
                continue;
            }
            let line = line_of(source, caps.get(0).map(|m| m.start()).unwrap_or(0));
            out.push(HttpEndpoint {
                method,
                path,
                line,
                source: SourceKind::Regex,
            });
        }
    }
    out
}

fn regex_entities(lang: Language, source: &str) -> Vec<Entity> {
    let mut out = Vec::new();
    for (re, kind) in entity_patterns(lang) {
        for caps in re.captures_iter(source) {
            let name = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let line = line_of(source, caps.get(0).map(|m| m.start()).unwrap_or(0));
            out.push(Entity {
                name,
                kind: kind.to_string(),
                line,
                source: SourceKind::Regex,
            });
        }
    }
    out
}

fn endpoint_patterns(lang: Language) -> Vec<(&'static Regex, usize, usize, Option<&'static str>)> {
    match lang {
        Language::TypeScript | Language::Tsx | Language::JavaScript => vec![
            (re_js_member(), 1, 2, None),
            (re_js_decorator(), 1, 2, None),
        ],
        Language::Python => vec![(re_py_decorator(), 1, 2, None)],
        Language::Php => vec![(re_php_route(), 1, 2, None)],
        Language::Go => vec![
            (re_go_method(), 1, 2, None),
            (re_go_handle(), 1, 1, Some("GET")),
        ],
        Language::Ruby => vec![(re_ruby_route(), 1, 2, None)],
        Language::Rust => vec![
            (re_rust_route(), 1, 1, Some("GET")),
            (re_rust_attr(), 1, 2, None),
            (re_rust_method_fn(), 1, 2, None),
        ],
    }
}

fn entity_patterns(lang: Language) -> Vec<(&'static Regex, &'static str)> {
    match lang {
        Language::TypeScript | Language::Tsx | Language::JavaScript => vec![
            (re_class(), "class"),
            (re_interface(), "interface"),
            (re_enum(), "enum"),
        ],
        Language::Python | Language::Php | Language::Ruby => vec![(re_class(), "class")],
        Language::Go => vec![(re_go_struct(), "struct"), (re_go_interface(), "interface")],
        Language::Rust => vec![
            (re_rust_struct(), "struct"),
            (re_rust_enum(), "enum"),
            (re_rust_trait(), "interface"),
        ],
    }
}

fn line_of(source: &str, byte_idx: usize) -> u32 {
    let idx = byte_idx.min(source.len());
    source[..idx].bytes().filter(|&b| b == b'\n').count() as u32 + 1
}

fn re_js_member() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?i)\.(get|post|put|patch|delete|options|head)\s*\(\s*['"]([^'"]+)['"]"#)
            .expect("regex")
    })
}

fn re_js_decorator() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?i)@(get|post|put|patch|delete|options|head)\s*\(\s*['"]([^'"]+)['"]"#)
            .expect("regex")
    })
}

fn re_py_decorator() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?i)@(?:app|router)\.(get|post|put|patch|delete|options|head)\s*\(\s*['"]([^'"]+)['"]"#,
        )
        .expect("regex")
    })
}

fn re_php_route() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?i)Route::(get|post|put|patch|delete|options|head)\s*\(\s*['"]([^'"]+)['"]"#)
            .expect("regex")
    })
}

fn re_go_method() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"\.(GET|POST|PUT|PATCH|DELETE|OPTIONS|HEAD)\s*\(\s*["']([^"']+)["']"#)
            .expect("regex")
    })
}

fn re_go_handle() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"HandleFunc\s*\(\s*["']([^"']+)["']"#).expect("regex"))
}

fn re_ruby_route() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?i)\b(get|post|put|patch|delete)\s+['"]([^'"]+)['"]"#).expect("regex")
    })
}

fn re_rust_route() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"\.route\s*\(\s*["']([^"']+)["']"#).expect("regex"))
}

fn re_rust_attr() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"#\[(get|post|put|patch|delete)\s*\(\s*["']([^"']+)["']\s*\)\]"#)
            .expect("regex")
    })
}

fn re_rust_method_fn() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"\.(get|post|put|patch|delete)\s*\(\s*["']([^"']+)["']"#).expect("regex")
    })
}

fn re_class() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*(?:export\s+)?(?:abstract\s+)?class\s+([A-Za-z_][\w]*)"#)
            .expect("regex")
    })
}

fn re_interface() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*(?:export\s+)?interface\s+([A-Za-z_][\w]*)"#).expect("regex")
    })
}

fn re_enum() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*(?:export\s+)?enum\s+([A-Za-z_][\w]*)"#).expect("regex")
    })
}

fn re_go_struct() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?m)^\s*type\s+([A-Za-z_][\w]*)\s+struct\b"#).expect("regex"))
}

fn re_go_interface() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*type\s+([A-Za-z_][\w]*)\s+interface\b"#).expect("regex")
    })
}

fn re_rust_struct() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z_][\w]*)"#).expect("regex")
    })
}

fn re_rust_enum() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?enum\s+([A-Za-z_][\w]*)"#).expect("regex")
    })
}

fn re_rust_trait() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?trait\s+([A-Za-z_][\w]*)"#).expect("regex")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_extracts_js_without_grammar() {
        let src = "app.get('/users', handler);\nexport class User {}\n";
        let (eps, ents) = extract_regex(Language::JavaScript, src);
        assert!(eps.iter().any(|e| e.method == "GET" && e.path == "/users"));
        assert!(ents.iter().any(|e| e.name == "User" && e.kind == "class"));
    }
}
