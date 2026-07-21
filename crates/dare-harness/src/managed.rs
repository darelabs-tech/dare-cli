//! Shared managed-content detection for harness adapters and update classify.

/// Returns true if the first line (after `trim_start`) indicates DARE-managed content.
///
/// Markers: HTML comment `<!-- dare:managed` or YAML frontmatter `---`.
pub fn content_is_managed(bytes: &[u8]) -> bool {
    String::from_utf8_lossy(bytes)
        .lines()
        .next()
        .map(|l| {
            let t = l.trim_start();
            t.starts_with("<!-- dare:managed") || t.starts_with("---")
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_is_managed_markers() {
        assert!(content_is_managed(b"<!-- dare:managed -->\nbody"));
        assert!(content_is_managed(b"  <!-- dare:managed foo -->\n"));
        assert!(content_is_managed(b"---\nname: skill\n---\n"));
        assert!(content_is_managed(b"  ---\n"));
        assert!(!content_is_managed(b"# Custom\n"));
        assert!(!content_is_managed(b""));
    }
}
