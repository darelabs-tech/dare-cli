//! Orchestrate static review under ProjectRoot.

use dare_core::fs::read_to_string;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};

use crate::agent::load_agent_semantic;
use crate::report::{new_report, sort_findings, ReviewReport};
use crate::scan::{is_scannable_path, scan_text};
use crate::spec::{execution_spec_rel, parse_spec_files, task_id_is_path_safe};
use crate::types::{FailOn, Finding, OutputFormat, Severity};
use crate::MAX_FILE_BYTES;

#[derive(Debug, Clone)]
pub struct ReviewOptions {
    pub task_id: String,
    pub files_override: Option<Vec<String>>,
    pub strict: bool,
    pub errors_only: bool,
    pub from_agent: Option<String>,
    pub format: OutputFormat,
    pub comment: bool,
    pub fail_on: FailOn,
    pub ai: bool,
}

pub fn run_review(root: &ProjectRoot, opts: &ReviewOptions) -> CoreResult<ReviewReport> {
    if !task_id_is_path_safe(&opts.task_id) {
        return Err(CoreError::invalid_input(
            "task id must match ^[A-Za-z0-9][A-Za-z0-9._-]*$",
        ));
    }

    let spec_rel = execution_spec_rel(&opts.task_id);
    let spec_path = SafeRelativePath::new(&spec_rel)?;
    let spec_md = match read_to_string(root, &spec_path) {
        Ok(s) => s,
        Err(CoreError::NotFound(_)) => {
            return Err(CoreError::not_found(format!("spec not found: {spec_rel}")));
        }
        Err(e) => return Err(e),
    };

    let file_list = if let Some(ref override_files) = opts.files_override {
        override_files.clone()
    } else {
        parse_spec_files(&spec_md)
    };

    let mut findings: Vec<Finding> = Vec::new();
    let mut files_scanned = 0u32;

    for rel in &file_list {
        let rel_norm = rel.replace('\\', "/");
        if rel_norm.contains("..") {
            return Err(CoreError::invalid_input(format!(
                "path escape denied: {rel_norm}"
            )));
        }
        let safe = SafeRelativePath::new(&rel_norm)?;
        if !is_scannable_path(safe.as_str()) {
            continue;
        }
        let abs = root.resolve(&safe)?;
        let meta = std::fs::metadata(abs.as_path().as_std_path()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::not_found(format!("file not found: {}", safe.as_str()))
            } else {
                CoreError::io(e.to_string())
            }
        });
        let meta = match meta {
            Ok(m) => m,
            Err(CoreError::NotFound(_)) => {
                // Spec lists a file not yet created — skip with warning
                findings.push(Finding {
                    path: safe.as_str().to_string(),
                    line: 1,
                    col: 1,
                    severity: Severity::Warning,
                    rule_id: "missing_file".into(),
                    message: "listed file not found on disk".into(),
                });
                continue;
            }
            Err(e) => return Err(e),
        };
        if meta.len() > MAX_FILE_BYTES {
            findings.push(Finding {
                path: safe.as_str().to_string(),
                line: 1,
                col: 1,
                severity: Severity::Warning,
                rule_id: "file_too_large".into(),
                message: format!("file exceeds {MAX_FILE_BYTES} bytes; skipped"),
            });
            continue;
        }
        let text = read_to_string(root, &safe)?;
        scan_text(safe.as_str(), &text, &mut findings);
        files_scanned = files_scanned.saturating_add(1);
    }

    let mut unmet = Vec::new();
    let mut notes = None;
    if let Some(ref agent_rel) = opts.from_agent {
        let agent_path = SafeRelativePath::new(agent_rel)?;
        let raw = read_to_string(root, &agent_path)?;
        let semantic = load_agent_semantic(&raw)?;
        notes = semantic.notes;
        if !semantic.passed || !semantic.unmet_criteria.is_empty() {
            unmet = semantic.unmet_criteria;
            if unmet.is_empty() {
                unmet.push("agent semantic review reported passed=false".into());
            }
        }
    }

    let mut enriched = false;
    if opts.ai {
        // Classe B soft stub — static scan always runs; no LLM call.
        findings.push(Finding {
            path: spec_rel.clone(),
            line: 1,
            col: 1,
            severity: Severity::Warning,
            rule_id: "enrichment_stub".into(),
            message: "AI enrichment not implemented; use --from-agent for semantic merge (Class B)"
                .into(),
        });
        enriched = false;
        notes = Some(match notes {
            Some(n) => format!("{n}; enrichment_stub"),
            None => "enrichment_stub".into(),
        });
    }

    sort_findings(&mut findings);
    Ok(new_report(
        &opts.task_id,
        findings,
        unmet,
        opts.strict,
        opts.fail_on,
        enriched,
        files_scanned,
        notes,
        opts.comment,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::should_fail_exit;
    use crate::types::FailOn;
    use tempfile::tempdir;

    fn write(root: &std::path::Path, rel: &str, body: &str) {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(p, body).unwrap();
    }

    #[test]
    fn run_review_detects_todo_and_from_agent() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        write(
            dir.path(),
            "DARE/EXECUTION/task-a.md",
            r#"# TASK

## 3. ARQUIVOS A CRIAR / MODIFICAR

| Ação | Caminho | Descrição |
|------|---------|-----------|
| CRIAR | `src/lib.rs` | x |

## 4. X
"#,
        );
        write(dir.path(), "src/lib.rs", "fn f() {\n  // TODO: later\n}\n");
        write(
            dir.path(),
            ".dare/agent.json",
            r#"{"passed":false,"unmetCriteria":["criterion X"]}"#,
        );

        let report = run_review(
            &root,
            &ReviewOptions {
                task_id: "task-a".into(),
                files_override: None,
                strict: false,
                errors_only: false,
                from_agent: Some(".dare/agent.json".into()),
                format: OutputFormat::Human,
                comment: true,
                fail_on: FailOn::Error,
                ai: false,
            },
        )
        .unwrap();
        assert!(report.error_count >= 1);
        assert!(!report.unmet_criteria.is_empty());
        assert!(should_fail_exit(&report, FailOn::Error));
        assert!(report.comment_markdown.is_some());
    }

    #[test]
    fn missing_spec_not_found() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let err = run_review(
            &root,
            &ReviewOptions {
                task_id: "missing".into(),
                files_override: None,
                strict: false,
                errors_only: false,
                from_agent: None,
                format: OutputFormat::Human,
                comment: false,
                fail_on: FailOn::Error,
                ai: false,
            },
        )
        .unwrap_err();
        assert!(matches!(err, CoreError::NotFound(_)));
    }
}
