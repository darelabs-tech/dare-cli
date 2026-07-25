//! Post-scaffold validation (BLUEPRINT-046 §0.5 Validate).

use dare_core::fs::read_to_string;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};

use crate::ax::ax_artifact_paths;
use crate::registry::scaffolder_for;
use crate::render::scan_secrets;
use crate::types::ValidationReport;

fn expected_paths(stack_id: &str) -> CoreResult<Vec<String>> {
    let scaffolder = scaffolder_for(stack_id)?;
    let meta = scaffolder.metadata();
    let mut paths = ax_artifact_paths(meta);
    if !paths.iter().any(|p| p == "dare.config.json") {
        paths.push("dare.config.json".to_string());
    }
    Ok(paths)
}

/// Validate AX artifacts + `dare.config.json` exist and pass secret scan.
pub fn validate_stack_output(root: &ProjectRoot, stack_id: &str) -> CoreResult<ValidationReport> {
    let paths = expected_paths(stack_id)?;

    let mut missing = Vec::new();
    let mut secret_hits = Vec::new();

    for path in paths {
        let rel = match SafeRelativePath::new(&path) {
            Ok(r) => r,
            Err(e) => {
                return Err(CoreError::Internal(format!(
                    "invalid expected path `{path}` for stack `{stack_id}`: {e}"
                )));
            }
        };
        let abs = match root.resolve(&rel) {
            Ok(a) => a,
            Err(_) => {
                missing.push(path);
                continue;
            }
        };
        if !abs.as_path().as_std_path().exists() {
            missing.push(path);
            continue;
        }
        match read_to_string(root, &rel) {
            Ok(content) => {
                if scan_secrets(&content).is_err() {
                    secret_hits.push(path);
                }
            }
            Err(_) => {
                missing.push(path);
            }
        }
    }

    missing.sort();
    secret_hits.sort();

    Ok(ValidationReport {
        stack_id: stack_id.to_string(),
        ok: missing.is_empty() && secret_hits.is_empty(),
        missing,
        secret_hits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ax::ax_artifact_paths;
    use crate::registry::{scaffolder_for, MSG_UNKNOWN_STACK};
    use dare_core::CoreError;
    use tempfile::tempdir;

    #[test]
    fn validate_missing_lists() {
        let dir = tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("project root");
        let stack_id = "rust-axum";

        let report = validate_stack_output(&root, stack_id).expect("validate");

        let meta = scaffolder_for(stack_id).expect("stack").metadata();
        let mut expected = ax_artifact_paths(meta);
        expected.push("dare.config.json".to_string());
        expected.sort();

        assert_eq!(report.stack_id, stack_id);
        assert!(!report.ok);
        assert!(report.secret_hits.is_empty());
        assert_eq!(report.missing, expected);
    }

    #[test]
    fn validate_unknown_stack() {
        let dir = tempdir().expect("tempdir");
        let root = ProjectRoot::new(dir.path()).expect("project root");

        let err = validate_stack_output(&root, "bogus-stack").expect_err("unknown stack");
        match err {
            CoreError::InvalidInput(msg) => {
                assert!(msg.contains(MSG_UNKNOWN_STACK));
            }
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }
}
