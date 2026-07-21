//! Monorepo heuristics (workspace markers + child manifests).

use std::fs;
use std::path::{Path, PathBuf};

use crate::report::{MANIFEST_READ_CAP, MONOREPO_MAX_DEPTH, MONOREPO_MAX_ENTRIES};

fn to_posix_rel(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.to_string_lossy().replace('\\', "/")
}

fn cargo_has_workspace_table(root: &Path) -> bool {
    let path = root.join("Cargo.toml");
    if !path.is_file() {
        return false;
    }
    let Ok(bytes) = fs::read(&path) else {
        return false;
    };
    let slice = if bytes.len() > MANIFEST_READ_CAP {
        &bytes[..MANIFEST_READ_CAP]
    } else {
        &bytes
    };
    let text = String::from_utf8_lossy(slice);
    for line in text.lines() {
        let trimmed = line.split('#').next().unwrap_or("").trim();
        if trimmed == "[workspace]" || trimmed.starts_with("[workspace.") {
            return true;
        }
    }
    false
}

fn is_skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | "node_modules" | "target" | "vendor" | ".dare"
    )
}

fn is_manifest_name(name: &str) -> bool {
    matches!(name, "package.json" | "Cargo.toml" | "pyproject.toml")
}

/// Walk children depth 1..=MAX; count manifests outside root; collect up to 16 paths.
fn child_manifests(root: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut visited = 0usize;
    let mut stack: Vec<(PathBuf, usize)> = Vec::new();

    if let Ok(entries) = fs::read_dir(root) {
        for ent in entries.flatten() {
            let path = ent.path();
            if path.is_dir() {
                let name = ent.file_name().to_string_lossy().to_string();
                if is_skip_dir(&name) {
                    continue;
                }
                stack.push((path, 1));
            }
        }
    }

    while let Some((dir, depth)) = stack.pop() {
        if visited >= MONOREPO_MAX_ENTRIES {
            break;
        }
        visited += 1;

        if let Ok(entries) = fs::read_dir(&dir) {
            for ent in entries.flatten() {
                let path = ent.path();
                let name = ent.file_name().to_string_lossy().to_string();
                if path.is_file() && is_manifest_name(&name) {
                    found.push(to_posix_rel(root, &path));
                } else if path.is_dir() && depth < MONOREPO_MAX_DEPTH && !is_skip_dir(&name) {
                    stack.push((path, depth + 1));
                }
            }
        }
    }

    found.sort();
    found
}

/// Detect monorepo evidence under `root`.
pub fn detect_monorepo(root: &Path) -> (bool, Vec<String>) {
    let mut evidence: Vec<String> = Vec::new();

    for name in ["pnpm-workspace.yaml", "lerna.json", "nx.json"] {
        if root.join(name).is_file() {
            evidence.push(name.to_string());
        }
    }

    if cargo_has_workspace_table(root) {
        evidence.push("Cargo.toml".to_string());
    }

    let children = child_manifests(root);
    if children.len() >= 2 {
        for p in children.into_iter().take(16) {
            if !evidence.contains(&p) {
                evidence.push(p);
            }
        }
    }

    evidence.sort();
    evidence.dedup();
    let monorepo = !evidence.is_empty();
    // Child-only path: monorepo true only if ≥2 children OR explicit markers.
    // If evidence came only from markers or cargo workspace or ≥2 children — already set.
    // Edge: single child shouldn't set monorepo — child_manifests only adds when len>=2.
    (monorepo, evidence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_monorepo_pnpm_workspace() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("pnpm-workspace.yaml"),
            "packages:\n  - 'packages/*'\n",
        )
        .unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let (mono, ev) = detect_monorepo(dir.path());
        assert!(mono);
        assert!(ev.iter().any(|e| e == "pnpm-workspace.yaml"));
    }

    #[test]
    fn detect_not_monorepo_single_package() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let (mono, ev) = detect_monorepo(dir.path());
        assert!(!mono);
        assert!(ev.is_empty());
    }
}
