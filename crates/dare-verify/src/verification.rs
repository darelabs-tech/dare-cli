//! Persist Ralph results under `.dare/verification/<taskId>.json`.

use dare_core::fs::atomic_write;
use dare_core::{
    redact, to_canonical_json_string, CoreError, CoreResult, ProjectRoot, SafeRelativePath,
};
use serde::{Deserialize, Serialize};

use crate::ralph::{GateStep, RalphReport};

/// Relative directory for verification artifacts.
pub const VERIFICATION_DIR_REL: &str = ".dare/verification";

/// On-disk Ralph verification report (schema version 1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VerificationReport {
    pub schema_version: u32,
    pub task_id: String,
    pub ok: bool,
    pub timed_out: bool,
    pub stack: String,
    pub aspects: Vec<GateStep>,
    pub updated_at: String,
}

/// Task ids safe as a single path segment under [`VERIFICATION_DIR_REL`].
pub fn task_id_is_path_safe(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

/// Build a [`VerificationReport`] from a Ralph run (redact applied at write time).
pub fn verification_from_ralph(
    task_id: &str,
    ralph: &RalphReport,
    updated_at: &str,
) -> VerificationReport {
    VerificationReport {
        schema_version: 1,
        task_id: task_id.to_string(),
        ok: ralph.ok,
        timed_out: ralph.timed_out,
        stack: ralph.stack.clone(),
        aspects: ralph.steps.clone(),
        updated_at: updated_at.to_string(),
    }
}

/// Atomically write a redacted verification report under the project jail.
pub fn write_verification(root: &ProjectRoot, report: &VerificationReport) -> CoreResult<()> {
    if !task_id_is_path_safe(&report.task_id) {
        return Err(CoreError::invalid_input(format!(
            "unsafe verification task id: {}",
            report.task_id
        )));
    }
    let mut redacted = report.clone();
    for step in &mut redacted.aspects {
        step.stdout_tail = redact(&step.stdout_tail);
        step.stderr_tail = redact(&step.stderr_tail);
    }
    let rel_str = format!("{VERIFICATION_DIR_REL}/{}.json", redacted.task_id);
    let rel = SafeRelativePath::new(&rel_str)?;
    let value = serde_json::to_value(&redacted).map_err(|e| CoreError::internal(e.to_string()))?;
    let body = to_canonical_json_string(&value)?;
    atomic_write(root, &rel, body.as_bytes())
}

/// Read a verification report (tests / tooling).
pub fn load_verification(root: &ProjectRoot, task_id: &str) -> CoreResult<VerificationReport> {
    if !task_id_is_path_safe(task_id) {
        return Err(CoreError::invalid_input(format!(
            "unsafe verification task id: {task_id}"
        )));
    }
    let rel_str = format!("{VERIFICATION_DIR_REL}/{task_id}.json");
    let rel = SafeRelativePath::new(&rel_str)?;
    let abs = root.resolve(&rel)?;
    let bytes =
        std::fs::read(abs.as_path().as_std_path()).map_err(|e| CoreError::io(e.to_string()))?;
    let report: VerificationReport =
        serde_json::from_slice(&bytes).map_err(|e| CoreError::config(e.to_string()))?;
    if report.schema_version != 1 {
        return Err(CoreError::config("unsupported verification schema version"));
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ralph::GateAspect;
    use dare_core::ProjectRoot;

    fn sample_report(task_id: &str) -> VerificationReport {
        VerificationReport {
            schema_version: 1,
            task_id: task_id.to_string(),
            ok: true,
            timed_out: false,
            stack: "rust-axum".into(),
            aspects: vec![GateStep {
                aspect: GateAspect::Build,
                program: "cargo".into(),
                args: vec!["build".into(), "--workspace".into()],
                exit_code: 0,
                timed_out: false,
                stdout_tail: "ok TOKEN=secret".into(),
                stderr_tail: String::new(),
                duration_ms: 1,
            }],
            updated_at: "2026-07-22T12:00:00Z".into(),
        }
    }

    #[test]
    fn task_id_path_safe_accepts_kebab() {
        assert!(task_id_is_path_safe("task-001"));
        assert!(task_id_is_path_safe("mp029-003"));
        assert!(!task_id_is_path_safe(""));
        assert!(!task_id_is_path_safe("a/b"));
        assert!(!task_id_is_path_safe("../x"));
        assert!(!task_id_is_path_safe("-bad"));
    }

    #[test]
    fn write_verification_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let report = sample_report("task-001");
        write_verification(&root, &report).expect("write");
        let loaded = load_verification(&root, "task-001").expect("load");
        assert_eq!(loaded.schema_version, 1);
        assert_eq!(loaded.task_id, "task-001");
        assert!(loaded.ok);
        assert_eq!(loaded.aspects.len(), 1);
        // redact may alter TOKEN=… — still non-empty path created
        assert!(dir
            .path()
            .join(".dare/verification/task-001.json")
            .is_file());
    }

    #[test]
    fn write_verification_unsafe_id() {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let report = sample_report("a/b");
        let err = write_verification(&root, &report).expect_err("unsafe");
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    #[test]
    fn verification_from_ralph_maps_fields() {
        let ralph = RalphReport {
            ok: false,
            timed_out: true,
            stack: "rust".into(),
            steps: vec![],
            total_duration_ms: 9,
        };
        let v = verification_from_ralph("t1", &ralph, "2026-07-22T00:00:00Z");
        assert_eq!(v.schema_version, 1);
        assert!(!v.ok);
        assert!(v.timed_out);
        assert_eq!(v.stack, "rust");
    }
}
