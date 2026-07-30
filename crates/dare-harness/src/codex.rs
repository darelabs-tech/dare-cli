//! Codex adapter (microplano 013): AGENTS.md + skills (matrix + .agents/skills).

use dare_assets::{
    load_capability_matrix_from_str, render_agent_skill, validate_capability_matrix, EmbeddedAssets,
};
use dare_core::fs::{atomic_write, read_to_string};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};

const MANAGED_PREFIX: &str = "<!-- dare:managed";
const AGENTS_MD: &str = "AGENTS.md";

/// Harness IDs that update policies must cover (Codex included — DEC-014).
pub const UPDATE_HARNESS_IDES: &[&str] = &[
    "claude-code",
    "cursor",
    "codex",
    "antigravity",
    "hybrid",
    "claude-hybrid",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexDetect {
    pub agents_md: bool,
    pub codex_dir: bool,
    pub agents_skills: bool,
}

fn exists(root: &ProjectRoot, rel: &str) -> CoreResult<bool> {
    let path = SafeRelativePath::new(rel)?;
    Ok(root.resolve(&path)?.as_path().exists())
}

pub fn detect_codex(root: &ProjectRoot) -> CoreResult<CodexDetect> {
    Ok(CodexDetect {
        agents_md: exists(root, AGENTS_MD)?,
        codex_dir: exists(root, ".codex")?,
        agents_skills: exists(root, ".agents/skills")?,
    })
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
    Ok(crate::content_is_managed(existing.as_bytes()))
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

fn skill_body(cap: &dare_assets::Capability) -> String {
    format!(
        "{MANAGED_PREFIX} capability={} -->\n{}",
        cap.id,
        render_agent_skill(cap)
    )
}

/// Dynamic AGENTS.md listing `$skill-name` invocations.
pub fn generate_agents_md(root: &ProjectRoot, force: bool) -> CoreResult<()> {
    let rel = SafeRelativePath::new(AGENTS_MD)?;
    if !should_write(root, &rel, force)? {
        return Ok(());
    }
    let matrix = load_matrix()?;
    let mut lines = vec![
        format!("{MANAGED_PREFIX} agents-md -->"),
        "# DARE Codex Agents".into(),
        String::new(),
        "Invoke Agent Skills with `$skill-name` (example: `$dare-design`).".into(),
        String::new(),
        "## Skills".into(),
    ];
    for cap in &matrix.capabilities {
        if cap.outputs.codex.is_none() {
            continue;
        }
        lines.push(format!("- `${}` — {}", cap.id, cap.description.trim()));
    }
    lines.push(String::new());
    lines.push(
        "Shared skills live under `.agents/skills/` (Antigravity coexistence) and `.codex/skills/`."
            .into(),
    );
    lines.push(String::new());
    atomic_write(root, &rel, lines.join("\n").as_bytes())
}

/// Install Codex skills from matrix; also materialize `.agents/skills/<id>/SKILL.md`
/// when missing or managed (reuse — no divergent duplicate for Antigravity).
pub fn install_codex_skills(root: &ProjectRoot, force: bool) -> CoreResult<usize> {
    let matrix = load_matrix()?;
    let mut written = 0usize;
    for cap in &matrix.capabilities {
        let Some(out) = cap.outputs.codex.as_deref() else {
            continue;
        };
        let body = skill_body(cap);
        let matrix_rel = SafeRelativePath::new(out)?;
        if should_write(root, &matrix_rel, force)? {
            atomic_write(root, &matrix_rel, body.as_bytes())?;
            written += 1;
        }
        // Shared Agent Skills path (microplano + Antigravity coexistence).
        let shared = format!(".agents/skills/{}/SKILL.md", cap.id);
        let shared_rel = SafeRelativePath::new(&shared)?;
        if should_write(root, &shared_rel, force)? {
            // Skip write if identical managed content already present (no-op reuse).
            let abs = root.resolve(&shared_rel)?;
            if abs.as_path().is_file() && !force {
                let existing = read_to_string(root, &shared_rel)?;
                if existing == body {
                    continue;
                }
            }
            atomic_write(root, &shared_rel, body.as_bytes())?;
        }
    }
    Ok(written)
}

pub fn validate_codex_install(root: &ProjectRoot) -> CoreResult<usize> {
    let matrix = load_matrix()?;
    let mut ok = 0usize;
    let mut missing = Vec::new();
    for cap in &matrix.capabilities {
        let Some(out) = cap.outputs.codex.as_deref() else {
            continue;
        };
        let rel = SafeRelativePath::new(out)?;
        if root.resolve(&rel)?.as_path().is_file() {
            ok += 1;
        } else {
            missing.push(out.to_string());
        }
    }
    let agents = SafeRelativePath::new(AGENTS_MD)?;
    if !root.resolve(&agents)?.as_path().is_file() {
        return Err(CoreError::config("AGENTS.md missing"));
    }
    if !missing.is_empty() {
        return Err(CoreError::config(format!(
            "codex skills missing ({}): {}",
            missing.len(),
            missing.into_iter().take(5).collect::<Vec<_>>().join(", ")
        )));
    }
    Ok(ok)
}

pub fn update_policies_include_codex() -> bool {
    UPDATE_HARNESS_IDES.contains(&"codex")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn detect_empty_and_generate_agents_md() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let d = detect_codex(&root).unwrap();
        assert!(!d.agents_md);
        assert!(!d.codex_dir);
        assert!(!d.agents_skills);
        assert!(update_policies_include_codex());
        generate_agents_md(&root, false).unwrap();
        let d2 = detect_codex(&root).unwrap();
        assert!(d2.agents_md);
        let agents = read_to_string(&root, &SafeRelativePath::new(AGENTS_MD).unwrap()).unwrap();
        assert!(agents.starts_with(MANAGED_PREFIX));
        assert!(agents.contains("$dare-design"));
        assert!(agents.contains("$skill-name"));
    }

    #[test]
    fn preserve_unmanaged_agents_md() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new(AGENTS_MD).unwrap();
        atomic_write(&root, &rel, b"# custom agents\nkeep\n").unwrap();
        generate_agents_md(&root, false).unwrap();
        let content = read_to_string(&root, &rel).unwrap();
        assert!(content.contains("custom agents"));
        assert!(!content.starts_with(MANAGED_PREFIX));
    }

    #[test]
    fn install_validate_agents_and_codex_in_policies() {
        assert!(update_policies_include_codex());
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        generate_agents_md(&root, true).unwrap();
        assert_eq!(install_codex_skills(&root, true).unwrap(), 50);
        assert_eq!(validate_codex_install(&root).unwrap(), 50);
        let agents = read_to_string(&root, &SafeRelativePath::new(AGENTS_MD).unwrap()).unwrap();
        assert!(agents.contains("$dare-design"));
        let shared = SafeRelativePath::new(".agents/skills/dare-design/SKILL.md").unwrap();
        assert!(root.resolve(&shared).unwrap().as_path().is_file());
    }

    #[test]
    fn coexistence_reuses_agents_skills_without_overwrite_unmanaged() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let shared = SafeRelativePath::new(".agents/skills/dare-design/SKILL.md").unwrap();
        atomic_write(&root, &shared, b"user custom skill\n").unwrap();
        generate_agents_md(&root, true).unwrap();
        let _ = install_codex_skills(&root, false).unwrap();
        let kept = read_to_string(&root, &shared).unwrap();
        assert_eq!(kept, "user custom skill\n");
    }

    #[test]
    fn validate_requires_agents_md() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let _ = install_codex_skills(&root, true).unwrap();
        let msg = validate_codex_install(&root).unwrap_err().to_string();
        assert!(msg.contains("AGENTS.md missing"));
    }
}
