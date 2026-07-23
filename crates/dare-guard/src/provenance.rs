//! Path provenance classification.

use serde::{Deserialize, Serialize};
use std::path::{Component, Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provenance {
    Human,
    Agent,
    External,
}

/// Classify a project-relative path.
pub fn classify_provenance(rel: &str, trusted_paths: &[String]) -> Provenance {
    let norm = normalize_rel(rel);
    if is_under_agent(&norm) {
        return Provenance::Agent;
    }
    for t in trusted_paths {
        let tn = normalize_rel(t);
        if tn.is_empty() {
            continue;
        }
        if norm == tn || norm.starts_with(&(tn.clone() + "/")) {
            return Provenance::Human;
        }
    }
    Provenance::External
}

pub fn default_trusted_paths() -> Vec<String> {
    vec!["DARE/".into(), "dare.config.json".into()]
}

pub fn is_control_path(rel: &str) -> bool {
    let norm = normalize_rel(rel);
    norm == "dare.config.json" || norm.starts_with("DARE/") || norm == "DARE"
}

fn is_under_agent(norm: &str) -> bool {
    norm.starts_with(".dare/agent-worktrees/") || norm.starts_with(".dare/agent-worktrees")
}

fn normalize_rel(p: &str) -> String {
    let path = Path::new(p);
    let mut parts = Vec::new();
    for c in path.components() {
        match c {
            Component::Normal(s) => parts.push(s.to_string_lossy().replace('\\', "/")),
            Component::CurDir => {}
            _ => {}
        }
    }
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_is_human() {
        let t = default_trusted_paths();
        assert_eq!(classify_provenance("DARE/DESIGN.md", &t), Provenance::Human);
        assert_eq!(
            classify_provenance("dare.config.json", &t),
            Provenance::Human
        );
    }

    #[test]
    fn agent_worktree() {
        let t = default_trusted_paths();
        assert_eq!(
            classify_provenance(".dare/agent-worktrees/t-1/foo.rs", &t),
            Provenance::Agent
        );
    }

    #[test]
    fn other_is_external() {
        let t = default_trusted_paths();
        assert_eq!(classify_provenance("src/main.rs", &t), Provenance::External);
    }
}
