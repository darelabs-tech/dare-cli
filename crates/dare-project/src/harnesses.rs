//! Map dare-harness detect_* → HarnessHit (always 4, sorted by id).

use dare_core::{CoreResult, ProjectRoot};
use dare_harness::{detect_antigravity, detect_claude, detect_codex, detect_cursor};

use crate::report::HarnessHit;

fn push_ev(ev: &mut Vec<String>, present: bool, name: &str) {
    if present {
        ev.push(name.to_string());
    }
}

/// Always returns four harness entries sorted by `id`.
pub fn detect_harnesses(root: &ProjectRoot) -> CoreResult<Vec<HarnessHit>> {
    let claude = detect_claude(root)?;
    let cursor = detect_cursor(root)?;
    let codex = detect_codex(root)?;
    let ag = detect_antigravity(root)?;

    let mut hits = Vec::with_capacity(4);

    {
        let present =
            ag.antigravityrules || ag.antigravity_dir || ag.agents_skills || ag.agents_workflows;
        let mut evidence = Vec::new();
        push_ev(&mut evidence, ag.antigravityrules, ".antigravityrules");
        push_ev(&mut evidence, ag.antigravity_dir, ".antigravity");
        push_ev(&mut evidence, ag.agents_skills, ".agents/skills");
        push_ev(&mut evidence, ag.agents_workflows, ".agents/workflows");
        evidence.sort();
        hits.push(HarnessHit {
            id: "antigravity".to_string(),
            present,
            evidence,
        });
    }

    {
        let present = claude.claude_md || claude.claude_dir;
        let mut evidence = Vec::new();
        push_ev(&mut evidence, claude.claude_md, "CLAUDE.md");
        push_ev(&mut evidence, claude.claude_dir, ".claude");
        evidence.sort();
        hits.push(HarnessHit {
            id: "claude".to_string(),
            present,
            evidence,
        });
    }

    {
        let present = codex.agents_md || codex.codex_dir || codex.agents_skills;
        let mut evidence = Vec::new();
        push_ev(&mut evidence, codex.agents_md, "AGENTS.md");
        push_ev(&mut evidence, codex.codex_dir, ".codex");
        push_ev(&mut evidence, codex.agents_skills, ".agents/skills");
        evidence.sort();
        hits.push(HarnessHit {
            id: "codex".to_string(),
            present,
            evidence,
        });
    }

    {
        let present = cursor.cursor_dir || cursor.cursorrules;
        let mut evidence = Vec::new();
        push_ev(&mut evidence, cursor.cursor_dir, ".cursor");
        push_ev(&mut evidence, cursor.cursorrules, ".cursorrules");
        evidence.sort();
        hits.push(HarnessHit {
            id: "cursor".to_string(),
            present,
            evidence,
        });
    }

    hits.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(hits)
}

/// Four absent harnesses (no project root).
pub fn empty_harnesses() -> Vec<HarnessHit> {
    ["antigravity", "claude", "codex", "cursor"]
        .iter()
        .map(|id| HarnessHit {
            id: (*id).to_string(),
            present: false,
            evidence: Vec::new(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detect_harnesses_sorted_four() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let hits = detect_harnesses(&root).unwrap();
        let ids: Vec<_> = hits.iter().map(|h| h.id.as_str()).collect();
        assert_eq!(ids, vec!["antigravity", "claude", "codex", "cursor"]);
    }
}
