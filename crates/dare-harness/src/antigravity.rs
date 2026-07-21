//! Antigravity adapter (microplano 014): .antigravityrules + commands + shared skills.

use dare_assets::{
    load_capability_matrix_from_str, render_agent_skill, render_claude_command,
    validate_capability_matrix, EmbeddedAssets,
};
use dare_core::fs::{atomic_write, read_to_string};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};

const MANAGED_PREFIX: &str = "<!-- dare:managed";
const RULES_REL: &str = ".antigravityrules";
const WORKFLOWS_KEEP: &str = ".agents/workflows/.gitkeep";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AntigravityDetect {
    pub antigravityrules: bool,
    pub antigravity_dir: bool,
    pub agents_skills: bool,
    pub agents_workflows: bool,
}

fn exists(root: &ProjectRoot, rel: &str) -> CoreResult<bool> {
    let path = SafeRelativePath::new(rel)?;
    Ok(root.resolve(&path)?.as_path().exists())
}

pub fn detect_antigravity(root: &ProjectRoot) -> CoreResult<AntigravityDetect> {
    Ok(AntigravityDetect {
        antigravityrules: exists(root, RULES_REL)?,
        antigravity_dir: exists(root, ".antigravity")?,
        agents_skills: exists(root, ".agents/skills")?,
        agents_workflows: exists(root, ".agents/workflows")?,
    })
}

fn is_managed(bytes: &[u8]) -> bool {
    String::from_utf8_lossy(bytes)
        .lines()
        .next()
        .map(|l| {
            let t = l.trim_start();
            t.starts_with(MANAGED_PREFIX) || t.starts_with("---")
        })
        .unwrap_or(false)
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
    Ok(is_managed(existing.as_bytes()))
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

pub fn generate_antigravityrules(root: &ProjectRoot, force: bool) -> CoreResult<()> {
    let rel = SafeRelativePath::new(RULES_REL)?;
    if !should_write(root, &rel, force)? {
        return Ok(());
    }
    let body = format!(
        "{MANAGED_PREFIX} antigravityrules -->\n# DARE Antigravity rules\n\n\
         Follow Design → Blueprint → Tasks → Execute. Use Agent Skills under `.agents/skills/`.\n\
         Shared with Codex — do not diverge skill bodies.\n"
    );
    atomic_write(root, &rel, body.as_bytes())
}

pub fn ensure_workflows_dir(root: &ProjectRoot, force: bool) -> CoreResult<()> {
    let rel = SafeRelativePath::new(WORKFLOWS_KEEP)?;
    if !should_write(root, &rel, force)? {
        return Ok(());
    }
    // Empty workflows dir marker (TS leaves workflows empty).
    atomic_write(root, &rel, b"")
}

fn skill_body(cap: &dare_assets::Capability) -> String {
    format!(
        "{MANAGED_PREFIX} capability={} -->\n{}",
        cap.id,
        render_agent_skill(cap)
    )
}

/// Install matrix `.antigravity/commands` + shared `.agents/skills` (Codex coexistence).
pub fn install_antigravity(root: &ProjectRoot, force: bool) -> CoreResult<usize> {
    let matrix = load_matrix()?;
    let mut written = 0usize;
    for cap in &matrix.capabilities {
        let Some(out) = cap.outputs.antigravity.as_deref() else {
            continue;
        };
        let cmd_rel = SafeRelativePath::new(out)?;
        if should_write(root, &cmd_rel, force)? {
            let rendered = render_claude_command(cap);
            let body = format!("{MANAGED_PREFIX} capability={} -->\n{rendered}", cap.id);
            atomic_write(root, &cmd_rel, body.as_bytes())?;
            written += 1;
        }
        let shared = format!(".agents/skills/{}/SKILL.md", cap.id);
        let shared_rel = SafeRelativePath::new(&shared)?;
        if should_write(root, &shared_rel, force)? {
            let body = skill_body(cap);
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

/// Validate frontmatter `name:` + `description:` on a skill body.
pub fn validate_skill_frontmatter(body: &str) -> CoreResult<()> {
    let mut name = false;
    let mut desc = false;
    let mut in_fm = false;
    for line in body.lines() {
        let t = line.trim();
        if t == "---" {
            if !in_fm {
                in_fm = true;
                continue;
            }
            break;
        }
        if !in_fm {
            continue;
        }
        if let Some(rest) = t.strip_prefix("name:") {
            name = !rest.trim().is_empty();
        }
        if let Some(rest) = t.strip_prefix("description:") {
            desc = !rest.trim().is_empty();
        }
    }
    if name && desc {
        Ok(())
    } else {
        Err(CoreError::config(
            "skill frontmatter missing name and/or description",
        ))
    }
}

pub fn validate_antigravity_install(root: &ProjectRoot) -> CoreResult<usize> {
    let matrix = load_matrix()?;
    let mut ok = 0usize;
    let mut missing = Vec::new();
    for cap in &matrix.capabilities {
        let Some(out) = cap.outputs.antigravity.as_deref() else {
            continue;
        };
        let rel = SafeRelativePath::new(out)?;
        if root.resolve(&rel)?.as_path().is_file() {
            ok += 1;
        } else {
            missing.push(out.to_string());
        }
        let shared = format!(".agents/skills/{}/SKILL.md", cap.id);
        let srel = SafeRelativePath::new(&shared)?;
        if root.resolve(&srel)?.as_path().is_file() {
            let body = read_to_string(root, &srel)?;
            validate_skill_frontmatter(&body)?;
        } else {
            missing.push(shared);
        }
    }
    let rules = SafeRelativePath::new(RULES_REL)?;
    if !root.resolve(&rules)?.as_path().is_file() {
        return Err(CoreError::config(".antigravityrules missing"));
    }
    if !missing.is_empty() {
        return Err(CoreError::config(format!(
            "antigravity assets missing ({}): {}",
            missing.len(),
            missing.into_iter().take(5).collect::<Vec<_>>().join(", ")
        )));
    }
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codex::{generate_agents_md, install_codex_skills};
    use tempfile::tempdir;

    #[test]
    fn detect_empty_and_generate_rules_workflows() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let d = detect_antigravity(&root).unwrap();
        assert!(!d.antigravityrules);
        assert!(!d.antigravity_dir);
        assert!(!d.agents_skills);
        assert!(!d.agents_workflows);
        generate_antigravityrules(&root, false).unwrap();
        ensure_workflows_dir(&root, false).unwrap();
        let d2 = detect_antigravity(&root).unwrap();
        assert!(d2.antigravityrules);
        assert!(d2.agents_workflows);
        let rules = read_to_string(&root, &SafeRelativePath::new(RULES_REL).unwrap()).unwrap();
        assert!(rules.starts_with(MANAGED_PREFIX));
        assert!(root
            .resolve(&SafeRelativePath::new(WORKFLOWS_KEEP).unwrap())
            .unwrap()
            .as_path()
            .is_file());
    }

    #[test]
    fn preserve_unmanaged_antigravityrules() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new(RULES_REL).unwrap();
        atomic_write(&root, &rel, b"# custom antigravity rules\nkeep\n").unwrap();
        generate_antigravityrules(&root, false).unwrap();
        let content = read_to_string(&root, &rel).unwrap();
        assert!(content.contains("custom antigravity rules"));
        assert!(!content.starts_with(MANAGED_PREFIX));
    }

    #[test]
    fn install_validate_and_codex_coexistence() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        generate_antigravityrules(&root, true).unwrap();
        ensure_workflows_dir(&root, true).unwrap();
        assert_eq!(install_antigravity(&root, true).unwrap(), 49);
        assert_eq!(validate_antigravity_install(&root).unwrap(), 49);
        // Codex after Antigravity reuses .agents/skills (preserve managed identical).
        generate_agents_md(&root, true).unwrap();
        let _ = install_codex_skills(&root, false).unwrap();
        assert_eq!(validate_antigravity_install(&root).unwrap(), 49);
    }

    #[test]
    fn frontmatter_rejects_incomplete() {
        assert!(validate_skill_frontmatter("---\nname: x\n---\n").is_err());
        assert!(validate_skill_frontmatter("---\nname: x\ndescription: y\n---\nbody\n").is_ok());
    }
}
