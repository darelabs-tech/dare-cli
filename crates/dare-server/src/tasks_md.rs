//! Read/update task status lines in `DARE/TASKS.md`.

use dare_core::fs::{atomic_write, read_to_string};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::Serialize;

pub const TASKS_REL: &str = "DARE/TASKS.md";
pub const MSG_TASK_NOT_FOUND: &str = "task not found";
pub const MSG_INVALID_STATUS: &str =
    "invalid status (expected PENDING|RUNNING|DONE|FAILED|SKIPPED)";
pub const MSG_PATH_ESCAPE: &str = "path escape forbidden";

const STATUS_TABLE: &[(&str, &str)] = &[
    ("PENDING", "⏳"),
    ("RUNNING", "🔄"),
    ("DONE", "✅"),
    ("FAILED", "❌"),
    ("SKIPPED", "⏭️"),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskView {
    pub id: String,
    pub status: String,
    pub line: String,
}

/// Reject path-escape characters in task ids before regex validation.
pub fn reject_path_escape_id(id: &str) -> CoreResult<()> {
    if id.contains("..") || id.contains('/') || id.contains('\\') {
        return Err(CoreError::invalid_input(MSG_PATH_ESCAPE));
    }
    Ok(())
}

/// Validate task id: `^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$`.
pub fn validate_task_id(id: &str) -> CoreResult<()> {
    reject_path_escape_id(id)?;
    let bytes = id.as_bytes();
    if bytes.is_empty() || bytes.len() > 128 {
        return Err(CoreError::invalid_input(format!("invalid task id: {id}")));
    }
    let first = bytes[0];
    if !first.is_ascii_alphanumeric() {
        return Err(CoreError::invalid_input(format!("invalid task id: {id}")));
    }
    for &b in &bytes[1..] {
        let ok = b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':' | b'-');
        if !ok {
            return Err(CoreError::invalid_input(format!("invalid task id: {id}")));
        }
    }
    Ok(())
}

pub fn normalize_status(status: &str) -> CoreResult<&'static str> {
    for &(word, _) in STATUS_TABLE {
        if status == word {
            return Ok(word);
        }
    }
    Err(CoreError::invalid_input(MSG_INVALID_STATUS))
}

fn emoji_for(status: &str) -> &'static str {
    STATUS_TABLE
        .iter()
        .find(|(w, _)| *w == status)
        .map(|(_, e)| *e)
        .unwrap_or("✅")
}

fn line_matches_id(line: &str, id: &str) -> bool {
    let pipe = format!("| {id} |");
    let tick = format!("`{id}`");
    if line.contains(&pipe) || line.contains(&tick) {
        return true;
    }
    // Token in a markdown table cell: | id | or leading/trailing pipes around id.
    for cell in line.split('|') {
        if cell.trim() == id {
            return true;
        }
    }
    false
}

fn detect_status(line: &str) -> String {
    for &(word, emoji) in STATUS_TABLE {
        if line.contains(emoji) || line.contains(word) {
            return word.to_string();
        }
    }
    "PENDING".to_string()
}

fn rewrite_status_line(line: &str, status: &str) -> String {
    let emoji = emoji_for(status);
    let mut out = line.to_string();
    let mut replaced = false;

    for &(_, old_emoji) in STATUS_TABLE {
        if out.contains(old_emoji) {
            out = out.replace(old_emoji, emoji);
            replaced = true;
            break;
        }
    }
    for &(word, _) in STATUS_TABLE {
        if out.contains(word) {
            out = out.replace(word, status);
            replaced = true;
            break;
        }
    }
    if !replaced {
        out.push(' ');
        out.push_str(emoji);
        out.push(' ');
        out.push_str(status);
    }
    out
}

fn tasks_rel() -> CoreResult<SafeRelativePath> {
    SafeRelativePath::new(TASKS_REL)
}

pub fn get_task_view(root: &ProjectRoot, id: &str) -> CoreResult<TaskView> {
    validate_task_id(id)?;
    let rel = tasks_rel()?;
    let text = read_to_string(root, &rel).map_err(|e| match e {
        CoreError::NotFound(_) => CoreError::not_found(format!("{MSG_TASK_NOT_FOUND}: {id}")),
        other => other,
    })?;
    for line in text.lines() {
        if line_matches_id(line, id) {
            return Ok(TaskView {
                id: id.to_string(),
                status: detect_status(line),
                line: line.to_string(),
            });
        }
    }
    Err(CoreError::not_found(format!("{MSG_TASK_NOT_FOUND}: {id}")))
}

pub fn put_task_status(root: &ProjectRoot, id: &str, status: &str) -> CoreResult<TaskView> {
    validate_task_id(id)?;
    let status = normalize_status(status)?;
    let rel = tasks_rel()?;
    let text = read_to_string(root, &rel).map_err(|e| match e {
        CoreError::NotFound(_) => CoreError::not_found(format!("{MSG_TASK_NOT_FOUND}: {id}")),
        other => other,
    })?;

    let mut found = false;
    let mut new_lines: Vec<String> = Vec::new();
    let mut updated_line = String::new();
    for line in text.lines() {
        if !found && line_matches_id(line, id) {
            let rewritten = rewrite_status_line(line, status);
            updated_line = rewritten.clone();
            new_lines.push(rewritten);
            found = true;
        } else {
            new_lines.push(line.to_string());
        }
    }
    if !found {
        return Err(CoreError::not_found(format!("{MSG_TASK_NOT_FOUND}: {id}")));
    }

    let mut body = new_lines.join("\n");
    if text.ends_with('\n') {
        body.push('\n');
    }
    atomic_write(root, &rel, body.as_bytes())?;

    Ok(TaskView {
        id: id.to_string(),
        status: status.to_string(),
        line: updated_line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn path_escape_rejected() {
        assert!(reject_path_escape_id("../x").is_err());
        assert!(reject_path_escape_id("a/b").is_err());
        assert!(reject_path_escape_id(r"a\b").is_err());
    }

    #[test]
    fn put_roundtrip_unit() {
        let dir = tempdir().unwrap();
        let dare = dir.path().join("DARE");
        std::fs::create_dir_all(&dare).unwrap();
        std::fs::write(
            dare.join("TASKS.md"),
            "| id | title | status |\n| mp051-001 | Skeleton | ⏳ PENDING |\n",
        )
        .unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let view = put_task_status(&root, "mp051-001", "DONE").unwrap();
        assert_eq!(view.status, "DONE");
        assert!(view.line.contains("✅"));
        let again = get_task_view(&root, "mp051-001").unwrap();
        assert_eq!(again.status, "DONE");
    }
}
