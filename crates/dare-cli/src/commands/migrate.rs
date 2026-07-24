//! `dare migrate` — migration plan + parity skeletons (microplano 039).

use std::path::PathBuf;

use dare_ai::{
    inject_sections, parse_and_validate_sections_with, resolve_provider, EnrichRequest, ProviderId,
};
use dare_core::fs::{atomic_write, read_to_string};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use dare_project::{
    format_migrate_human, migrate_report_to_json, run_migrate, MigrateOptions, MIGRATION_MD_REL,
};
use serde_json::Value;

/// Sections injectable in MIGRATION.md (soft-fail enrichment).
const MIGRATE_ENRICHABLE: &[&str] = &[
    "paradigm",
    "strategy",
    "risk-register",
    "target-architecture",
    "cutover-rollback",
];

pub struct MigrateCliOpts {
    pub to: String,
    pub dir: Option<PathBuf>,
    pub check: bool,
    pub ai: bool,
    pub provider: Option<String>,
}

/// Run migrate: plan artifacts, optional writes, optional soft-fail AI.
pub fn run_migrate_cmd(opts: MigrateCliOpts) -> CoreResult<(String, Value)> {
    if opts.provider.is_some() && !opts.ai {
        return Err(CoreError::usage("--provider requires --ai"));
    }

    let start: PathBuf = opts
        .dir
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let mut report = run_migrate(
        &start,
        &MigrateOptions {
            to_stack: opts.to,
            check: opts.check,
            ai: opts.ai,
        },
    )?;

    if opts.ai && !opts.check {
        let warnings = maybe_enrich(&start, &opts.provider)?;
        report.warnings.extend(warnings);
    }

    let human = format_migrate_human(&report);
    let json_str = migrate_report_to_json(&report)?;
    let data: Value = serde_json::from_str(&json_str)
        .map_err(|e| CoreError::io(format!("parse migrate report json: {e}")))?;
    Ok((human, data))
}

fn maybe_enrich(start: &std::path::Path, provider: &Option<String>) -> CoreResult<Vec<String>> {
    let mut warnings = Vec::new();
    let Some(pr) = dare_project::find_project_root(start) else {
        warnings.push("AI enrichment skipped: project root not found".into());
        return Ok(warnings);
    };
    let root = match ProjectRoot::new(&pr) {
        Ok(r) => r,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok(warnings);
        }
    };

    let pid = match provider.as_deref() {
        None => ProviderId::Codex,
        Some(s) => match ProviderId::parse(s) {
            Ok(p) => p,
            Err(e) => {
                warnings.push(format!("AI enrichment skipped: {}", e.message()));
                return Ok(warnings);
            }
        },
    };

    let prov = match resolve_provider(pid) {
        Ok(p) => p,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok(warnings);
        }
    };

    let rel = match SafeRelativePath::new(MIGRATION_MD_REL) {
        Ok(r) => r,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok(warnings);
        }
    };

    let md = match read_to_string(&root, &rel) {
        Ok(m) => m,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok(warnings);
        }
    };

    let cwd_rel = match SafeRelativePath::new("DARE/MIGRATION") {
        Ok(r) => r,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok(warnings);
        }
    };

    let req = EnrichRequest {
        command: "migrate".into(),
        title: "MIGRATION plan enrichment".into(),
        description: "Fill AGENT sections in MIGRATION.md from migrate facts.".into(),
        current_markdown: md.clone(),
        cwd: Some((root.clone(), cwd_rel)),
    };

    let raw = match prov.enrich(&req) {
        Ok(r) => r,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok(warnings);
        }
    };

    let sections = match parse_and_validate_sections_with(&raw.stdout, MIGRATE_ENRICHABLE) {
        Ok(s) => s,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return Ok(warnings);
        }
    };

    match inject_sections(&md, &sections, MIGRATE_ENRICHABLE) {
        Ok(injected) => {
            if let Err(e) = atomic_write(&root, &rel, injected.as_bytes()) {
                warnings.push(format!("AI enrichment skipped: {}", e.message()));
            }
            Ok(warnings)
        }
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            Ok(warnings)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_without_ai_is_usage_error() {
        let err = run_migrate_cmd(MigrateCliOpts {
            to: "rust-axum".into(),
            dir: None,
            check: true,
            ai: false,
            provider: Some("codex".into()),
        })
        .unwrap_err();
        assert!(matches!(err, CoreError::Usage(_)));
        assert!(err.message().contains("--provider requires --ai"));
    }
}
