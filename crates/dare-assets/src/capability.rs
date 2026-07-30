//! Canonical capability model (ADR-007) — microplano 010.

use dare_core::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::manifest::assert_safe_asset_path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HarnessOutputs {
    #[serde(default)]
    pub claude: Option<String>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub codex: Option<String>,
    #[serde(default)]
    pub antigravity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    pub id: String,
    pub title: String,
    pub description: String,
    pub instructions: String,
    #[serde(default)]
    pub cli_commands: Vec<String>,
    pub outputs: HarnessOutputs,
    #[serde(default)]
    pub assets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityException {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityMatrix {
    pub version: u32,
    #[serde(default)]
    pub exceptions: Vec<CapabilityException>,
    pub capabilities: Vec<Capability>,
}

pub fn load_capability_matrix_from_str(yaml: &str) -> CoreResult<CapabilityMatrix> {
    let m: CapabilityMatrix = serde_yaml::from_str(yaml)
        .map_err(|e| CoreError::config(format!("invalid capability-matrix: {e}")))?;
    if m.version != 1 {
        return Err(CoreError::config(format!(
            "unsupported capability-matrix version: {}",
            m.version
        )));
    }
    Ok(m)
}

/// `^[a-z0-9]+(-[a-z0-9]+)*$` without pulling in the regex crate.
fn is_kebab_id(id: &str) -> bool {
    if id.is_empty() {
        return false;
    }
    let mut need_alnum = true;
    for c in id.chars() {
        if need_alnum {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() {
                return false;
            }
            need_alnum = false;
        } else if c == '-' {
            need_alnum = true;
        } else if !c.is_ascii_lowercase() && !c.is_ascii_digit() {
            return false;
        }
    }
    !need_alnum
}

/// Validate ids kebab-case, uniqueness, required fields, safe unique output paths.
pub fn validate_capability_matrix(m: &CapabilityMatrix) -> CoreResult<()> {
    for exc in &m.exceptions {
        if exc.id.is_empty() || exc.reason.is_empty() {
            return Err(CoreError::config(
                "capability exception missing id or reason",
            ));
        }
    }

    let mut ids = HashSet::new();
    let mut output_paths = HashSet::new();
    for cap in &m.capabilities {
        if !is_kebab_id(&cap.id) {
            return Err(CoreError::config(format!(
                "invalid capability id: {}",
                cap.id
            )));
        }
        if !ids.insert(cap.id.clone()) {
            return Err(CoreError::config(format!(
                "duplicate capability id: {}",
                cap.id
            )));
        }
        if cap.title.is_empty() || cap.description.is_empty() || cap.instructions.is_empty() {
            return Err(CoreError::config(format!(
                "capability {} missing required text fields",
                cap.id
            )));
        }
        for path in [
            cap.outputs.claude.as_deref(),
            cap.outputs.cursor.as_deref(),
            cap.outputs.codex.as_deref(),
            cap.outputs.antigravity.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            assert_safe_asset_path(path)?;
            if !output_paths.insert(path.to_string()) {
                return Err(CoreError::config(format!(
                    "duplicate harness output path: {path}"
                )));
            }
        }
        for path in &cap.assets {
            assert_safe_asset_path(path)?;
        }
    }
    Ok(())
}

/// Render a minimal Claude command markdown body for a capability (reproducible).
pub fn render_claude_command(cap: &Capability) -> String {
    format!(
        "# /{id}\n\n{title}\n\n{description}\n\n{instructions}\n",
        id = cap.id,
        title = cap.title,
        description = cap.description,
        instructions = cap.instructions.trim()
    )
}

/// Agent Skills body (Codex / Antigravity shared) with YAML frontmatter.
pub fn render_agent_skill(cap: &Capability) -> String {
    format!(
        "---\nname: {id}\ndescription: {desc}\n---\n\n# {title}\n\n{instructions}\n",
        id = cap.id,
        desc = cap.description.replace('\n', " ").trim(),
        title = cap.title,
        instructions = cap.instructions.trim()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::EmbeddedAssets;

    fn sample_cap(id: &str, claude: Option<&str>) -> Capability {
        Capability {
            id: id.into(),
            title: id.into(),
            description: "d".into(),
            instructions: "i".into(),
            cli_commands: vec![],
            outputs: HarnessOutputs {
                claude: claude.map(str::to_string),
                cursor: None,
                codex: None,
                antigravity: None,
            },
            assets: vec![],
        }
    }

    #[test]
    fn matrix_loads_and_validates() {
        let file = EmbeddedAssets::get("capability-matrix.yml").expect("embedded");
        let yaml = std::str::from_utf8(file.data.as_ref()).unwrap();
        let m = load_capability_matrix_from_str(yaml).unwrap();
        assert_eq!(m.capabilities.len(), 51);
        assert_eq!(m.exceptions.len(), 3);
        validate_capability_matrix(&m).unwrap();
    }

    #[test]
    fn rejects_underscore_id() {
        let m = CapabilityMatrix {
            version: 1,
            exceptions: vec![],
            capabilities: vec![sample_cap("dare_validate", Some(".claude/commands/x.md"))],
        };
        let err = validate_capability_matrix(&m).unwrap_err();
        assert!(err.to_string().contains("invalid capability id"));
    }

    #[test]
    fn rejects_dotdot_output_path() {
        let m = CapabilityMatrix {
            version: 1,
            exceptions: vec![],
            capabilities: vec![sample_cap(
                "dare-validate",
                Some(".claude/commands/../etc/passwd"),
            )],
        };
        let err = validate_capability_matrix(&m).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("dot-dot") || msg.contains("invalid asset path"),
            "{msg}"
        );
    }

    #[test]
    fn render_reproducible() {
        let cap = sample_cap("dare-validate", Some(".claude/commands/dare-validate.md"));
        let a = render_claude_command(&cap);
        let b = render_claude_command(&cap);
        assert_eq!(a, b);
        assert!(a.contains("/dare-validate"));
    }

    #[test]
    fn render_agent_skill_has_frontmatter() {
        let cap = sample_cap("dare-validate", None);
        let body = render_agent_skill(&cap);
        assert!(body.starts_with("---\nname: dare-validate\n"));
        assert!(body.contains("description: d\n---\n"));
        assert!(body.contains("# dare-validate\n"));
    }
}
