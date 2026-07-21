//! Stack family detection (node / rust / python) and conflicts.

use std::path::Path;

use crate::report::{StackConflict, StackHit};

fn push_if_file(evidence: &mut Vec<String>, root: &Path, name: &str) {
    if root.join(name).is_file() {
        evidence.push(name.to_string());
    }
}

/// Detect MUST stacks at `root` (not children). Returns sorted stacks + conflicts.
pub fn detect_stacks(root: &Path) -> (Vec<StackHit>, Vec<StackConflict>) {
    let mut stacks = Vec::new();

    if root.join("package.json").is_file() {
        let mut evidence = vec!["package.json".to_string()];
        push_if_file(&mut evidence, root, "pnpm-lock.yaml");
        push_if_file(&mut evidence, root, "yarn.lock");
        push_if_file(&mut evidence, root, "package-lock.json");
        evidence.sort();
        stacks.push(StackHit {
            id: "node".to_string(),
            family: "node".to_string(),
            confidence: "high".to_string(),
            evidence,
        });
    }

    if root.join("Cargo.toml").is_file() {
        stacks.push(StackHit {
            id: "rust".to_string(),
            family: "rust".to_string(),
            confidence: "high".to_string(),
            evidence: vec!["Cargo.toml".to_string()],
        });
    }

    let has_pyproject = root.join("pyproject.toml").is_file();
    let has_requirements = root.join("requirements.txt").is_file();
    let has_setup = root.join("setup.py").is_file();
    if has_pyproject || has_requirements || has_setup {
        let mut evidence = Vec::new();
        if has_pyproject {
            evidence.push("pyproject.toml".to_string());
        }
        if has_requirements {
            evidence.push("requirements.txt".to_string());
        }
        if has_setup {
            evidence.push("setup.py".to_string());
        }
        evidence.sort();
        let confidence = if has_pyproject { "high" } else { "medium" };
        stacks.push(StackHit {
            id: "python".to_string(),
            family: "python".to_string(),
            confidence: confidence.to_string(),
            evidence,
        });
    }

    stacks.sort_by(|a, b| a.id.cmp(&b.id));

    let conflicts = if stacks.len() >= 2 {
        let mut kinds: Vec<String> = stacks.iter().map(|s| s.family.clone()).collect();
        kinds.sort();
        let mut evidence: Vec<String> = stacks.iter().flat_map(|s| s.evidence.clone()).collect();
        evidence.sort();
        evidence.dedup();
        vec![StackConflict { kinds, evidence }]
    } else {
        Vec::new()
    };

    (stacks, conflicts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detect_node_fixture() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let (stacks, conflicts) = detect_stacks(dir.path());
        assert_eq!(stacks.len(), 1);
        assert_eq!(stacks[0].id, "node");
        assert!(conflicts.is_empty());
    }

    #[test]
    fn detect_rust_fixture() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();
        let (stacks, _) = detect_stacks(dir.path());
        assert_eq!(stacks[0].id, "rust");
    }

    #[test]
    fn detect_python_fixture() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("pyproject.toml"), "[project]\nname=\"x\"\n").unwrap();
        let (stacks, _) = detect_stacks(dir.path());
        assert_eq!(stacks[0].id, "python");
        assert_eq!(stacks[0].confidence, "high");
    }

    #[test]
    fn detect_conflict_node_rust() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();
        let (stacks, conflicts) = detect_stacks(dir.path());
        assert_eq!(stacks.len(), 2);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].kinds, vec!["node", "rust"]);
        assert!(conflicts[0].evidence.contains(&"Cargo.toml".to_string()));
        assert!(conflicts[0].evidence.contains(&"package.json".to_string()));
    }

    #[test]
    fn stacks_and_evidence_sorted() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        fs::write(dir.path().join("yarn.lock"), "").unwrap();
        fs::write(dir.path().join("package-lock.json"), "{}").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();
        let (stacks, conflicts) = detect_stacks(dir.path());
        let ids: Vec<_> = stacks.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["node", "rust"]);
        let node = stacks.iter().find(|s| s.id == "node").unwrap();
        let mut sorted = node.evidence.clone();
        sorted.sort();
        assert_eq!(node.evidence, sorted);
        assert_eq!(conflicts[0].kinds, {
            let mut k = conflicts[0].kinds.clone();
            k.sort();
            k
        });
    }
}
