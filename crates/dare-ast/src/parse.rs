//! tree-sitter parse helpers with feature-gated grammars.

use thiserror::Error;
use tree_sitter::{Language as TsLanguage, Parser, Tree};

use crate::model::Language;

/// Parse failure (grammar missing, language set, or tree-sitter error).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
    #[error("grammar not available for language `{0}` (feature disabled or missing)")]
    GrammarUnavailable(String),
    #[error("failed to set tree-sitter language: {0}")]
    Language(String),
    #[error("tree-sitter parse returned no tree")]
    EmptyTree,
}

/// Whether a native grammar is compiled in for `lang`.
pub fn grammar_available(lang: Language) -> bool {
    ts_language(lang).is_some()
}

fn ts_language(lang: Language) -> Option<TsLanguage> {
    match lang {
        #[cfg(feature = "lang-typescript")]
        Language::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        #[cfg(not(feature = "lang-typescript"))]
        Language::TypeScript => None,

        #[cfg(feature = "lang-tsx")]
        Language::Tsx => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
        #[cfg(not(feature = "lang-tsx"))]
        Language::Tsx => None,

        #[cfg(feature = "lang-javascript")]
        Language::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
        #[cfg(not(feature = "lang-javascript"))]
        Language::JavaScript => None,

        #[cfg(feature = "lang-python")]
        Language::Python => Some(tree_sitter_python::LANGUAGE.into()),
        #[cfg(not(feature = "lang-python"))]
        Language::Python => None,

        #[cfg(feature = "lang-php")]
        Language::Php => Some(tree_sitter_php::LANGUAGE_PHP.into()),
        #[cfg(not(feature = "lang-php"))]
        Language::Php => None,

        #[cfg(feature = "lang-go")]
        Language::Go => Some(tree_sitter_go::LANGUAGE.into()),
        #[cfg(not(feature = "lang-go"))]
        Language::Go => None,

        #[cfg(feature = "lang-ruby")]
        Language::Ruby => Some(tree_sitter_ruby::LANGUAGE.into()),
        #[cfg(not(feature = "lang-ruby"))]
        Language::Ruby => None,

        #[cfg(feature = "lang-rust")]
        Language::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
        #[cfg(not(feature = "lang-rust"))]
        Language::Rust => None,
    }
}

/// Parse `source` with the grammar for `lang`.
pub fn parse_source(lang: Language, source: &str) -> Result<Tree, ParseError> {
    let language = ts_language(lang)
        .ok_or_else(|| ParseError::GrammarUnavailable(lang.as_str().to_string()))?;
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .map_err(|e| ParseError::Language(e.to_string()))?;
    parser.parse(source, None).ok_or(ParseError::EmptyTree)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_flags_match_features() {
        #[cfg(feature = "lang-python")]
        assert!(grammar_available(Language::Python));
        #[cfg(not(feature = "lang-python"))]
        assert!(!grammar_available(Language::Python));
    }

    #[cfg(feature = "lang-javascript")]
    #[test]
    fn parses_javascript_snippet() {
        let tree = parse_source(Language::JavaScript, "const x = 1;\n").expect("parse");
        assert!(!tree.root_node().has_error());
    }
}
