//! `run_enrich` — provider enrich + schema validate + optional `--write` (BLUEPRINT-050 §0.5 / §4.3 / §5.2).

use std::time::Instant;

use dare_core::{
    fs::atomic_write, CoreResult, ProcessRunner, ProjectRoot, SafeRelativePath,
};
use serde::{Deserialize, Serialize};

use crate::command_registry::sections_for_command;
use crate::inject::inject_sections;
use crate::provider::{resolve_provider, ProviderId};
use crate::request::EnrichRequest;
use crate::schema::parse_and_validate_sections_with;

/// Frozen JSON schema version for [`RunReport`] (BLUEPRINT-050 §0.1 / §4.3).
pub const AI_REPORT_SCHEMA: u32 = 1;

/// Template for unimplemented provider errors (BLUEPRINT-050 §0.1).
/// Concrete errors interpolate `{id}` with the provider id string.
pub const MSG_PROVIDER_NOT_IMPL: &str = "provider not implemented: {id}";

/// `--write` requires an explicit markdown path (BLUEPRINT-050 §0.1).
pub const MSG_WRITE_NEEDS_MARKDOWN: &str = "--write requires --markdown <path>";

/// Domain request for [`run_enrich`].
pub struct RunEnrichRequest {
    pub provider: ProviderId,
    pub command: String,
    pub title: String,
    pub description: String,
    pub markdown: String,
    pub cwd: (ProjectRoot, SafeRelativePath),
    /// `Some` ⇒ `--write` opt-in (inject + [`atomic_write`]).
    pub write_rel: Option<SafeRelativePath>,
}

/// Run enrich report (`schemaVersion` 1, camelCase JSON).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RunReport {
    pub schema_version: u32,
    pub ok: bool,
    pub command: String,
    pub provider: String,
    pub enriched: bool,
    pub written: bool,
    pub write_path: Option<String>,
    pub sections: Vec<String>,
    pub duration_ms: u64,
    pub warnings: Vec<String>,
}

/// Resolve provider, enrich, validate sections; optionally inject + atomic write.
///
/// `runner` is reserved for process-backed providers; mock ignores it (providers
/// embed their own runner via [`resolve_provider`]).
pub fn run_enrich(
    req: &RunEnrichRequest,
    runner: &dyn ProcessRunner,
) -> CoreResult<RunReport> {
    let _ = runner;

    let started = Instant::now();
    let section_ids = sections_for_command(&req.command)?;
    // Unimplemented providers → invalid_input with MSG_PROVIDER_NOT_IMPL shape.
    let provider = resolve_provider(req.provider)?;

    let enrich_req = EnrichRequest {
        command: req.command.clone(),
        title: req.title.clone(),
        description: req.description.clone(),
        current_markdown: req.markdown.clone(),
        cwd: Some(req.cwd.clone()),
    };

    let raw = provider.enrich(&enrich_req)?;
    let sections = parse_and_validate_sections_with(&raw.stdout, section_ids)?;

    let mut written = false;
    let mut write_path = None;

    if let Some(rel) = &req.write_rel {
        let injected = inject_sections(&req.markdown, &sections, section_ids)?;
        atomic_write(&req.cwd.0, rel, injected.as_bytes())?;
        written = true;
        write_path = Some(rel.as_str().to_string());
    }

    let duration_ms = started.elapsed().as_millis() as u64;
    let section_names: Vec<String> = section_ids.iter().map(|s| (*s).to_string()).collect();

    Ok(RunReport {
        schema_version: AI_REPORT_SCHEMA,
        ok: true,
        command: req.command.clone(),
        provider: req.provider.as_str().to_string(),
        enriched: true,
        written,
        write_path,
        sections: section_names,
        duration_ms,
        warnings: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::{ErrorKind, MockProcessRunner};
    use std::fs;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn design_markdown() -> String {
        let begin = |id: &str| format!("<!-- AGENT:BEGIN section=\"{id}\" -->");
        let end = |id: &str| format!("<!-- AGENT:END section=\"{id}\" -->");
        format!(
            "# Design\n\n\
             Unmanaged paragraph must survive.\n\n\
             {d0}\nold description\n{d1}\n\n\
             {o0}\nold objectives\n{o1}\n\n\
             {f0}\nold fr\n{f1}\n\n\
             {s0}\nold stack\n{s1}\n",
            d0 = begin("description"),
            d1 = end("description"),
            o0 = begin("objectives"),
            o1 = end("objectives"),
            f0 = begin("functional-requirements"),
            f1 = end("functional-requirements"),
            s0 = begin("stack"),
            s1 = end("stack"),
        )
    }

    fn sample_req(
        root: ProjectRoot,
        markdown: String,
        write_rel: Option<SafeRelativePath>,
    ) -> RunEnrichRequest {
        let cwd_rel = SafeRelativePath::new(".").expect("cwd");
        RunEnrichRequest {
            provider: ProviderId::Mock,
            command: "design".into(),
            title: "Payments API".into(),
            description: "API de pagamentos com Stripe".into(),
            markdown,
            cwd: (root, cwd_rel),
            write_rel,
        }
    }

    #[test]
    fn run_mock_ok() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::remove_var("DARE_AI_MOCK_MODE");

        let dir = tempdir().expect("temp");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let md = design_markdown();
        let req = sample_req(root, md, None);
        let runner = MockProcessRunner::new();

        let report = run_enrich(&req, &runner).expect("run_enrich");
        assert_eq!(report.schema_version, AI_REPORT_SCHEMA);
        assert_eq!(report.schema_version, 1);
        assert!(report.ok);
        assert!(report.enriched);
        assert!(!report.written);
        assert!(report.write_path.is_none());
        assert_eq!(report.command, "design");
        assert_eq!(report.provider, "mock");
        assert_eq!(
            report.sections,
            vec![
                "description",
                "objectives",
                "functional-requirements",
                "stack",
            ]
        );
        assert!(report.warnings.is_empty());

        let json = serde_json::to_string(&report).expect("serialize");
        assert!(json.contains("\"schemaVersion\":1"));
        assert!(json.contains("\"writePath\":null"));
        assert!(json.contains("\"durationMs\""));
    }

    struct ClearMockMode;

    impl Drop for ClearMockMode {
        fn drop(&mut self) {
            std::env::remove_var("DARE_AI_MOCK_MODE");
        }
    }

    #[test]
    fn schema_fail_no_write() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let _clear = ClearMockMode;
        std::env::set_var("DARE_AI_MOCK_MODE", "invalid-json");

        let dir = tempdir().expect("temp");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let rel = SafeRelativePath::new("DARE/DESIGN.md").expect("rel");
        let original = design_markdown();
        atomic_write(&root, &rel, original.as_bytes()).expect("seed");

        let req = sample_req(root.clone(), original.clone(), Some(rel.clone()));
        let runner = MockProcessRunner::new();
        let err = run_enrich(&req, &runner).expect_err("schema fail");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);

        let after = dare_core::fs::read_to_string(&root, &rel).expect("read");
        assert_eq!(after, original, "must not write on schema failure");
    }

    #[test]
    fn write_injects_markers() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::remove_var("DARE_AI_MOCK_MODE");

        let dir = tempdir().expect("temp");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let rel = SafeRelativePath::new("DARE/DESIGN.md").expect("rel");
        let original = design_markdown();
        atomic_write(&root, &rel, original.as_bytes()).expect("seed");

        let req = sample_req(root.clone(), original, Some(rel.clone()));
        let runner = MockProcessRunner::new();
        let report = run_enrich(&req, &runner).expect("write ok");
        assert!(report.written);
        assert_eq!(report.write_path.as_deref(), Some("DARE/DESIGN.md"));
        assert!(report.enriched);

        let after = dare_core::fs::read_to_string(&root, &rel).expect("read");
        assert!(after.contains("Unmanaged paragraph must survive."));
        assert!(after.contains("<!-- AGENT:BEGIN section=\"description\" -->"));
        assert!(after.contains("<!-- AGENT:END section=\"description\" -->"));
        assert!(after.contains("API de pagamentos com Stripe"));
        assert!(after.contains("Generated by mock"));
        assert!(!after.contains("old description"));
        assert!(!after.contains("old stack"));
    }

    #[test]
    fn write_requires_path() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::remove_var("DARE_AI_MOCK_MODE");

        let dir = tempdir().expect("temp");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let rel = SafeRelativePath::new("DARE/DESIGN.md").expect("rel");
        let original = design_markdown();
        atomic_write(&root, &rel, original.as_bytes()).expect("seed");

        // No write_rel ⇒ must not touch disk (path required for --write).
        let req = sample_req(root.clone(), original.clone(), None);
        let runner = MockProcessRunner::new();
        let report = run_enrich(&req, &runner).expect("enrich without write");
        assert!(!report.written);
        assert!(report.write_path.is_none());

        let after = dare_core::fs::read_to_string(&root, &rel).expect("read");
        assert_eq!(after, original);
        assert!(MSG_WRITE_NEEDS_MARKDOWN.contains("--write requires --markdown"));
    }

    #[test]
    fn not_implemented_provider() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::remove_var("DARE_AI_MOCK_MODE");

        let dir = tempdir().expect("temp");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let mut req = sample_req(root, design_markdown(), None);
        req.provider = ProviderId::ClaudeCode;
        let runner = MockProcessRunner::new();
        let err = run_enrich(&req, &runner).expect_err("not impl");
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert_eq!(
            err.message(),
            "provider not implemented: claude-code"
        );
        assert!(MSG_PROVIDER_NOT_IMPL.contains("provider not implemented"));
    }

    #[test]
    fn fixture_valid_sections_shape() {
        // Reuse fixtures/ai without mass CRLF: read + parse only.
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("../../tests/fixtures/ai/mock-sections-valid.json");
        let raw = fs::read_to_string(&p).expect("fixture");
        let sections = parse_and_validate_sections_with(
            &raw,
            &[
                "description",
                "objectives",
                "functional-requirements",
                "stack",
            ],
        )
        .expect("valid fixture");
        assert_eq!(sections["description"], "API de pagamentos com Stripe");
    }
}
