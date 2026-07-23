//! Domain types for skills-pacote (microplano 044).

use dare_core::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

/// Canonical generic package skills (six).
pub const GENERIC_SKILL_IDS: [&str; 6] = [
    "dare-ax",
    "dare-frontend-design",
    "dare-layered-design",
    "dare-llm-integration",
    "dare-quality-telemetry",
    "dare-realtime",
];

/// Generic (built-in package) vs stack-specific skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillKind {
    Generic,
    Stack,
}

/// Which registry contributed the skill entry after merge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillSource {
    Mock,
    Local,
    Remote,
}

impl SkillSource {
    pub fn as_str(self) -> &'static str {
        match self {
            SkillSource::Mock => "mock",
            SkillSource::Local => "local",
            SkillSource::Remote => "remote",
        }
    }

    /// Higher wins on merge (remote > local > mock).
    pub fn priority(self) -> u8 {
        match self {
            SkillSource::Remote => 3,
            SkillSource::Local => 2,
            SkillSource::Mock => 1,
        }
    }
}

/// Entry exposed by registries (list/info).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrySkill {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dare_version: Option<String>,
    #[serde(default, alias = "dependsOn")]
    pub depends_on: Vec<String>,
    pub kind: SkillKind,
    pub source: SkillSource,
}

/// Package-level `skill.yml` schema (distinct from project `.dare/skills.yml`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dare_version: Option<String>,
    #[serde(default, alias = "dependsOn")]
    pub depends_on: Vec<String>,
}

impl SkillManifest {
    pub fn into_registry_skill(self, source: SkillSource) -> RegistrySkill {
        let kind = classify_skill(&self.name);
        RegistrySkill {
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            license: self.license,
            dare_version: self.dare_version,
            depends_on: self.depends_on,
            kind,
            source,
        }
    }
}

/// Classify by canonical generic list; `skill-*` and others are stack.
pub fn classify_skill(name: &str) -> SkillKind {
    if GENERIC_SKILL_IDS.contains(&name) {
        SkillKind::Generic
    } else {
        SkillKind::Stack
    }
}

/// Path-safe skill id: relative segment, no `..`, no separators, ASCII-ish.
pub fn validate_skill_id(name: &str) -> CoreResult<()> {
    if name.is_empty() || name.contains('\0') {
        return Err(CoreError::invalid_input("invalid skill name"));
    }
    if name == "." || name == ".." || name.contains('/') || name.contains('\\') {
        return Err(CoreError::invalid_input("invalid skill name"));
    }
    if name.starts_with('.') {
        return Err(CoreError::invalid_input("invalid skill name"));
    }
    // Reject Windows drive-like and absolute forms early.
    let bytes = name.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return Err(CoreError::invalid_input("invalid skill name"));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err(CoreError::invalid_input("invalid skill name"));
    }
    Ok(())
}

pub fn validate_version_segment(version: &str) -> CoreResult<()> {
    validate_skill_id(version).map_err(|_| CoreError::invalid_input("invalid skill version"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_six_generics() {
        for id in GENERIC_SKILL_IDS {
            assert_eq!(classify_skill(id), SkillKind::Generic);
        }
    }

    #[test]
    fn classify_stack() {
        assert_eq!(classify_skill("skill-nestjs-api"), SkillKind::Stack);
        assert_eq!(classify_skill("skill-laravel-api"), SkillKind::Stack);
        assert_eq!(classify_skill("custom-thing"), SkillKind::Stack);
    }

    #[test]
    fn validate_id_rejects_traversal() {
        assert!(validate_skill_id("../evil").is_err());
        assert!(validate_skill_id("a/b").is_err());
        assert!(validate_skill_id("").is_err());
        assert!(validate_skill_id("dare-ax").is_ok());
    }

    #[test]
    fn source_priority_order() {
        assert!(SkillSource::Remote.priority() > SkillSource::Local.priority());
        assert!(SkillSource::Local.priority() > SkillSource::Mock.priority());
    }
}
