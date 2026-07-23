//! `dare reverse` — brownfield reverse engineering (microplano 036).

use std::path::PathBuf;

use dare_ai::{
    inject_sections, parse_and_validate_sections_with, resolve_provider, EnrichRequest, ProviderId,
};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use dare_project::{
    format_reverse_human, read_ideia, reverse, reverse_report_to_json, write_ideia, ReverseOptions,
};
use serde_json::Value;

/// Sections injectable in IDEIA.md (soft-fail enrichment).
const REVERSE_ENRICHABLE: &[&str] = &[
    "purpose",
    "domain",
    "data-model",
    "api-surface",
    "system-flow",
    "gaps",
];

pub struct ReverseCliOpts {
    pub dir: Option<PathBuf>,
    pub check: bool,
    pub deep: bool,
    pub modules: Option<String>,
    pub ast: bool,
    pub no_excalidraw: bool,
    pub report: bool,
    pub ai: bool,
    pub provider: Option<String>,
}

/// Run reverse: analyze modules, optional writes, optional soft-fail AI.
pub fn run_reverse(opts: ReverseCliOpts) -> CoreResult<(String, Value)> {
    if opts.provider.is_some() && !opts.ai {
        return Err(CoreError::usage("--provider requires --ai"));
    }

    let start: PathBuf = opts
        .dir
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let modules = parse_modules_flag(opts.modules.as_deref())?;

    let rev_opts = ReverseOptions {
        check: opts.check,
        deep: opts.deep,
        modules,
        ast: opts.ast,
        excalidraw: !opts.no_excalidraw,
        report: opts.report,
    };

    let mut report = reverse(&start, &rev_opts)?;

    if opts.ai && !opts.check {
        let (enriched, warnings) = maybe_enrich(&start, &opts.provider)?;
        report.enriched = enriched;
        report.warnings.extend(warnings);
    }

    let human = format_reverse_human(&report);
    let data = reverse_report_to_json(&report);
    Ok((human, data))
}

fn parse_modules_flag(raw: Option<&str>) -> CoreResult<Vec<String>> {
    let Some(s) = raw else {
        return Ok(Vec::new());
    };
    let parts: Vec<String> = s
        .split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        return Err(CoreError::invalid_input(
            "--modules requires at least one module id",
        ));
    }
    Ok(parts)
}

fn maybe_enrich(
    start: &std::path::Path,
    provider: &Option<String>,
) -> CoreResult<(bool, Vec<String>)> {
    let mut warnings = Vec::new();
    let Some(pr) = dare_project::find_project_root(start) else {
        warnings.push("AI enrichment skipped: project root not found".into());
        return Ok((false, warnings));
    };
    let root = match ProjectRoot::new(&pr) {
        Ok(r) => r,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok((false, warnings));
        }
    };

    let pid = match provider.as_deref() {
        None => ProviderId::Codex,
        Some(s) => match ProviderId::parse(s) {
            Ok(p) => p,
            Err(e) => {
                warnings.push(format!("AI enrichment skipped: {}", e.message()));
                return Ok((false, warnings));
            }
        },
    };

    let prov = match resolve_provider(pid) {
        Ok(p) => p,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok((false, warnings));
        }
    };

    let md = match read_ideia(&root) {
        Ok(m) => m,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok((false, warnings));
        }
    };

    let cwd_rel = match SafeRelativePath::new("DARE") {
        Ok(r) => r,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok((false, warnings));
        }
    };

    let req = EnrichRequest {
        command: "reverse".into(),
        title: "IDEIA reverse enrichment".into(),
        description: "Fill AGENT sections in IDEIA.md from reverse facts.".into(),
        current_markdown: md.clone(),
        cwd: Some((root.clone(), cwd_rel)),
    };

    let raw = match prov.enrich(&req) {
        Ok(r) => r,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok((false, warnings));
        }
    };

    let sections = match parse_and_validate_sections_with(&raw.stdout, REVERSE_ENRICHABLE) {
        Ok(s) => s,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok((false, warnings));
        }
    };

    match inject_sections(&md, &sections, REVERSE_ENRICHABLE) {
        Ok(injected) => match write_ideia(&root, &injected) {
            Ok(()) => Ok((true, warnings)),
            Err(e) => {
                warnings.push(format!("AI enrichment skipped: {}", e.message()));
                Ok((false, warnings))
            }
        },
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            Ok((false, warnings))
        }
    }
}
