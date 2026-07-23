//! Public analysis entrypoints.

use dare_core::{CoreError, CoreResult};

use crate::extract::extract_from_tree;
use crate::language::detect_language;
use crate::merge::merge_extractions;
use crate::model::{DataModel, Language, MAX_SOURCE_BYTES};
use crate::parse::{grammar_available, parse_source};
use crate::regex_fallback::extract_regex;

/// Analyze a source buffer identified by `path` (used for language detection).
pub fn analyze_source(path: &str, source: &str) -> CoreResult<DataModel> {
    if source.as_bytes().contains(&0) {
        return Err(CoreError::invalid_input("source must not contain NUL"));
    }
    if source.len() > MAX_SOURCE_BYTES {
        return Err(CoreError::invalid_input(format!(
            "source exceeds MAX_SOURCE_BYTES ({MAX_SOURCE_BYTES})"
        )));
    }

    let language = detect_language(path).ok_or_else(|| {
        CoreError::invalid_input(format!("unsupported language for path `{path}`"))
    })?;

    analyze_language(language, source)
}

fn analyze_language(language: Language, source: &str) -> CoreResult<DataModel> {
    let mut warnings = Vec::new();
    let (ast_endpoints, ast_entities) = if grammar_available(language) {
        match parse_source(language, source) {
            Ok(tree) => extract_from_tree(language, source, &tree),
            Err(e) => {
                warnings.push(format!("ast parse failed: {e}; using regex fallback"));
                (Vec::new(), Vec::new())
            }
        }
    } else {
        warnings.push(format!(
            "grammar not available for `{}`; using regex fallback",
            language.as_str()
        ));
        (Vec::new(), Vec::new())
    };

    let (regex_endpoints, regex_entities) = extract_regex(language, source);
    let (endpoints, entities) =
        merge_extractions(ast_endpoints, ast_entities, regex_endpoints, regex_entities);

    Ok(DataModel {
        language,
        endpoints,
        entities,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SourceKind;

    #[test]
    fn rejects_nul_and_oversize() {
        assert!(analyze_source("a.js", "a\0b").is_err());
        let big = "x".repeat(MAX_SOURCE_BYTES + 1);
        assert!(analyze_source("a.js", &big).is_err());
    }

    #[test]
    fn rejects_unknown_extension() {
        assert!(analyze_source("readme.md", "hello").is_err());
    }

    #[test]
    fn analyzes_javascript_sample() {
        let src = include_str!("../fixtures/javascript/sample.js");
        let model = analyze_source("fixtures/javascript/sample.js", src).expect("ok");
        assert_eq!(model.language, Language::JavaScript);
        assert!(
            model
                .endpoints
                .iter()
                .any(|e| e.method == "GET" && e.path == "/users"),
            "endpoints={:?}",
            model.endpoints
        );
        assert!(
            model.entities.iter().any(|e| e.name == "User"),
            "entities={:?}",
            model.entities
        );
        // With default features, AST should win for overlapping keys when parse works.
        let _ = SourceKind::Ast;
    }
}
