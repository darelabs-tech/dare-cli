//! Language detection from file path extensions.

use crate::model::Language;

/// Detect language from a file path (extension, case-insensitive).
///
/// Returns `None` when the extension is unsupported.
pub fn detect_language(path: &str) -> Option<Language> {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let ext = name.rsplit_once('.').map(|(_, e)| e)?;
    let ext = ext.to_ascii_lowercase();
    match ext.as_str() {
        "ts" | "mts" | "cts" => Some(Language::TypeScript),
        "tsx" => Some(Language::Tsx),
        "js" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
        "py" | "pyi" => Some(Language::Python),
        "php" => Some(Language::Php),
        "go" => Some(Language::Go),
        "rb" => Some(Language::Ruby),
        "rs" => Some(Language::Rust),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_common_extensions() {
        assert_eq!(detect_language("a/b/c.ts"), Some(Language::TypeScript));
        assert_eq!(detect_language("x.TSX"), Some(Language::Tsx));
        assert_eq!(detect_language("app.mjs"), Some(Language::JavaScript));
        assert_eq!(detect_language("main.py"), Some(Language::Python));
        assert_eq!(detect_language("routes.php"), Some(Language::Php));
        assert_eq!(detect_language("server.go"), Some(Language::Go));
        assert_eq!(detect_language("config.rb"), Some(Language::Ruby));
        assert_eq!(detect_language("lib.rs"), Some(Language::Rust));
        assert_eq!(detect_language("readme.md"), None);
    }
}
