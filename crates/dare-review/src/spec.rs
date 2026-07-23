//! EXECUTION spec path helpers and section-3 file parse.

use crate::EXECUTION_DIR_REL;

pub fn task_id_is_path_safe(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

pub fn execution_spec_rel(task_id: &str) -> String {
    format!("{EXECUTION_DIR_REL}/{task_id}.md")
}

/// Extract project-relative paths from TASK-SPEC section 3 table (backtick paths).
pub fn parse_spec_files(markdown: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut in_section3 = false;
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            let title = trimmed.trim_start_matches('#').trim();
            in_section3 = title.starts_with('3')
                || title.to_ascii_lowercase().contains("arquivos a criar")
                || title.to_ascii_lowercase().contains("files to create")
                || title
                    .to_ascii_lowercase()
                    .contains("arquivos a criar / modificar");
            continue;
        }
        if !in_section3 {
            continue;
        }
        if trimmed.starts_with("## ") {
            break;
        }
        // Table row with backtick path
        if !trimmed.starts_with('|') {
            continue;
        }
        let mut parts = trimmed.split('`');
        // skip before first `
        let _ = parts.next();
        while let Some(inside) = parts.next() {
            let path = inside.trim();
            if looks_like_path(path) {
                let norm = path.replace('\\', "/");
                if !paths.iter().any(|p| p == &norm) {
                    paths.push(norm);
                }
            }
            let _ = parts.next(); // outside
        }
    }
    paths
}

fn looks_like_path(s: &str) -> bool {
    if s.is_empty() || s.contains(' ') {
        return false;
    }
    if s.contains("..") {
        return false;
    }
    s.contains('/') || s.contains('.') || s.ends_with(".rs") || s.ends_with(".ts")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_section_three_paths() {
        let md = r#"# TASK

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| CRIAR | `src/foo.rs` | x |
| MODIFICAR | `crates/a/src/lib.rs` | y |

## 4. IMPLEMENTAÇÃO
"#;
        let paths = parse_spec_files(md);
        assert_eq!(paths, vec!["src/foo.rs", "crates/a/src/lib.rs"]);
    }

    #[test]
    fn task_id_safe() {
        assert!(task_id_is_path_safe("mp032-001"));
        assert!(!task_id_is_path_safe("../x"));
        assert!(!task_id_is_path_safe(""));
    }
}
