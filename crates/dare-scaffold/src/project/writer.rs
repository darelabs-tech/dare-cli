//! File-system writer used by the npm-parity generator.
//!
//! Records every directory and file it creates so a failed `dare init` can
//! remove exactly what it created (fix B-09) and nothing that pre-existed.

use std::fs;
use std::path::{Path, PathBuf};

use dare_core::{CoreError, CoreResult};

/// Writes under a root directory and journals created paths.
#[derive(Debug, Default)]
pub struct ProjectWriter {
    created_dirs: Vec<PathBuf>,
    created_files: Vec<PathBuf>,
}

fn io_err(action: &str, path: &Path, err: std::io::Error) -> CoreError {
    CoreError::io(format!("{action} {}: {err}", path.display()))
}

impl ProjectWriter {
    pub fn new() -> Self {
        Self::default()
    }

    /// `fs.ensureDir` — creates missing ancestors and journals them.
    pub fn ensure_dir(&mut self, dir: &Path) -> CoreResult<()> {
        if dir.is_dir() {
            return Ok(());
        }
        let mut missing = Vec::new();
        let mut cur = Some(dir);
        while let Some(p) = cur {
            if p.as_os_str().is_empty() || p.exists() {
                break;
            }
            missing.push(p.to_path_buf());
            cur = p.parent();
        }
        fs::create_dir_all(dir).map_err(|e| io_err("create dir", dir, e))?;
        missing.reverse();
        self.created_dirs.extend(missing);
        Ok(())
    }

    /// `fs.writeFile` — overwrites; creates parent directories.
    pub fn write(&mut self, path: &Path, contents: impl AsRef<[u8]>) -> CoreResult<()> {
        if let Some(parent) = path.parent() {
            self.ensure_dir(parent)?;
        }
        let existed = path.exists();
        fs::write(path, contents).map_err(|e| io_err("write", path, e))?;
        if !existed {
            self.created_files.push(path.to_path_buf());
        }
        Ok(())
    }

    /// Write and mark executable (0o755) on Unix; no-op permission on Windows.
    pub fn write_executable(&mut self, path: &Path, contents: impl AsRef<[u8]>) -> CoreResult<()> {
        self.write(path, contents)?;
        set_executable(path)
    }

    /// Record a file produced by an external process (e.g. `npm install`).
    pub fn note_external_dir(&mut self, dir: &Path) {
        if !self.created_dirs.iter().any(|d| d == dir) {
            self.created_dirs.push(dir.to_path_buf());
        }
    }

    pub fn created_files(&self) -> &[PathBuf] {
        &self.created_files
    }

    pub fn created_dirs(&self) -> &[PathBuf] {
        &self.created_dirs
    }

    /// Remove everything this writer created (files first, then dirs deepest-first).
    pub fn rollback(&self) {
        for f in self.created_files.iter().rev() {
            let _ = fs::remove_file(f);
        }
        let mut dirs = self.created_dirs.clone();
        dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
        for d in dirs {
            // Directories created by this run may also hold output from external
            // scaffolders (npm install, rails new); remove them wholesale.
            let _ = fs::remove_dir_all(&d);
        }
    }
}

#[cfg(unix)]
fn set_executable(path: &Path) -> CoreResult<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .map_err(|e| io_err("chmod", path, e))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> CoreResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollback_removes_only_created_paths() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let keep = tmp.path().join("existing.txt");
        fs::write(&keep, "keep").expect("seed");

        let mut w = ProjectWriter::new();
        w.write(&tmp.path().join("a/b/c.txt"), "x").expect("write");
        w.write(&keep, "overwritten").expect("overwrite");
        w.ensure_dir(&tmp.path().join("empty")).expect("dir");
        w.rollback();

        assert!(keep.exists(), "pre-existing file must survive rollback");
        assert!(!tmp.path().join("a").exists());
        assert!(!tmp.path().join("empty").exists());
    }
}
