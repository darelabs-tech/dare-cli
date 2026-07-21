//! Project root discovery (walk-up markers).

use std::path::{Path, PathBuf};

const FILE_MARKERS: &[&str] = &[
    "dare.config.json",
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "requirements.txt",
    "setup.py",
];

fn has_project_marker(dir: &Path) -> bool {
    for name in FILE_MARKERS {
        if dir.join(name).is_file() {
            return true;
        }
    }
    dir.join("DARE").is_dir()
}

fn absolutize(start: &Path) -> PathBuf {
    if let Ok(c) = std::fs::canonicalize(start) {
        return c;
    }
    if start.is_absolute() {
        return start.to_path_buf();
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(start)
}

/// Walk up from `start` until a project marker is found.
///
/// Markers: `dare.config.json`, `DARE/`, `package.json`, `Cargo.toml`,
/// `pyproject.toml`, `requirements.txt`, `setup.py`.
/// `.git` alone is **not** a project-root marker.
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut cur = absolutize(start);
    loop {
        if has_project_marker(&cur) {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn find_root_walks_up_to_package_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let nested = dir.path().join("pkgs").join("a");
        fs::create_dir_all(&nested).unwrap();
        let root = find_project_root(&nested).expect("root");
        assert_eq!(
            root.canonicalize().unwrap(),
            dir.path().canonicalize().unwrap()
        );
    }

    #[test]
    fn find_root_none_on_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(find_project_root(dir.path()).is_none());
    }
}
