//! Orchestrator: `detect`, human + JSON formatting.

use std::path::Path;

use dare_core::{CoreError, CoreResult, ProjectRoot};
use serde_json::Value;

use crate::git::find_git_root;
use crate::harnesses::{detect_harnesses, empty_harnesses};
use crate::monorepo::detect_monorepo;
use crate::report::{DetectionReport, DETECTION_SCHEMA_VERSION};
use crate::root::find_project_root;
use crate::stacks::detect_stacks;

fn display_path(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

/// Read-only brownfield detection. `mode` is always `"check"`.
pub fn detect(start: &Path) -> CoreResult<DetectionReport> {
    if !start.exists() || !start.is_dir() {
        return Err(CoreError::not_found(format!(
            "directory not found: {}",
            start.display()
        )));
    }

    let project_root = find_project_root(start);
    let git_root = find_git_root(start, project_root.as_deref());
    let git_disp = git_root.as_ref().map(|p| display_path(p));

    let Some(pr) = project_root else {
        return Ok(DetectionReport::empty_check(git_disp, empty_harnesses()));
    };

    let dare_already_present = pr.join("dare.config.json").is_file() || pr.join("DARE").is_dir();
    let (stacks, conflicts) = detect_stacks(&pr);
    let (monorepo, monorepo_evidence) = detect_monorepo(&pr);
    let root = ProjectRoot::new(&pr)?;
    let harnesses = detect_harnesses(&root)?;

    Ok(DetectionReport {
        schema_version: DETECTION_SCHEMA_VERSION,
        mode: "check".to_string(),
        project_root: Some(display_path(&pr)),
        git_root: git_disp,
        stacks,
        conflicts,
        monorepo,
        monorepo_evidence,
        harnesses,
        dare_already_present,
    })
}

/// Human-readable report (en-US). Final line MUST be exact.
pub fn format_human(r: &DetectionReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("schemaVersion: {}\n", r.schema_version));
    out.push_str(&format!("mode: {}\n", r.mode));
    out.push_str(&format!(
        "projectRoot: {}\n",
        r.project_root.as_deref().unwrap_or("(none)")
    ));
    out.push_str(&format!(
        "gitRoot: {}\n",
        r.git_root.as_deref().unwrap_or("(none)")
    ));
    out.push_str(&format!("dareAlreadyPresent: {}\n", r.dare_already_present));

    out.push_str("stacks:\n");
    if r.stacks.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for s in &r.stacks {
            out.push_str(&format!(
                "  - id={} family={} confidence={} evidence={:?}\n",
                s.id, s.family, s.confidence, s.evidence
            ));
        }
    }

    if r.conflicts.is_empty() {
        out.push_str("conflicts: none\n");
    } else {
        out.push_str("conflicts:\n");
        for c in &r.conflicts {
            out.push_str(&format!(
                "  - kinds={:?} evidence={:?}\n",
                c.kinds, c.evidence
            ));
        }
    }

    out.push_str(&format!("monorepo: {}\n", r.monorepo));
    if !r.monorepo_evidence.is_empty() {
        out.push_str(&format!("monorepoEvidence: {:?}\n", r.monorepo_evidence));
    }

    out.push_str("harnesses:\n");
    for h in &r.harnesses {
        out.push_str(&format!("  - id={} present={}\n", h.id, h.present));
    }

    out.push_str("mode: check (zero mutations)\n");
    out
}

/// JSON value for schema 1 (camelCase via serde).
pub fn report_to_json(r: &DetectionReport) -> Value {
    serde_json::to_value(r).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn list_names(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        v.sort();
        v
    }

    #[test]
    fn detect_is_read_only() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let before = list_names(dir.path());
        let _ = detect(dir.path()).unwrap();
        let after = list_names(dir.path());
        assert_eq!(before, after);
    }

    #[test]
    fn report_schema_via_detect() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let r = detect(dir.path()).unwrap();
        let v = report_to_json(&r);
        assert_eq!(v["schemaVersion"], 1);
        assert_eq!(v["mode"], "check");
        assert!(format_human(&r).contains("mode: check (zero mutations)"));
    }
}
