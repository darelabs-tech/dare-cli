//! Parser for `docs/compatibility/parity-diff-log.md` table rows.

use std::collections::BTreeSet;
use std::path::Path;

use dare_core::{CoreError, CoreResult};

/// Error when Class C skip lacks ADR / diff-log classification.
pub const MSG_UNCLASSIFIED_DIFF: &str =
    "unclassified parity diff; add entry to parity-diff-log.md";

/// Index of `diff_id` + `surface` keys from the parity diff log markdown table.
#[derive(Debug, Clone, Default)]
pub struct DiffLogIndex {
    keys: BTreeSet<String>,
}

impl DiffLogIndex {
    /// Empty index (useful for unit tests).
    pub fn empty() -> Self {
        Self {
            keys: BTreeSet::new(),
        }
    }

    /// Load and parse the markdown table from `path`.
    pub fn load(path: &Path) -> CoreResult<Self> {
        let text = std::fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::not_found(format!("missing {}", path.display()))
            } else {
                CoreError::io(format!("read {}: {e}", path.display()))
            }
        })?;
        Self::parse(&text)
    }

    /// Parse markdown table body into an index of `diff_id` and `surface` values.
    pub fn parse(markdown: &str) -> CoreResult<Self> {
        let mut keys = BTreeSet::new();
        let mut in_table = false;

        for line in markdown.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with('|') {
                in_table = false;
                continue;
            }
            let cells: Vec<&str> = trimmed
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect();
            if cells.len() < 2 {
                continue;
            }
            // Header / separator
            if cells[0].eq_ignore_ascii_case("diff_id")
                || cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':'))
            {
                in_table = true;
                continue;
            }
            if !in_table {
                // Accept rows even before explicit header if they look like PD-* ids.
                if !cells[0].starts_with("PD-") {
                    continue;
                }
                in_table = true;
            }
            let diff_id = cells[0].trim();
            let surface = cells[1].trim();
            if !diff_id.is_empty() {
                keys.insert(diff_id.to_string());
            }
            if !surface.is_empty() {
                keys.insert(surface.to_string());
            }
        }

        Ok(Self { keys })
    }

    /// True if `key` matches a `diff_id`, `surface`, or exact case id entry.
    pub fn contains_surface_or_id(&self, key: &str) -> bool {
        let k = key.trim();
        if k.is_empty() {
            return false;
        }
        self.keys.contains(k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_table_rows() {
        let md = r#"
# Parity

| diff_id | surface | class | action | adr_ref | notes |
|---------|---------|-------|--------|---------|-------|
| PD-001 | design LLM variance | C | accept | ADR-1 | note |
| PD-002 | golden.help.root | C | skip | | |
"#;
        let idx = DiffLogIndex::parse(md).expect("parse");
        assert!(idx.contains_surface_or_id("PD-001"));
        assert!(idx.contains_surface_or_id("design LLM variance"));
        assert!(idx.contains_surface_or_id("golden.help.root"));
        assert!(!idx.contains_surface_or_id("missing"));
    }
}
