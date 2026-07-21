//! Canonical capability model (ADR-007) — microplano 010.

use dare_core::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

/// Validate ids kebab-case-ish, uniqueness, required fields, no duplicate output paths.
pub fn validate_capability_matrix(m: &CapabilityMatrix) -> CoreResult<()> {
    let mut ids = HashSet::new();
    let mut output_paths = HashSet::new();
    for cap in &m.capabilities {
        if cap.id.is_empty() || cap.id.contains(' ') || cap.id.contains('_') {
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
            if !output_paths.insert(path.to_string()) {
                return Err(CoreError::config(format!(
                    "duplicate harness output path: {path}"
                )));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::EmbeddedAssets;

    #[test]
    fn matrix_loads_and_validates() {
        let file = EmbeddedAssets::get("capability-matrix.yml").expect("embedded");
        let yaml = std::str::from_utf8(file.data.as_ref()).unwrap();
        let m = load_capability_matrix_from_str(yaml).unwrap();
        assert_eq!(m.capabilities.len(), 49);
        validate_capability_matrix(&m).unwrap();
    }

    #[test]
    fn render_reproducible() {
        let cap = Capability {
            id: "dare-validate".into(),
            title: "dare-validate".into(),
            description: "d".into(),
            instructions: "i".into(),
            cli_commands: vec![],
            outputs: HarnessOutputs {
                claude: Some(".claude/commands/dare-validate.md".into()),
                cursor: None,
                codex: None,
                antigravity: None,
            },
            assets: vec![],
        };
        let a = render_claude_command(&cap);
        let b = render_claude_command(&cap);
        assert_eq!(a, b);
        assert!(a.contains("/dare-validate"));
    }
}
