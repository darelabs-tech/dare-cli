//! Local-only diagnostic bundle. Strict allowlist: generated report only.
use std::path::{Path, PathBuf};

use dare_core::{CoreError, CoreResult};
use serde::Serialize;

use super::info::{collect_info, Diagnostic};
use super::lock_diagnostics::{state_lock_diagnostic, LockDiagnostic};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundlePreview {
    pub schema_version: u32,
    pub included: Vec<&'static str>,
    pub excluded: Vec<&'static str>,
    pub diagnostics: Vec<Diagnostic>,
    pub state_lock: LockDiagnostic,
}

pub fn preview(root: &Path) -> CoreResult<BundlePreview> {
    let report = collect_info(root)?;
    Ok(BundlePreview {
        schema_version: 1,
        included: vec!["diagnostics.json"],
        excluded: vec![
            "dare.config.json",
            ".dare/state.json",
            ".env",
            "source files",
            "worktree contents",
            "raw logs",
        ],
        diagnostics: report.diagnostics,
        state_lock: state_lock_diagnostic(root),
    })
}

/// Export only generated, fixed-schema diagnostic codes. No arbitrary file contents.
pub fn export(root: &Path, destination: &Path) -> CoreResult<PathBuf> {
    if destination.file_name().and_then(|s| s.to_str()) != Some("dare-support.json") {
        return Err(CoreError::invalid_input("destination filename must be dare-support.json"));
    }
    let parent = destination.parent().ok_or_else(|| CoreError::invalid_input("destination parent required"))?;
    let canonical_parent = parent.canonicalize().map_err(|_| CoreError::invalid_input("destination parent must exist"))?;
    let canonical_root = root.canonicalize().map_err(|_| CoreError::invalid_input("project root must exist"))?;
    if canonical_parent.starts_with(&canonical_root) {
        return Err(CoreError::invalid_input("support export must be outside the project directory"));
    }
    if destination.exists() {
        return Err(CoreError::invalid_input("destination already exists; refusing overwrite"));
    }
    let data = serde_json::to_vec_pretty(&preview(root)?)
        .map_err(|_| CoreError::invalid_input("cannot serialize support report"))?;
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|_| CoreError::invalid_input("cannot create support report"))?;
    file.write_all(&data).map_err(|_| CoreError::invalid_input("cannot write support report"))?;
    Ok(destination.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn preview_never_copies_secret_files() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("dare.config.json"), "{secret_password_123").unwrap();
        let p = preview(dir.path()).unwrap();
        let json = serde_json::to_string(&p).unwrap();
        assert!(!json.contains("secret_password_123"));
        assert_eq!(p.included, vec!["diagnostics.json"]);
    }

    #[test]
    fn export_requires_explicit_outside_destination() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("dare.config.json"), "{}").unwrap();
        let inside = dir.path().join("dare-support.json");
        assert!(export(dir.path(), &inside).is_err());
        let output = tempdir().unwrap();
        let target = output.path().join("dare-support.json");
        export(dir.path(), &target).unwrap();
        assert!(target.exists());
        assert!(export(dir.path(), &target).is_err());
    }
}
