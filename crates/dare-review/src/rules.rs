//! Line-level anti-stub rules (deterministic, no regex crate).

use crate::types::{Finding, Severity};

fn word_boundary_match(hay: &str, needle: &str) -> Option<usize> {
    let bytes = hay.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() || bytes.len() < n.len() {
        return None;
    }
    let mut i = 0;
    while i + n.len() <= bytes.len() {
        if &bytes[i..i + n.len()] == n {
            let before_ok = i == 0 || !is_ident_byte(bytes[i - 1]);
            let after_ok = i + n.len() == bytes.len() || !is_ident_byte(bytes[i + n.len()]);
            if before_ok && after_ok {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn col_at(line: &str, byte_idx: usize) -> u32 {
    let prefix = &line[..byte_idx.min(line.len())];
    (prefix.chars().count() as u32).saturating_add(1)
}

fn push(
    out: &mut Vec<Finding>,
    path: &str,
    line_no: u32,
    col: u32,
    severity: Severity,
    rule_id: &str,
    message: &str,
) {
    out.push(Finding {
        path: path.to_string(),
        line: line_no,
        col,
        severity,
        rule_id: rule_id.to_string(),
        message: message.to_string(),
    });
}

fn strip_strings_rough(line: &str) -> String {
    // Best-effort: blank out double/single quoted regions so soft markers in strings are quieter.
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' || c == '\'' {
            let quote = c;
            out.push(' ');
            while let Some(n) = chars.next() {
                out.push(' ');
                if n == '\\' {
                    let _ = chars.next();
                    out.push(' ');
                    continue;
                }
                if n == quote {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Apply all static rules to a single source line.
pub fn apply_line(
    path: &str,
    line_no: u32,
    line: &str,
    in_test_file: bool,
    out: &mut Vec<Finding>,
) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return;
    }

    for marker in ["TODO", "FIXME", "XXX", "HACK"] {
        if let Some(idx) = word_boundary_match(line, marker) {
            push(
                out,
                path,
                line_no,
                col_at(line, idx),
                Severity::Error,
                "todo_marker",
                &format!("forbidden marker `{marker}`"),
            );
        }
    }

    for macro_name in ["unimplemented!", "todo!"] {
        if let Some(idx) = line.find(macro_name) {
            push(
                out,
                path,
                line_no,
                col_at(line, idx),
                Severity::Error,
                "unimplemented_macro",
                &format!("forbidden macro `{macro_name}`"),
            );
        }
    }

    let lower = line.to_ascii_lowercase();
    let is_comment = trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*');
    if is_comment {
        let stub_needles = [
            " stub",
            "//stub",
            "# stub",
            "#stub",
            "implement later",
            "not implemented",
        ];
        for needle in stub_needles {
            if lower.contains(needle.trim_start()) || lower.contains(needle) {
                // Prefer precise contains checks:
                if lower.contains("stub")
                    || lower.contains("implement later")
                    || lower.contains("not implemented")
                {
                    push(
                        out,
                        path,
                        line_no,
                        1,
                        Severity::Error,
                        "stub_comment",
                        "stub/placeholder comment",
                    );
                    break;
                }
            }
        }
    }

    let soft = strip_strings_rough(&lower);
    if soft.contains("coming soon") || soft.contains("placeholder") {
        push(
            out,
            path,
            line_no,
            1,
            Severity::Warning,
            "placeholder_soft",
            "soft placeholder language",
        );
    }

    if !in_test_file {
        for mock in [
            "jest.fn(",
            "sinon.stub(",
            "mockReturnValue",
            "mockResolvedValue",
            "vi.fn(",
        ] {
            if let Some(idx) = line.find(mock) {
                push(
                    out,
                    path,
                    line_no,
                    col_at(line, idx),
                    Severity::Error,
                    "mock_outside_test",
                    &format!("mock pattern `{mock}` outside test path"),
                );
            }
        }
    }

    // Single-line empty Ok/None stubs
    let compact: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
    if compact == "{Ok(())}" || compact == "{Ok(None)}" || compact == "{None}" {
        push(
            out,
            path,
            line_no,
            1,
            Severity::Warning,
            "empty_ok_stub",
            "suspicious empty Ok/None stub body",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_todo_marker() {
        let mut f = Vec::new();
        apply_line("src/a.rs", 3, "    // TODO: finish", false, &mut f);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule_id, "todo_marker");
        assert_eq!(f[0].severity, Severity::Error);
    }

    #[test]
    fn todo_in_identifier_not_flagged() {
        let mut f = Vec::new();
        apply_line("src/a.rs", 1, "let todolist = 1;", false, &mut f);
        assert!(f.iter().all(|x| x.rule_id != "todo_marker"));
    }

    #[test]
    fn unimplemented_macro() {
        let mut f = Vec::new();
        apply_line("src/a.rs", 1, "unimplemented!(\"x\")", false, &mut f);
        assert!(f.iter().any(|x| x.rule_id == "unimplemented_macro"));
    }
}
