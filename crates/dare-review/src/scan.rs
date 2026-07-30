//! Path helpers and text scanning.

use crate::rules::apply_line;
use crate::types::Finding;

const SCANNABLE_EXT: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "py", "go", "php", "rb", "java", "kt", "cs", "vue", "svelte",
    "md", "toml", "yml", "yaml", "json", "sh", "bash", "zsh", "c", "h", "cpp", "hpp", "sql",
];

pub fn is_scannable_path(path: &str) -> bool {
    let norm = path.replace('\\', "/");
    let Some(name) = norm.rsplit('/').next() else {
        return false;
    };
    let Some((_, ext)) = name.rsplit_once('.') else {
        return false;
    };
    SCANNABLE_EXT.contains(&ext)
}

pub fn is_test_path(path: &str) -> bool {
    let p = path.replace('\\', "/").to_ascii_lowercase();
    if p.contains("/__tests__/") || p.contains("/tests/") || p.contains("/spec/") {
        return true;
    }
    if p.contains(".test.") || p.contains(".spec.") {
        return true;
    }
    if p.ends_with("_test.rs") || p.ends_with("_tests.rs") || p.ends_with("/tests.rs") {
        return true;
    }
    // Rust integration tests directory file
    if p.contains("/tests/") && p.ends_with(".rs") {
        return true;
    }
    false
}

pub fn scan_text(path: &str, text: &str, out: &mut Vec<Finding>) {
    let in_test = is_test_path(path);
    for (i, line) in text.lines().enumerate() {
        let line_no = (i + 1) as u32;
        apply_line(path, line_no, line, in_test, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Severity;

    #[test]
    fn mock_ignored_in_test_path() {
        let mut f = Vec::new();
        scan_text("src/foo.test.ts", "const x = jest.fn();", &mut f);
        assert!(f.iter().all(|x| x.rule_id != "mock_outside_test"));
    }

    #[test]
    fn mock_flagged_outside_test() {
        let mut f = Vec::new();
        scan_text("src/service.ts", "const x = jest.fn();", &mut f);
        assert!(f.iter().any(|x| x.rule_id == "mock_outside_test"));
        assert_eq!(
            f.iter()
                .find(|x| x.rule_id == "mock_outside_test")
                .unwrap()
                .severity,
            Severity::Error
        );
    }

    #[test]
    fn is_test_path_variants() {
        assert!(is_test_path("crates/x/tests/cli.rs"));
        assert!(is_test_path("src/__tests__/a.js"));
        assert!(is_test_path("lib/foo_test.rs"));
        assert!(!is_test_path("src/main.rs"));
    }
}
