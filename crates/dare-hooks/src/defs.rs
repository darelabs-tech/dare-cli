//! Hook definitions: embedded defaults + optional `.dare/hooks.yml` overlay.

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::Deserialize;

use crate::action::HookAction;
use crate::event::HookEvent;

/// Embedded SoT (`assets/hooks/default-hooks.yml`).
pub const DEFAULT_HOOKS_YML: &str = include_str!("../../../assets/hooks/default-hooks.yml");

/// Relative path of the project overlay (posix).
pub const HOOKS_OVERLAY_REL: &str = ".dare/hooks.yml";

/// Max bytes read for overlay files.
pub const HOOKS_READ_CAP: u64 = 1_048_576;

/// Required `schemaVersion` in hooks.yml.
pub const HOOKS_SCHEMA_VERSION: u32 = 1;

/// One hook rule: event + ordered actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookDef {
    pub event: HookEvent,
    pub actions: Vec<HookAction>,
}

/// Parsed hooks file (`schemaVersion` + hooks list).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HooksFile {
    pub schema_version: u32,
    pub hooks: Vec<HookDef>,
}

#[derive(Debug, Deserialize)]
struct RawHooksFile {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    hooks: Vec<RawHookDef>,
}

#[derive(Debug, Deserialize)]
struct RawHookDef {
    event: String,
    actions: Vec<String>,
}

/// Parse and validate hooks YAML from a string.
pub fn load_hooks_from_str(s: &str) -> CoreResult<HooksFile> {
    let raw: RawHooksFile = serde_yaml::from_str(s)
        .map_err(|e| CoreError::invalid_input(format!("invalid hooks.yml: {e}")))?;
    if raw.schema_version != HOOKS_SCHEMA_VERSION {
        return Err(CoreError::invalid_input(format!(
            "hooks schemaVersion must be {HOOKS_SCHEMA_VERSION}, got {}",
            raw.schema_version
        )));
    }
    let mut hooks = Vec::with_capacity(raw.hooks.len());
    for h in raw.hooks {
        let event = HookEvent::parse(&h.event)?;
        let mut actions = Vec::with_capacity(h.actions.len());
        for a in h.actions {
            actions.push(HookAction::parse(&a)?);
        }
        hooks.push(HookDef { event, actions });
    }
    Ok(HooksFile {
        schema_version: raw.schema_version,
        hooks,
    })
}

/// Load hooks defs from overlay (if present) or embedded defaults.
///
/// Returns `(file, source)` where `source` is `"overlay"` or `"embed"`.
/// Overlay **replaces** the entire hooks list (no merge with embed).
pub fn load_hooks_defs(root: &ProjectRoot) -> CoreResult<(HooksFile, &'static str)> {
    let rel = SafeRelativePath::new(HOOKS_OVERLAY_REL)?;
    let abs = root.resolve(&rel)?;
    let path = abs.as_path().as_std_path();
    if path.is_file() {
        let meta = std::fs::metadata(path).map_err(|e| CoreError::io(e.to_string()))?;
        if meta.len() > HOOKS_READ_CAP {
            return Err(CoreError::invalid_input(format!(
                "hooks overlay exceeds {HOOKS_READ_CAP} bytes"
            )));
        }
        let raw = std::fs::read_to_string(path).map_err(|e| CoreError::io(e.to_string()))?;
        let file = load_hooks_from_str(&raw)?;
        return Ok((file, "overlay"));
    }
    let file = load_hooks_from_str(DEFAULT_HOOKS_YML)?;
    Ok((file, "embed"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ErrorKind;
    use tempfile::tempdir;

    #[test]
    fn load_embed() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let (file, source) = load_hooks_defs(&root).expect("embed");
        assert_eq!(source, "embed");
        assert_eq!(file.schema_version, 1);
        assert_eq!(file.hooks.len(), 4);
        assert_eq!(file.hooks[0].event, HookEvent::OnSave);
        assert_eq!(file.hooks[0].actions, vec![HookAction::DareValidate]);
        assert_eq!(file.hooks[1].event, HookEvent::OnFileCreate);
        assert_eq!(file.hooks[1].actions, vec![HookAction::DareValidate]);
        assert_eq!(file.hooks[2].event, HookEvent::OnTaskComplete);
        assert_eq!(
            file.hooks[2].actions,
            vec![HookAction::DareReview, HookAction::GraphRegister]
        );
        assert_eq!(file.hooks[3].event, HookEvent::PreCommit);
        assert_eq!(
            file.hooks[3].actions,
            vec![HookAction::DareValidate, HookAction::Lint]
        );
    }

    #[test]
    fn overlay_replaces() {
        let dir = tempdir().unwrap();
        let dare = dir.path().join(".dare");
        std::fs::create_dir_all(&dare).unwrap();
        std::fs::write(
            dare.join("hooks.yml"),
            r#"
schemaVersion: 1
hooks:
  - event: pre-commit
    actions: [test]
"#,
        )
        .unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let (file, source) = load_hooks_defs(&root).expect("overlay");
        assert_eq!(source, "overlay");
        assert_eq!(file.hooks.len(), 1);
        assert_eq!(file.hooks[0].event, HookEvent::PreCommit);
        assert_eq!(file.hooks[0].actions, vec![HookAction::Test]);
    }

    #[test]
    fn reject_unknown_action_in_overlay() {
        let dir = tempdir().unwrap();
        let dare = dir.path().join(".dare");
        std::fs::create_dir_all(&dare).unwrap();
        std::fs::write(
            dare.join("hooks.yml"),
            r#"
schemaVersion: 1
hooks:
  - event: on-save
    actions: [rm-rf]
"#,
        )
        .unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let err = load_hooks_defs(&root).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(err.message().contains("unknown hook action: rm-rf"));
    }
}
