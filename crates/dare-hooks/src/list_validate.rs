//! `list_hooks` / `validate_hooks` domain operations.

use dare_contracts::DareConfig;
use dare_core::{CoreResult, ProjectRoot};

use crate::config::{hooks_enabled, hooks_trusted};
use crate::defs::load_hooks_defs;
use crate::report::{
    HookListItem, HooksListReport, HooksValidateReport, HOOKS_LIST_SCHEMA, HOOKS_VALIDATE_SCHEMA,
};

/// Load hooks defs and return a sorted list report.
pub fn list_hooks(root: &ProjectRoot, cfg: &DareConfig) -> CoreResult<HooksListReport> {
    let (file, source) = load_hooks_defs(root)?;
    let mut hooks: Vec<HookListItem> = file
        .hooks
        .into_iter()
        .map(|h| HookListItem {
            event: h.event.as_str().to_string(),
            actions: h.actions.iter().map(|a| a.as_str().to_string()).collect(),
        })
        .collect();
    hooks.sort_by(|a, b| a.event.cmp(&b.event));
    Ok(HooksListReport {
        schema_version: HOOKS_LIST_SCHEMA,
        project_root: root.to_posix(),
        trusted: hooks_trusted(cfg),
        enabled: hooks_enabled(cfg),
        source: source.to_string(),
        hooks,
    })
}

/// Validate hooks defs without writing. Load errors become report errors (`ok=false`).
pub fn validate_hooks(root: &ProjectRoot, _cfg: &DareConfig) -> CoreResult<HooksValidateReport> {
    match load_hooks_defs(root) {
        Ok((file, source)) => {
            let _ = file;
            Ok(HooksValidateReport {
                schema_version: HOOKS_VALIDATE_SCHEMA,
                ok: true,
                source: source.to_string(),
                errors: Vec::new(),
                warnings: Vec::new(),
            })
        }
        Err(e) => Ok(HooksValidateReport {
            schema_version: HOOKS_VALIDATE_SCHEMA,
            ok: false,
            source: String::new(),
            errors: vec![e.message().to_string()],
            warnings: Vec::new(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn validate_bad_action() {
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
        let cfg = DareConfig::default();
        let report = validate_hooks(&root, &cfg).expect("report");
        assert!(!report.ok);
        assert!(!report.errors.is_empty());
        assert!(report.errors[0].contains("unknown hook action: rm-rf"));
        assert!(report.source.is_empty());
        // zero writes: only the overlay we created should exist under .dare
        let entries: Vec<_> = std::fs::read_dir(&dare).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }
}
