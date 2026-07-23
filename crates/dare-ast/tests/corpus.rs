//! Corpus golden tests — one fixture per language.

use dare_ast::{analyze_source, Language};

fn assert_min(path: &str, source: &str, lang: Language, endpoint_path: &str, entity_name: &str) {
    let model = analyze_source(path, source).unwrap_or_else(|e| panic!("{path}: {e}"));
    assert_eq!(model.language, lang, "{path}");
    assert!(
        model.endpoints.iter().any(|e| e.path == endpoint_path),
        "{path} missing endpoint {endpoint_path}; got {:?}",
        model.endpoints
    );
    assert!(
        model.entities.iter().any(|e| e.name == entity_name),
        "{path} missing entity {entity_name}; got {:?}",
        model.entities
    );
}

#[test]
fn corpus_javascript() {
    assert_min(
        "fixtures/javascript/sample.js",
        include_str!("../fixtures/javascript/sample.js"),
        Language::JavaScript,
        "/users",
        "User",
    );
}

#[test]
fn corpus_typescript() {
    assert_min(
        "fixtures/typescript/sample.ts",
        include_str!("../fixtures/typescript/sample.ts"),
        Language::TypeScript,
        "/items",
        "Item",
    );
}

#[test]
fn corpus_tsx() {
    assert_min(
        "fixtures/tsx/sample.tsx",
        include_str!("../fixtures/tsx/sample.tsx"),
        Language::Tsx,
        "/tsx-items",
        "Catalog",
    );
}

#[test]
fn corpus_python() {
    assert_min(
        "fixtures/python/sample.py",
        include_str!("../fixtures/python/sample.py"),
        Language::Python,
        "/health",
        "Item",
    );
}

#[test]
fn corpus_php() {
    assert_min(
        "fixtures/php/sample.php",
        include_str!("../fixtures/php/sample.php"),
        Language::Php,
        "/api/users",
        "User",
    );
}

#[test]
fn corpus_go() {
    assert_min(
        "fixtures/go/sample.go",
        include_str!("../fixtures/go/sample.go"),
        Language::Go,
        "/users",
        "User",
    );
}

#[test]
fn corpus_ruby() {
    assert_min(
        "fixtures/ruby/sample.rb",
        include_str!("../fixtures/ruby/sample.rb"),
        Language::Ruby,
        "/users",
        "User",
    );
}

#[test]
fn corpus_rust() {
    assert_min(
        "fixtures/rust/sample.rs",
        include_str!("../fixtures/rust/sample.rs"),
        Language::Rust,
        "/users",
        "User",
    );
}

#[test]
fn regex_only_still_works() {
    // Even if grammars are off, regex path must produce results.
    let src = include_str!("../fixtures/javascript/sample.js");
    let model = analyze_source("sample.js", src).expect("ok");
    assert!(!model.endpoints.is_empty());
    assert!(!model.entities.is_empty());
}
