//! `dare ai doctor|providers|run|prompt` — AI enrich CLI (microplano 050).

use std::path::PathBuf;

use dare_ai::{
    diagnose_all, diagnose_provider, list_provider_capabilities, prompt_preview, run_enrich,
    sections_for_command, DoctorReport, EnrichRequest, ProviderId, RunEnrichRequest,
    MSG_WRITE_NEEDS_MARKDOWN,
};
use dare_core::fs::read_to_string;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath, SystemProcessRunner};
use serde_json::Value;

/// Default provider when `--provider` is omitted (BLUEPRINT-050 §0.1).
pub const DEFAULT_PROVIDER: &str = "codex";

/// CLI usage when neither `--facts` nor `--markdown` is given.
pub const MSG_FACTS_REQUIRED: &str = "--facts or --markdown required for ai run/prompt";

/// Options shared by `ai run` / `ai prompt`.
pub struct AiEnrichCliOpts {
    pub command: String,
    pub provider: Option<String>,
    pub facts: Option<String>,
    pub markdown: Option<String>,
    pub write: bool,
    pub dir: Option<PathBuf>,
}

/// `dare ai doctor [--provider <id>] [-d <dir>]`
pub fn run_ai_doctor(
    provider: Option<String>,
    dir: Option<PathBuf>,
) -> CoreResult<(String, Value)> {
    let _root = resolve_root(dir)?;
    let report = match provider {
        Some(id) => {
            let pid = ProviderId::parse(&id)?;
            let entry = diagnose_provider(pid)?;
            DoctorReport {
                schema_version: dare_ai::DOCTOR_SCHEMA_VERSION,
                ok: true,
                providers: vec![entry],
            }
        }
        None => diagnose_all()?,
    };
    let human = format_doctor_human(&report);
    let data = serde_json::to_value(&report).map_err(|e| CoreError::internal(e.to_string()))?;
    Ok((human, data))
}

/// `dare ai providers [-d <dir>]`
pub fn run_ai_providers(dir: Option<PathBuf>) -> CoreResult<(String, Value)> {
    let _root = resolve_root(dir)?;
    let report = list_provider_capabilities();
    let human = format_providers_human(&report);
    let data = serde_json::to_value(&report).map_err(|e| CoreError::internal(e.to_string()))?;
    Ok((human, data))
}

/// `dare ai prompt --command …`
pub fn run_ai_prompt(opts: AiEnrichCliOpts) -> CoreResult<(String, Value)> {
    let root = resolve_root(opts.dir)?;
    let provider = parse_provider(opts.provider.as_deref())?;
    let loaded = load_facts_markdown(&root, &opts.command, opts.facts.as_deref(), opts.markdown.as_deref())?;
    let section_ids = sections_for_command(&opts.command)?;
    let cwd_rel = SafeRelativePath::new(".")?;
    let enrich_req = EnrichRequest {
        command: opts.command.clone(),
        title: loaded.title,
        description: loaded.description,
        current_markdown: loaded.markdown,
        cwd: Some((root, cwd_rel)),
    };
    let mut report = prompt_preview(&enrich_req, section_ids);
    report.provider = provider.as_str().to_string();
    let human = format!(
        "ai prompt: command={} provider={} chars={} envLeaked={}",
        report.command, report.provider, report.prompt_chars, report.env_leaked
    );
    let data = serde_json::to_value(&report).map_err(|e| CoreError::internal(e.to_string()))?;
    Ok((human, data))
}

/// `dare ai run --command …`
pub fn run_ai_run(opts: AiEnrichCliOpts) -> CoreResult<(String, Value)> {
    if opts.write && opts.markdown.is_none() {
        return Err(CoreError::usage(MSG_WRITE_NEEDS_MARKDOWN));
    }
    let root = resolve_root(opts.dir)?;
    let provider = parse_provider(opts.provider.as_deref())?;
    let loaded = load_facts_markdown(&root, &opts.command, opts.facts.as_deref(), opts.markdown.as_deref())?;
    let write_rel = if opts.write {
        Some(SafeRelativePath::new(
            opts.markdown
                .as_deref()
                .ok_or_else(|| CoreError::usage(MSG_WRITE_NEEDS_MARKDOWN))?,
        )?)
    } else {
        None
    };
    let cwd_rel = SafeRelativePath::new(".")?;
    let req = RunEnrichRequest {
        provider,
        command: opts.command,
        title: loaded.title,
        description: loaded.description,
        markdown: loaded.markdown,
        cwd: (root, cwd_rel),
        write_rel,
    };
    let report = run_enrich(&req, &SystemProcessRunner)?;
    let human = format!(
        "ai run: command={} provider={} enriched={} written={}",
        report.command, report.provider, report.enriched, report.written
    );
    let data = serde_json::to_value(&report).map_err(|e| CoreError::internal(e.to_string()))?;
    Ok((human, data))
}

/// Map domain errors to CLI exit codes (0/1/2/3/4/5/124).
pub fn ai_exit_code(err: &CoreError) -> i32 {
    if err.message().contains("provider timed out") {
        return 124;
    }
    err.exit_code()
}

fn parse_provider(raw: Option<&str>) -> CoreResult<ProviderId> {
    ProviderId::parse(raw.unwrap_or(DEFAULT_PROVIDER))
}

fn resolve_root(dir: Option<PathBuf>) -> CoreResult<ProjectRoot> {
    let path =
        dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    ProjectRoot::new(&path)
}

struct LoadedInput {
    title: String,
    description: String,
    markdown: String,
}

/// Load facts/markdown per BLUEPRINT-050 §5.4.
fn load_facts_markdown(
    root: &ProjectRoot,
    command: &str,
    facts: Option<&str>,
    markdown: Option<&str>,
) -> CoreResult<LoadedInput> {
    if facts.is_none() && markdown.is_none() {
        return Err(CoreError::usage(MSG_FACTS_REQUIRED));
    }

    let mut title = command.to_string();
    let mut description = format!("ai run {command}");
    let mut body = String::new();

    if let Some(facts_rel) = facts {
        let rel = SafeRelativePath::new(facts_rel)?;
        let raw = read_to_string(root, &rel)?;
        let v: Value = serde_json::from_str(&raw).map_err(|e| {
            CoreError::invalid_input(format!("malformed facts JSON: {e}"))
        })?;
        let obj = v.as_object().ok_or_else(|| {
            CoreError::invalid_input("facts must be a JSON object with title and description")
        })?;
        title = obj
            .get("title")
            .and_then(|x| x.as_str())
            .ok_or_else(|| CoreError::invalid_input("facts missing string field: title"))?
            .to_string();
        description = obj
            .get("description")
            .and_then(|x| x.as_str())
            .ok_or_else(|| CoreError::invalid_input("facts missing string field: description"))?
            .to_string();

        if let Some(md) = obj.get("markdown").and_then(|x| x.as_str()) {
            body = md.to_string();
        } else if let Some(path) = obj.get("markdownPath").and_then(|x| x.as_str()) {
            let md_rel = SafeRelativePath::new(path)?;
            body = read_to_string(root, &md_rel)?;
        }
    }

    // Explicit `--markdown` wins for body when present (§5.4).
    if let Some(md_rel) = markdown {
        let rel = SafeRelativePath::new(md_rel)?;
        body = read_to_string(root, &rel)?;
    }

    Ok(LoadedInput {
        title,
        description,
        markdown: body,
    })
}

fn format_doctor_human(report: &DoctorReport) -> String {
    let mut lines = vec![format!(
        "ai doctor: ok={} providers={}",
        report.ok,
        report.providers.len()
    )];
    for p in &report.providers {
        let reason = p.reason.as_deref().unwrap_or("-");
        lines.push(format!(
            "  {}  status={:?}  program={}  reason={}",
            p.id,
            p.status,
            p.program,
            reason
        ));
    }
    lines.join("\n")
}

fn format_providers_human(report: &dare_ai::ProvidersReport) -> String {
    let mut lines = vec![format!(
        "ai providers: count={}",
        report.providers.len()
    )];
    for p in &report.providers {
        lines.push(format!(
            "  {}  implemented={}  enrich={}",
            p.id, p.implemented, p.enrich
        ));
    }
    lines.join("\n")
}
