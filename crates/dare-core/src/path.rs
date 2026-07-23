//! Project jail paths: SafeRelativePath, ProjectRoot, SafeAbsolutePath.

use std::path::{Component, Path};

use camino::{Utf8Path, Utf8PathBuf};

use crate::error::{CoreError, CoreResult};

pub const PATH_ESCAPE_MSG: &str = "path must be relative and stay within the project";

fn escape_err() -> CoreError {
    CoreError::invalid_input(PATH_ESCAPE_MSG)
}

/// Relative path stored with `/` separators; no `..`, no absolute forms.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SafeRelativePath {
    inner: String,
}

impl SafeRelativePath {
    pub fn new(raw: &str) -> CoreResult<Self> {
        if raw.is_empty() || raw.contains('\0') {
            return Err(escape_err());
        }
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(escape_err());
        }
        // Reject Windows UNC / drive / absolute POSIX early
        if trimmed.starts_with('/')
            || trimmed.starts_with('\\')
            || trimmed.starts_with("//")
            || trimmed.starts_with("\\\\")
        {
            return Err(escape_err());
        }
        if trimmed.len() >= 2 {
            let bytes = trimmed.as_bytes();
            if bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
                return Err(escape_err());
            }
        }

        let normalized = trimmed.replace('\\', "/");
        let path = Path::new(&normalized);
        let mut parts: Vec<&str> = Vec::new();
        for comp in path.components() {
            match comp {
                Component::Normal(s) => {
                    let s = s.to_str().ok_or_else(escape_err)?;
                    if s == ".." {
                        return Err(escape_err());
                    }
                    if s != "." {
                        parts.push(s);
                    }
                }
                Component::CurDir => {}
                Component::ParentDir => return Err(escape_err()),
                Component::RootDir | Component::Prefix(_) => return Err(escape_err()),
            }
        }
        if parts.is_empty() {
            // Allow "." (and "././.") to mean the project root itself (cwd jail).
            let only_curdir = normalized
                .split('/')
                .filter(|s| !s.is_empty())
                .all(|s| s == ".");
            if only_curdir {
                return Ok(Self {
                    inner: ".".to_string(),
                });
            }
            return Err(escape_err());
        }
        Ok(Self {
            inner: parts.join("/"),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn to_path_buf(&self) -> Utf8PathBuf {
        Utf8PathBuf::from(&self.inner)
    }
}

/// Absolute path verified to stay within a [`ProjectRoot`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeAbsolutePath {
    path: Utf8PathBuf,
}

impl SafeAbsolutePath {
    pub fn as_path(&self) -> &Utf8Path {
        &self.path
    }

    pub fn to_posix(&self) -> String {
        to_posix(&self.path)
    }
}

/// Canonical project root directory (jail).
#[derive(Debug, Clone)]
pub struct ProjectRoot {
    root: Utf8PathBuf,
}

impl ProjectRoot {
    pub fn new(dir: impl AsRef<Path>) -> CoreResult<Self> {
        let dir = dir.as_ref();
        if !dir.is_dir() {
            return Err(CoreError::not_found(format!(
                "project root is not a directory: {}",
                dir.display()
            )));
        }
        let canon = std::fs::canonicalize(dir).map_err(|e| CoreError::io(e.to_string()))?;
        let root = Utf8PathBuf::from_path_buf(canon)
            .map_err(|_| CoreError::invalid_input("project root path is not valid UTF-8"))?;
        Ok(Self { root })
    }

    pub fn as_path(&self) -> &Utf8Path {
        &self.root
    }

    pub fn to_posix(&self) -> String {
        to_posix(&self.root)
    }

    pub fn contains(&self, candidate: &Utf8Path) -> CoreResult<bool> {
        let cand = if candidate.exists() {
            let c = std::fs::canonicalize(candidate.as_std_path())
                .map_err(|e| CoreError::io(e.to_string()))?;
            Utf8PathBuf::from_path_buf(c)
                .map_err(|_| CoreError::invalid_input("path is not valid UTF-8"))?
        } else {
            candidate.to_path_buf()
        };
        Ok(is_within(&self.root, &cand))
    }

    /// Resolve relative path under jail (symlink escape denied).
    pub fn resolve(&self, rel: &SafeRelativePath) -> CoreResult<SafeAbsolutePath> {
        if rel.as_str() == "." {
            return Ok(SafeAbsolutePath {
                path: self.root.clone(),
            });
        }
        let joined = self.root.as_std_path().join(rel.as_str());
        let verified = verify_within_root(&self.root, &joined)?;
        Ok(SafeAbsolutePath { path: verified })
    }
}

/// Normalize separators to `/` without resolving `.`/`..`.
pub fn to_posix(path: &Utf8Path) -> String {
    path.as_str().replace('\\', "/")
}

fn strip_verbatim_prefix(p: &Utf8Path) -> Utf8PathBuf {
    let s = p.as_str();
    // Windows \\?\C:\... or //?/C:/...
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        return Utf8PathBuf::from(rest);
    }
    if let Some(rest) = s.strip_prefix("//?/") {
        return Utf8PathBuf::from(rest);
    }
    p.to_path_buf()
}

fn is_within(root: &Utf8Path, candidate: &Utf8Path) -> bool {
    let root = strip_verbatim_prefix(root);
    let cand = strip_verbatim_prefix(candidate);
    let root_s = to_posix(&root).to_ascii_lowercase();
    let cand_s = to_posix(&cand).to_ascii_lowercase();
    if cand_s == root_s {
        return true;
    }
    let prefix = if root_s.ends_with('/') {
        root_s
    } else {
        format!("{root_s}/")
    };
    cand_s.starts_with(&prefix)
}

fn verify_within_root(root: &Utf8Path, joined: &Path) -> CoreResult<Utf8PathBuf> {
    if joined.exists() {
        let canon = std::fs::canonicalize(joined).map_err(|e| CoreError::io(e.to_string()))?;
        let utf = Utf8PathBuf::from_path_buf(canon)
            .map_err(|_| CoreError::invalid_input("path is not valid UTF-8"))?;
        if !is_within(root, &utf) {
            return Err(escape_err());
        }
        return Ok(utf);
    }

    // Path does not exist yet: walk ancestors
    let mut ancestor = joined.to_path_buf();
    let mut missing: Vec<std::ffi::OsString> = Vec::new();
    while !ancestor.exists() {
        let file_name = ancestor.file_name().ok_or_else(escape_err)?.to_os_string();
        missing.push(file_name);
        ancestor = ancestor.parent().ok_or_else(escape_err)?.to_path_buf();
    }
    let ancestor_canon =
        std::fs::canonicalize(&ancestor).map_err(|e| CoreError::io(e.to_string()))?;
    let ancestor_utf = Utf8PathBuf::from_path_buf(ancestor_canon)
        .map_err(|_| CoreError::invalid_input("path is not valid UTF-8"))?;
    if !is_within(root, &ancestor_utf) {
        return Err(escape_err());
    }

    // Rebuild path from canonical ancestor + missing components (reject .. already filtered)
    let mut out = ancestor_utf;
    for part in missing.into_iter().rev() {
        let s = part
            .to_str()
            .ok_or_else(|| CoreError::invalid_input("path component is not valid UTF-8"))?;
        if s == ".." || s.contains('\0') {
            return Err(escape_err());
        }
        out = out.join(s);
        // If an intermediate suddenly exists as symlink outside, catch on final exists check
        if out.exists() {
            let c = std::fs::canonicalize(out.as_std_path())
                .map_err(|e| CoreError::io(e.to_string()))?;
            let cu = Utf8PathBuf::from_path_buf(c)
                .map_err(|_| CoreError::invalid_input("path is not valid UTF-8"))?;
            if !is_within(root, &cu) {
                return Err(escape_err());
            }
            out = cu;
        }
    }
    if !is_within(root, &out) {
        return Err(escape_err());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn safe_relative_rejects_dotdot_and_absolute() {
        assert!(SafeRelativePath::new("../x").is_err());
        assert!(SafeRelativePath::new("a/../../b").is_err());
        assert!(SafeRelativePath::new("/abs").is_err());
        assert!(SafeRelativePath::new(r"C:\x").is_err());
        assert!(SafeRelativePath::new(r"\\server\share").is_err());
        assert!(SafeRelativePath::new("").is_err());
        assert!(SafeRelativePath::new("a\0b").is_err());
        let ok = SafeRelativePath::new("foo/bar").unwrap();
        assert_eq!(ok.as_str(), "foo/bar");
        let ok2 = SafeRelativePath::new(r"foo\bar").unwrap();
        assert_eq!(ok2.as_str(), "foo/bar");
        let root_rel = SafeRelativePath::new(".").unwrap();
        assert_eq!(root_rel.as_str(), ".");
    }

    #[test]
    fn to_posix_normalizes_backslashes() {
        let p = Utf8PathBuf::from(r"a\b\c");
        assert_eq!(to_posix(&p), "a/b/c");
    }

    #[test]
    fn resolve_keeps_path_within_root() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("nested/file.txt").unwrap();
        let abs = root.resolve(&rel).unwrap();
        assert!(abs.as_path().as_str().contains("nested"));
        assert!(root.contains(abs.as_path()).unwrap());

        // lexical escape already blocked by SafeRelativePath
        assert!(SafeRelativePath::new("../outside").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_rejected() {
        use std::os::unix::fs::symlink;
        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let link = dir.path().join("escape");
        symlink(outside.path(), &link).unwrap();
        let rel = SafeRelativePath::new("escape").unwrap();
        let err = root.resolve(&rel).unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)), "{err:?}");
        assert!(err.message().contains(PATH_ESCAPE_MSG));
    }

    #[cfg(unix)]
    #[test]
    fn symlink_internal_ok() {
        use std::os::unix::fs::symlink;
        let dir = tempdir().unwrap();
        let target = dir.path().join("real");
        std::fs::create_dir(&target).unwrap();
        let link = dir.path().join("link");
        symlink(&target, &link).unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("link").unwrap();
        assert!(root.resolve(&rel).is_ok());
    }
}
