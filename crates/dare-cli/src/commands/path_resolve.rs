//! Shared project-relative path resolution for CLI commands (validate / dag viz).

use std::path::Path;

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};

/// Resolve `--dag` / similar: default rel, or jail absolute/relative under `root`.
///
/// When `must_exist` is true, missing files → `NotFound`.
pub fn resolve_project_rel(
    root: &ProjectRoot,
    path: Option<&Path>,
    default_rel: &str,
    must_exist: bool,
) -> CoreResult<SafeRelativePath> {
    let Some(path) = path else {
        let rel = SafeRelativePath::new(default_rel)?;
        if must_exist {
            let joined = root.as_path().as_std_path().join(default_rel);
            if !joined.exists() {
                return Err(CoreError::not_found(format!(
                    "dag file not found: {default_rel}"
                )));
            }
        }
        return Ok(rel);
    };

    if path.is_absolute() {
        let path_canon = if path.exists() {
            std::fs::canonicalize(path).map_err(|e| CoreError::io(e.to_string()))?
        } else {
            path.to_path_buf()
        };
        let root_std = root.as_path().as_std_path();
        let root_canon = std::fs::canonicalize(root_std).unwrap_or_else(|_| root_std.to_path_buf());
        let rel = path_canon
            .strip_prefix(&root_canon)
            .or_else(|_| path_canon.strip_prefix(root_std))
            .map_err(|_| CoreError::invalid_input("path is outside project root"))?;
        let s = rel.to_string_lossy().replace('\\', "/");
        if s.is_empty() {
            return Err(CoreError::invalid_input("invalid path"));
        }
        if must_exist && !path.exists() {
            return Err(CoreError::not_found(format!(
                "dag file not found: {}",
                path.display()
            )));
        }
        return SafeRelativePath::new(&s);
    }

    let joined = root.as_path().as_std_path().join(path);
    if must_exist && !joined.exists() {
        return Err(CoreError::not_found(format!(
            "dag file not found: {}",
            path.display()
        )));
    }
    let s = path.to_string_lossy().replace('\\', "/");
    SafeRelativePath::new(&s)
}
