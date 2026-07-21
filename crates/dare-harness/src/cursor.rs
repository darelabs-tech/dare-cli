//! Cursor IDE adapter (microplano 012).

use dare_assets::{
    load_capability_matrix_from_str, render_claude_command, validate_capability_matrix,
    EmbeddedAssets,
};
use dare_core::fs::{atomic_write, read_to_string};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};

const MANAGED_PREFIX: &str = "<!-- dare:managed";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorDetect {
    pub cursor_dir: bool,
    pub cursorrules: bool,
}

fn exists(root: &ProjectRoot, rel: &str) -> CoreResult<bool> {
    let path = SafeRelativePath::new(rel)?;
    Ok(root.resolve(&path)?.as_path().exists())
}

pub fn detect_cursor(root: &ProjectRoot) -> CoreResult<CursorDetect> {
    Ok(CursorDetect {
        cursor_dir: exists(root, ".cursor")?,
        cursorrules: exists(root, ".cursorrules")?,
    })
}

fn is_managed(bytes: &[u8]) -> bool {
    String::from_utf8_lossy(bytes)
        .lines()
        .next()
        .map(|l| l.trim_start().starts_with(MANAGED_PREFIX))
        .unwrap_or(false)
}

fn should_write(root: &ProjectRoot, rel: &SafeRelativePath, force: bool) -> CoreResult<bool> {
    if force {
        return Ok(true);
    }
    let abs = root.resolve(rel)?;
    if !abs.as_path().exists() {
        return Ok(true);
    }
    let existing = read_to_string(root, rel)?;
    Ok(is_managed(existing.as_bytes()))
}

fn load_matrix() -> CoreResult<dare_assets::CapabilityMatrix> {
    let file = EmbeddedAssets::get("capability-matrix.yml")
        .ok_or_else(|| CoreError::config("asset missing: capability-matrix.yml"))?;
    let yaml = std::str::from_utf8(file.data.as_ref())
        .map_err(|e| CoreError::config(format!("invalid capability-matrix encoding: {e}")))?;
    let matrix = load_capability_matrix_from_str(yaml)?;
    validate_capability_matrix(&matrix)?;
    Ok(matrix)
}

pub fn generate_cursorrules(root: &ProjectRoot, force: bool) -> CoreResult<()> {
    let rel = SafeRelativePath::new(".cursorrules")?;
    if !should_write(root, &rel, force)? {
        return Ok(());
    }
    let body = format!(
        "{MANAGED_PREFIX} cursorrules -->\n# DARE Cursor rules\n\n\
         Follow DARE Design → Blueprint → Tasks → Execute. Prefer slash commands.\n"
    );
    atomic_write(root, &rel, body.as_bytes())
}

pub fn install_cursor_commands(root: &ProjectRoot, force: bool) -> CoreResult<usize> {
    let matrix = load_matrix()?;
    let mut written = 0usize;
    for cap in &matrix.capabilities {
        let Some(out) = cap.outputs.cursor.as_deref() else {
            continue;
        };
        let rel = SafeRelativePath::new(out)?;
        if !should_write(root, &rel, force)? {
            continue;
        }
        let rendered = render_claude_command(cap);
        let body = format!("{MANAGED_PREFIX} capability={} -->\n{rendered}", cap.id);
        atomic_write(root, &rel, body.as_bytes())?;
        written += 1;
    }
    Ok(written)
}

pub fn validate_cursor_install(root: &ProjectRoot) -> CoreResult<usize> {
    let matrix = load_matrix()?;
    let mut ok = 0usize;
    let mut missing = Vec::new();
    for cap in &matrix.capabilities {
        let Some(out) = cap.outputs.cursor.as_deref() else {
            continue;
        };
        let rel = SafeRelativePath::new(out)?;
        if root.resolve(&rel)?.as_path().is_file() {
            ok += 1;
        } else {
            missing.push(out.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(CoreError::config(format!(
            "cursor commands missing ({}): {}",
            missing.len(),
            missing.into_iter().take(5).collect::<Vec<_>>().join(", ")
        )));
    }
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn detect_empty_and_generate_managed() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let d = detect_cursor(&root).unwrap();
        assert!(!d.cursor_dir);
        assert!(!d.cursorrules);
        generate_cursorrules(&root, false).unwrap();
        let d2 = detect_cursor(&root).unwrap();
        assert!(d2.cursorrules);
        let content =
            read_to_string(&root, &SafeRelativePath::new(".cursorrules").unwrap()).unwrap();
        assert!(content.starts_with(MANAGED_PREFIX));
        assert!(content.contains("DARE Cursor rules"));
    }

    #[test]
    fn preserve_unmanaged_cursorrules() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new(".cursorrules").unwrap();
        atomic_write(&root, &rel, b"# custom cursorrules\nkeep\n").unwrap();
        generate_cursorrules(&root, false).unwrap();
        let content = read_to_string(&root, &rel).unwrap();
        assert!(content.contains("custom cursorrules"));
        assert!(!content.starts_with(MANAGED_PREFIX));
    }

    #[test]
    fn install_validate_force() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        generate_cursorrules(&root, true).unwrap();
        assert_eq!(install_cursor_commands(&root, true).unwrap(), 49);
        assert_eq!(validate_cursor_install(&root).unwrap(), 49);
        let n2 = install_cursor_commands(&root, false).unwrap();
        assert_eq!(n2, 49); // managed rewritten
    }

    #[test]
    fn preserve_unmanaged_command() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new(".cursor/commands/dare-design.md").unwrap();
        atomic_write(&root, &rel, b"# custom cursor command\n").unwrap();
        let n = install_cursor_commands(&root, false).unwrap();
        assert_eq!(n, 48);
        let content = read_to_string(&root, &rel).unwrap();
        assert!(content.contains("custom cursor command"));
    }

    #[test]
    fn validate_reports_missing_sample() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let msg = validate_cursor_install(&root).unwrap_err().to_string();
        assert!(msg.contains("cursor commands missing"));
        assert!(msg.contains("missing (49)") || msg.contains("(49):"));
    }
}
