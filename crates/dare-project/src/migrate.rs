//! Migration plan domain — types, allowlist, compare, phases, gaps & I/O (microplano 039).

use std::fs;
use std::path::Path;

use dare_core::fs::atomic_write;
use dare_core::redact;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};

use crate::root::find_project_root;
use crate::stacks::detect_stacks;

/// Frozen JSON schema version for migrate reports/facts.
pub const MIGRATE_SCHEMA_VERSION: u32 = 1;

pub const MIGRATION_DIR: &str = "DARE/MIGRATION";
pub const MIGRATION_MD_REL: &str = "DARE/MIGRATION/MIGRATION.md";
pub const MIGRATION_FACTS_REL: &str = "DARE/MIGRATION/migration-facts.json";
pub const PARITY_DIR_REL: &str = "DARE/MIGRATION/parity";
pub const IDEIA_REL: &str = "DARE/IDEIA.md";
pub const REVERSE_FACTS_REL: &str = "DARE/REVERSE/reverse-facts.json";
pub const PROJECT_DNA_REL: &str = "DARE/PROJECT-DNA.md";
pub const PATTERNS_MD_REL: &str = "DARE/PATTERNS.md";
pub const MAX_MODULES: usize = 64;
pub const MSG_CHECK: &str = "mode: check (zero mutations)";

/// Closed allowlist for `dare migrate --to` (case-sensitive, lowercase ids).
pub const MIGRATE_TARGET_ALLOWLIST: &[&str] = &[
    "node-nestjs",
    "python-fastapi",
    "php-laravel",
    "go-gin",
    "go-stdlib",
    "rails",
    "rust-axum",
    "rust",
    "rust-leptos",
    "rust-leptos-csr",
    "react",
    "vue",
    "mcp-node-ts",
];

/// Options for `dare migrate` (AI reserved for CLI; domain ignores).
#[derive(Debug, Clone)]
pub struct MigrateOptions {
    pub to_stack: String,
    pub check: bool,
    /// Reserved for CLI; domain ignores AI (CLI owns enrich).
    pub ai: bool,
}

/// Deterministic blocking/warning gap in the migration plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockingGap {
    pub id: String,
    pub severity: String, // "blocking" | "warning"
    pub evidence: String,
    pub detail: String,
}

/// One ordered phase of the migration plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationPhase {
    pub id: String, // foundations | modules | cutover
    pub title: String,
    pub modules: Vec<String>,
    pub evidence: Vec<String>,
}

/// CLI/JSON migrate report (schemaVersion 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrateReport {
    pub schema_version: u32,
    pub mode: String, // "write" | "check"
    pub from_stacks: Vec<String>,
    pub to_stack: String,
    pub to_family: String,
    pub comparison: String, // same_family | cross_stack | unknown_origin
    pub phases: Vec<MigrationPhase>,
    pub blocking_gaps: Vec<BlockingGap>,
    pub module_ids: Vec<String>,
    pub written: Vec<String>,
    pub warnings: Vec<String>,
}

/// Map `--to` allowlist id → family used for comparison.
pub fn target_family(to: &str) -> Option<&'static str> {
    match to.trim() {
        "node-nestjs" | "react" | "vue" | "mcp-node-ts" => Some("node"),
        "python-fastapi" => Some("python"),
        "php-laravel" => Some("php"),
        "go-gin" | "go-stdlib" => Some("go"),
        "rails" => Some("ruby"),
        "rust-axum" | "rust" | "rust-leptos" | "rust-leptos-csr" => Some("rust"),
        _ => None,
    }
}

/// Validate `--to` against the frozen allowlist (trim; empty/unknown → InvalidInput).
pub fn validate_migrate_target(to: &str) -> CoreResult<()> {
    let trimmed = to.trim();
    if trimmed.is_empty() {
        return Err(CoreError::invalid_input(
            "unknown migrate target: empty --to (not in allowlist)",
        ));
    }
    if !MIGRATE_TARGET_ALLOWLIST.contains(&trimmed) {
        return Err(CoreError::invalid_input(format!(
            "unknown migrate target '{trimmed}' (not in allowlist)"
        )));
    }
    Ok(())
}

/// Compare origin stack families vs target family.
///
/// - `same_family` — any `from_families` equals `to_family`
/// - `cross_stack` — from non-empty and no match
/// - `unknown_origin` — from empty
pub fn compare_migration(from_families: &[String], to_family: &str) -> String {
    if from_families.is_empty() {
        return "unknown_origin".to_string();
    }
    if from_families.iter().any(|f| f == to_family) {
        "same_family".to_string()
    } else {
        "cross_stack".to_string()
    }
}

/// Build the three fixed phases: foundations → modules → cutover.
pub fn build_phases(
    module_ids: &[String],
    from_stacks: &[String],
    to_stack: &str,
) -> Vec<MigrationPhase> {
    let from_ev = if from_stacks.is_empty() {
        "fromStacks: (none)".to_string()
    } else {
        format!("fromStacks: {}", from_stacks.join(","))
    };
    let to_ev = format!("toStack: {to_stack}");

    let mut module_evidence: Vec<String> = module_ids
        .iter()
        .map(|id| format!("DARE/REVERSE/module-{id}.md"))
        .collect();
    module_evidence.sort();

    let mut parity_evidence: Vec<String> = module_ids
        .iter()
        .map(|id| format!("DARE/MIGRATION/parity/{id}.feature"))
        .collect();
    parity_evidence.sort();
    if parity_evidence.is_empty() {
        parity_evidence.push("DARE/MIGRATION/parity/*.feature".to_string());
    }

    vec![
        MigrationPhase {
            id: "foundations".to_string(),
            title: "Foundations & toolchain".to_string(),
            modules: Vec::new(),
            evidence: vec![from_ev, to_ev, "dare.config.json".to_string()],
        },
        MigrationPhase {
            id: "modules".to_string(),
            title: "Module reimplementation".to_string(),
            modules: module_ids.to_vec(),
            evidence: module_evidence,
        },
        MigrationPhase {
            id: "cutover".to_string(),
            title: "Cutover & parity validation".to_string(),
            modules: Vec::new(),
            evidence: parity_evidence,
        },
    ]
}

/// Emit deterministic gaps from comparison + optional artifact presence + conflicts.
pub fn build_blocking_gaps(
    comparison: &str,
    has_project_dna: bool,
    has_patterns: bool,
    has_stack_conflicts: bool,
) -> Vec<BlockingGap> {
    let mut gaps = Vec::new();

    if !has_project_dna {
        gaps.push(BlockingGap {
            id: "gap-no-dna".to_string(),
            severity: "warning".to_string(),
            evidence: "DARE/PROJECT-DNA.md".to_string(),
            detail: "PROJECT-DNA.md missing; DNA conventions unavailable for migration plan"
                .to_string(),
        });
    }
    if !has_patterns {
        gaps.push(BlockingGap {
            id: "gap-no-patterns".to_string(),
            severity: "warning".to_string(),
            evidence: "DARE/PATTERNS.md".to_string(),
            detail: "PATTERNS.md missing; mined patterns unavailable for migration plan"
                .to_string(),
        });
    }
    if comparison == "cross_stack" {
        gaps.push(BlockingGap {
            id: "gap-cross-stack".to_string(),
            severity: "blocking".to_string(),
            evidence: "comparison".to_string(),
            detail: "origin and target families differ (cross_stack)".to_string(),
        });
    }
    if comparison == "unknown_origin" {
        gaps.push(BlockingGap {
            id: "gap-unknown-origin".to_string(),
            severity: "blocking".to_string(),
            evidence: "fromStacks".to_string(),
            detail: "no origin stacks detected (unknown_origin)".to_string(),
        });
    }
    if has_stack_conflicts {
        gaps.push(BlockingGap {
            id: "gap-stack-conflict".to_string(),
            severity: "blocking".to_string(),
            evidence: "detect.conflicts".to_string(),
            detail: "stack detection reported non-empty conflicts".to_string(),
        });
    }

    sort_blocking_gaps(&mut gaps);
    gaps
}

/// Sort gaps: blocking before warning, then id ascending.
#[allow(clippy::ptr_arg)] // public API frozen as `&mut Vec` for callers that own the vec
pub fn sort_blocking_gaps(gaps: &mut Vec<BlockingGap>) {
    gaps.sort_by(|a, b| {
        let sev = severity_rank(&a.severity).cmp(&severity_rank(&b.severity));
        sev.then_with(|| a.id.cmp(&b.id))
    });
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "blocking" => 0,
        "warning" => 1,
        _ => 2,
    }
}

#[derive(Debug, Deserialize)]
struct ReverseFactsLite {
    #[serde(default)]
    modules: Vec<ReverseModuleLite>,
}

#[derive(Debug, Deserialize)]
struct ReverseModuleLite {
    id: String,
}

/// Ensure relative write path stays under `DARE/MIGRATION/**` (path jail).
fn migration_safe_rel(rel: &str) -> CoreResult<SafeRelativePath> {
    let safe = SafeRelativePath::new(rel)?;
    let s = safe.as_str();
    if s != MIGRATION_DIR && !s.starts_with(&format!("{MIGRATION_DIR}/")) {
        return Err(CoreError::invalid_input(
            "path must be relative and stay within DARE/MIGRATION",
        ));
    }
    Ok(safe)
}

fn write_migration_rel(
    root: &ProjectRoot,
    rel: &str,
    data: &str,
    written: &mut Vec<String>,
) -> CoreResult<()> {
    let safe = migration_safe_rel(rel)?;
    atomic_write(root, &safe, data.as_bytes())?;
    written.push(safe.as_str().to_string());
    Ok(())
}

fn load_module_ids(project_root: &Path) -> CoreResult<Vec<String>> {
    let facts_path = project_root.join(REVERSE_FACTS_REL);
    let mut ids = Vec::new();

    if facts_path.is_file() {
        let text = fs::read_to_string(&facts_path).map_err(|e| CoreError::io(e.to_string()))?;
        let lite: ReverseFactsLite = serde_json::from_str(&text).map_err(|e| {
            CoreError::invalid_input(format!("invalid reverse-facts.json: {e}"))
        })?;
        for m in lite.modules {
            let id = m.id.trim();
            if !id.is_empty() {
                ids.push(id.to_string());
            }
        }
    }

    if ids.is_empty() {
        let reverse_dir = project_root.join("DARE/REVERSE");
        if reverse_dir.is_dir() {
            let entries = fs::read_dir(&reverse_dir).map_err(|e| CoreError::io(e.to_string()))?;
            for ent in entries.flatten() {
                let name = ent.file_name().to_string_lossy().to_string();
                if let Some(id) = name
                    .strip_prefix("module-")
                    .and_then(|s| s.strip_suffix(".md"))
                {
                    if !id.is_empty() && ent.path().is_file() {
                        ids.push(id.to_string());
                    }
                }
            }
        }
    }

    ids.sort();
    ids.dedup();
    if ids.len() > MAX_MODULES {
        ids.truncate(MAX_MODULES);
    }
    Ok(ids)
}

fn redact_gaps(gaps: &mut [BlockingGap]) {
    for g in gaps.iter_mut() {
        g.evidence = redact(&g.evidence);
        g.detail = redact(&g.detail);
    }
}

fn redact_phases(phases: &mut [MigrationPhase]) {
    for p in phases.iter_mut() {
        for e in p.evidence.iter_mut() {
            *e = redact(e);
        }
    }
}

/// Exact Gherkin skeleton for parity (no invented business steps).
pub fn render_parity_feature(module_id: &str) -> String {
    format!(
        "Feature: Parity for module {module_id}\n\
         \x20\x20# dare:managed skeleton — fill via /dare-migrate\n\
         \x20\x20# evidence: DARE/REVERSE/module-{module_id}.md\n\
         \n\
         \x20\x20@module:{module_id} @parity @skeleton\n\
         \x20\x20Scenario: Observable behavior placeholder\n\
         \x20\x20\x20\x20Given the legacy module \"{module_id}\" is available\n\
         \x20\x20\x20\x20When a critical user flow of \"{module_id}\" is exercised\n\
         \x20\x20\x20\x20Then the target stack behavior matches the legacy outcomes\n"
    )
}

fn render_migration_md(report: &MigrateReport) -> String {
    let from = if report.from_stacks.is_empty() {
        "(unknown)".to_string()
    } else {
        report.from_stacks.join(",")
    };
    let mut out = String::new();
    out.push_str(&format!(
        "# Migration Plan: {from} → {}\n\n",
        report.to_stack
    ));
    out.push_str("## Summary\n\n");
    out.push_str("| Field | Value |\n");
    out.push_str("|-------|-------|\n");
    out.push_str(&format!(
        "| fromStacks | {} |\n",
        if report.from_stacks.is_empty() {
            "(none)".to_string()
        } else {
            report.from_stacks.join(", ")
        }
    ));
    out.push_str(&format!("| toStack | {} |\n", report.to_stack));
    out.push_str(&format!("| comparison | {} |\n", report.comparison));
    out.push_str(&format!("| modules | {} |\n\n", report.module_ids.len()));

    out.push_str("## Phases\n\n");
    for (i, phase) in report.phases.iter().enumerate() {
        out.push_str(&format!(
            "{}. **{}** — {}\n",
            i + 1,
            phase.id,
            phase.title
        ));
        if !phase.modules.is_empty() {
            out.push_str(&format!("   - modules: {}\n", phase.modules.join(", ")));
        }
        if !phase.evidence.is_empty() {
            out.push_str(&format!("   - evidence: {}\n", phase.evidence.join("; ")));
        }
        out.push('\n');
    }

    out.push_str("## Blocking gaps\n\n");
    out.push_str("| id | severity | evidence | detail |\n");
    out.push_str("|----|----------|----------|--------|\n");
    if report.blocking_gaps.is_empty() {
        out.push_str("| (none) | — | — | — |\n");
    } else {
        for g in &report.blocking_gaps {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                g.id, g.severity, g.evidence, g.detail
            ));
        }
    }
    out.push('\n');

    for section in [
        "paradigm",
        "strategy",
        "risk-register",
        "target-architecture",
        "cutover-rollback",
    ] {
        out.push_str(&format!("<!-- AGENT:BEGIN section=\"{section}\" -->\n"));
        out.push_str(&format!("<!-- AGENT:END section=\"{section}\" -->\n"));
    }
    out
}

/// Build migration plan; `--check` performs zero filesystem mutations under `DARE/MIGRATION`.
///
/// Domain ignores `opts.ai` (CLI owns enrich).
pub fn run_migrate(root: &Path, opts: &MigrateOptions) -> CoreResult<MigrateReport> {
    let _ = opts.ai; // reserved for CLI

    if !root.exists() || !root.is_dir() {
        return Err(CoreError::not_found(format!(
            "directory not found: {}",
            root.display()
        )));
    }

    validate_migrate_target(&opts.to_stack)?;
    let to_stack = opts.to_stack.trim().to_string();

    let Some(project_root) = find_project_root(root) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let project = ProjectRoot::new(&project_root)?;

    let ideia = project_root.join(IDEIA_REL);
    if !ideia.is_file() {
        return Err(CoreError::invalid_input(
            "run dare reverse first: missing DARE/IDEIA.md",
        ));
    }

    let module_ids = load_module_ids(&project_root)?;
    if module_ids.is_empty() {
        return Err(CoreError::invalid_input(
            "run dare reverse first: no modules",
        ));
    }

    let (stacks, conflicts) = detect_stacks(&project_root);
    let from_stacks: Vec<String> = stacks.iter().map(|s| s.id.clone()).collect();
    let from_families: Vec<String> = {
        let mut f: Vec<String> = stacks.iter().map(|s| s.family.clone()).collect();
        f.sort();
        f.dedup();
        f
    };

    let to_family = target_family(&to_stack)
        .ok_or_else(|| {
            CoreError::invalid_input(format!(
                "unknown migrate target '{to_stack}' (not in allowlist)"
            ))
        })?
        .to_string();

    let comparison = compare_migration(&from_families, &to_family);
    let has_project_dna = project_root.join(PROJECT_DNA_REL).is_file();
    let has_patterns = project_root.join(PATTERNS_MD_REL).is_file();
    let has_stack_conflicts = !conflicts.is_empty();

    let mut phases = build_phases(&module_ids, &from_stacks, &to_stack);
    let mut blocking_gaps =
        build_blocking_gaps(&comparison, has_project_dna, has_patterns, has_stack_conflicts);
    redact_phases(&mut phases);
    redact_gaps(&mut blocking_gaps);

    let mut warnings = Vec::new();
    if !has_project_dna {
        warnings.push("PROJECT-DNA.md missing; DNA conventions unavailable".to_string());
    }
    if !has_patterns {
        warnings.push("PATTERNS.md missing; mined patterns unavailable".to_string());
    }

    let mode = if opts.check { "check" } else { "write" };
    let mut written = Vec::new();

    let mut report = MigrateReport {
        schema_version: MIGRATE_SCHEMA_VERSION,
        mode: mode.to_string(),
        from_stacks,
        to_stack: to_stack.clone(),
        to_family,
        comparison,
        phases,
        blocking_gaps,
        module_ids: module_ids.clone(),
        written: Vec::new(),
        warnings,
    };

    if !opts.check {
        let md = render_migration_md(&report);
        write_migration_rel(&project, MIGRATION_MD_REL, &md, &mut written)?;

        // facts written after md; include written list with md + facts + parity paths
        let mut planned: Vec<String> = vec![
            MIGRATION_MD_REL.to_string(),
            MIGRATION_FACTS_REL.to_string(),
        ];
        for id in &module_ids {
            planned.push(format!("{PARITY_DIR_REL}/{id}.feature"));
        }
        planned.sort();
        report.written = planned.clone();

        let facts = serde_json::to_string_pretty(&report)
            .map_err(|e| CoreError::io(format!("serialize migration-facts: {e}")))?;
        write_migration_rel(&project, MIGRATION_FACTS_REL, &facts, &mut written)?;

        for id in &module_ids {
            let rel = format!("{PARITY_DIR_REL}/{id}.feature");
            let body = render_parity_feature(id);
            write_migration_rel(&project, &rel, &body, &mut written)?;
        }

        written.sort();
        report.written = written;
    }

    Ok(report)
}

/// Human-readable migrate report (en-US).
pub fn format_migrate_human(r: &MigrateReport) -> String {
    let mut out = String::new();
    if r.mode == "check" {
        out.push_str(MSG_CHECK);
        out.push('\n');
    } else {
        out.push_str("mode: write\n");
    }
    out.push_str(&format!(
        "fromStacks: {}\n",
        if r.from_stacks.is_empty() {
            "(none)".to_string()
        } else {
            r.from_stacks.join(",")
        }
    ));
    out.push_str(&format!("toStack: {}\n", r.to_stack));
    out.push_str(&format!("toFamily: {}\n", r.to_family));
    out.push_str(&format!("comparison: {}\n", r.comparison));
    out.push_str(&format!("modules: {}\n", r.module_ids.len()));
    out.push_str(&format!("phases: {}\n", r.phases.len()));
    out.push_str(&format!("blockingGaps: {}\n", r.blocking_gaps.len()));
    out.push_str(&format!("written: {}\n", r.written.len()));
    if !r.warnings.is_empty() {
        out.push_str("warnings:\n");
        for w in &r.warnings {
            out.push_str(&format!("  - {w}\n"));
        }
    }
    out.push_str("mode: migrate\n");
    out
}

/// Serialize migrate report as camelCase JSON (schemaVersion 1).
pub fn migrate_report_to_json(r: &MigrateReport) -> CoreResult<String> {
    serde_json::to_string_pretty(r)
        .map_err(|e| CoreError::io(format!("serialize migrate report: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn fixture_with_reverse(module_ids: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"demo\"\nedition=\"2021\"\n")
            .unwrap();
        fs::create_dir_all(dir.path().join("DARE/REVERSE")).unwrap();
        fs::write(dir.path().join("DARE/IDEIA.md"), "# IDEIA\n\nlegacy\n").unwrap();
        let modules_json: Vec<String> = module_ids
            .iter()
            .map(|id| {
                format!(
                    r#"{{"id":"{id}","path":"crates/{id}","languages":["rust"],"loc":1,"fileCount":1,"dependsOn":[]}}"#
                )
            })
            .collect();
        let facts = format!(
            r#"{{"schemaVersion":1,"projectRoot":".","stacks":["rust"],"modules":[{}],"deep":false}}"#,
            modules_json.join(",")
        );
        fs::write(dir.path().join("DARE/REVERSE/reverse-facts.json"), facts).unwrap();
        for id in module_ids {
            fs::write(
                dir.path().join(format!("DARE/REVERSE/module-{id}.md")),
                format!("# module {id}\n"),
            )
            .unwrap();
        }
        dir
    }

    fn walk_rel_files(root: &Path) -> BTreeSet<PathBuf> {
        let mut out = BTreeSet::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(cur) = stack.pop() {
            let Ok(entries) = fs::read_dir(&cur) else {
                continue;
            };
            for ent in entries.flatten() {
                let p = ent.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.is_file() {
                    if let Ok(rel) = p.strip_prefix(root) {
                        out.insert(rel.to_path_buf());
                    }
                }
            }
        }
        out
    }

    #[test]
    fn validate_migrate_target_accepts_allowlist() {
        for id in MIGRATE_TARGET_ALLOWLIST {
            validate_migrate_target(id).unwrap_or_else(|e| panic!("expected ok for {id}: {e}"));
        }
        validate_migrate_target(" node-nestjs ").expect("trim should accept");
        validate_migrate_target("  rust-axum  ").expect("trim rust-axum");
    }

    #[test]
    fn validate_migrate_target_rejects_unknown() {
        for bad in ["", "   ", "Not-A-Stack", "nestjs", "node", "RUST"] {
            let err = validate_migrate_target(bad).expect_err("expected InvalidInput");
            let msg = err.to_string();
            assert!(
                msg.contains("unknown migrate target") || msg.contains("not in allowlist"),
                "msg={msg}"
            );
            assert!(
                matches!(err, CoreError::InvalidInput(_)),
                "expected InvalidInput, got {err:?}"
            );
        }
    }

    #[test]
    fn compare_families_same_cross_unknown() {
        assert_eq!(
            compare_migration(&["rust".into()], "rust"),
            "same_family"
        );
        assert_eq!(
            compare_migration(&["node".into(), "python".into()], "node"),
            "same_family"
        );
        assert_eq!(
            compare_migration(&["rust".into()], "node"),
            "cross_stack"
        );
        assert_eq!(compare_migration(&[], "rust"), "unknown_origin");
        assert_eq!(
            target_family("node-nestjs"),
            Some("node")
        );
        assert_eq!(target_family("rust"), Some("rust"));
        assert_eq!(target_family("go-gin"), Some("go"));
        assert_eq!(target_family("rails"), Some("ruby"));
        assert_eq!(target_family("nope"), None);
    }

    #[test]
    fn phases_order_foundations_modules_cutover() {
        let modules = vec!["mod-b".into(), "mod-a".into()];
        let from = vec!["rust".into()];
        let phases = build_phases(&modules, &from, "node-nestjs");
        assert_eq!(phases.len(), 3);
        assert_eq!(phases[0].id, "foundations");
        assert_eq!(phases[1].id, "modules");
        assert_eq!(phases[2].id, "cutover");
        assert!(phases[0].modules.is_empty());
        assert_eq!(phases[1].modules, modules);
        assert!(phases[2].modules.is_empty());
        assert!(phases[0]
            .evidence
            .iter()
            .any(|e| e.contains("fromStacks") && e.contains("rust")));
        assert!(phases[0]
            .evidence
            .iter()
            .any(|e| e.contains("toStack") && e.contains("node-nestjs")));
        assert!(phases[1]
            .evidence
            .iter()
            .any(|e| e.contains("module-mod-a.md")));
    }

    #[test]
    fn blocking_gaps_sort_blocking_then_id() {
        let mut gaps = build_blocking_gaps("cross_stack", false, false, true);
        let ids: Vec<&str> = gaps.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "gap-cross-stack",
                "gap-stack-conflict",
                "gap-no-dna",
                "gap-no-patterns",
            ]
        );
        assert!(gaps.iter().all(|g| {
            if g.id.starts_with("gap-no-") {
                g.severity == "warning"
            } else {
                g.severity == "blocking"
            }
        }));

        // unknown_origin + no conflicts + artifacts present
        gaps = build_blocking_gaps("unknown_origin", true, true, false);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].id, "gap-unknown-origin");
        assert_eq!(gaps[0].severity, "blocking");

        // same_family, all present → empty
        gaps = build_blocking_gaps("same_family", true, true, false);
        assert!(gaps.is_empty());

        // sort helper alone
        let mut unsorted = vec![
            BlockingGap {
                id: "gap-no-dna".into(),
                severity: "warning".into(),
                evidence: "e".into(),
                detail: "d".into(),
            },
            BlockingGap {
                id: "gap-z".into(),
                severity: "blocking".into(),
                evidence: "e".into(),
                detail: "d".into(),
            },
            BlockingGap {
                id: "gap-a".into(),
                severity: "blocking".into(),
                evidence: "e".into(),
                detail: "d".into(),
            },
        ];
        sort_blocking_gaps(&mut unsorted);
        assert_eq!(unsorted[0].id, "gap-a");
        assert_eq!(unsorted[1].id, "gap-z");
        assert_eq!(unsorted[2].id, "gap-no-dna");
    }

    #[test]
    fn run_migrate_check_zero_write() {
        let dir = fixture_with_reverse(&["mod-a", "mod-b"]);
        let before = walk_rel_files(dir.path());
        let report = run_migrate(
            dir.path(),
            &MigrateOptions {
                to_stack: "node-nestjs".into(),
                check: true,
                ai: true, // ignored by domain
            },
        )
        .expect("check ok");
        assert_eq!(report.mode, "check");
        assert!(report.written.is_empty());
        assert_eq!(report.module_ids, vec!["mod-a", "mod-b"]);
        let after = walk_rel_files(dir.path());
        assert_eq!(before, after, "check must not mutate filesystem");
        assert!(!dir.path().join(MIGRATION_DIR).exists());
    }

    #[test]
    fn run_migrate_write_creates_md_facts_parity() {
        let dir = fixture_with_reverse(&["alpha", "beta"]);
        let report = run_migrate(
            dir.path(),
            &MigrateOptions {
                to_stack: "rust-axum".into(),
                check: false,
                ai: false,
            },
        )
        .expect("write ok");
        assert_eq!(report.mode, "write");
        assert_eq!(report.to_stack, "rust-axum");
        assert_eq!(report.to_family, "rust");
        assert_eq!(report.comparison, "same_family");
        assert!(dir.path().join(MIGRATION_MD_REL).is_file());
        assert!(dir.path().join(MIGRATION_FACTS_REL).is_file());
        assert!(dir
            .path()
            .join("DARE/MIGRATION/parity/alpha.feature")
            .is_file());
        assert!(dir
            .path()
            .join("DARE/MIGRATION/parity/beta.feature")
            .is_file());
        let mut expected = vec![
            MIGRATION_MD_REL.to_string(),
            MIGRATION_FACTS_REL.to_string(),
            "DARE/MIGRATION/parity/alpha.feature".to_string(),
            "DARE/MIGRATION/parity/beta.feature".to_string(),
        ];
        expected.sort();
        assert_eq!(report.written, expected);

        let facts_raw = fs::read_to_string(dir.path().join(MIGRATION_FACTS_REL)).unwrap();
        let parsed: MigrateReport = serde_json::from_str(&facts_raw).unwrap();
        assert_eq!(parsed.schema_version, 1);
        assert_eq!(parsed.mode, "write");

        let md = fs::read_to_string(dir.path().join(MIGRATION_MD_REL)).unwrap();
        assert!(md.contains("# Migration Plan:"));
        assert!(md.contains("<!-- AGENT:BEGIN section=\"paradigm\" -->"));
        assert!(md.contains("<!-- AGENT:END section=\"paradigm\" -->"));
        assert!(md.contains("<!-- AGENT:BEGIN section=\"strategy\" -->"));
        assert!(md.contains("<!-- AGENT:BEGIN section=\"risk-register\" -->"));
        assert!(md.contains("<!-- AGENT:BEGIN section=\"target-architecture\" -->"));
        assert!(md.contains("<!-- AGENT:BEGIN section=\"cutover-rollback\" -->"));
    }

    #[test]
    fn run_migrate_missing_ideia() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"\nedition=\"2021\"\n")
            .unwrap();
        fs::create_dir_all(dir.path().join("DARE/REVERSE")).unwrap();
        fs::write(
            dir.path().join("DARE/REVERSE/module-a.md"),
            "# a\n",
        )
        .unwrap();
        let err = run_migrate(
            dir.path(),
            &MigrateOptions {
                to_stack: "node-nestjs".into(),
                check: true,
                ai: false,
            },
        )
        .expect_err("missing IDEIA");
        let msg = err.to_string();
        assert!(
            msg.contains("missing DARE/IDEIA.md") || msg.contains("dare reverse"),
            "msg={msg}"
        );
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    #[test]
    fn run_migrate_no_modules() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"\nedition=\"2021\"\n")
            .unwrap();
        fs::create_dir_all(dir.path().join("DARE")).unwrap();
        fs::write(dir.path().join("DARE/IDEIA.md"), "# IDEIA\n").unwrap();
        // empty reverse facts / no module-*.md
        fs::create_dir_all(dir.path().join("DARE/REVERSE")).unwrap();
        fs::write(
            dir.path().join("DARE/REVERSE/reverse-facts.json"),
            r#"{"schemaVersion":1,"modules":[]}"#,
        )
        .unwrap();
        let err = run_migrate(
            dir.path(),
            &MigrateOptions {
                to_stack: "python-fastapi".into(),
                check: false,
                ai: false,
            },
        )
        .expect_err("no modules");
        let msg = err.to_string();
        assert!(msg.contains("no modules"), "msg={msg}");
        assert!(matches!(err, CoreError::InvalidInput(_)));
        assert!(!dir.path().join(MIGRATION_DIR).exists());
    }

    #[test]
    fn parity_feature_skeleton_shape() {
        let body = render_parity_feature("dare-core");
        let expected = concat!(
            "Feature: Parity for module dare-core\n",
            "  # dare:managed skeleton — fill via /dare-migrate\n",
            "  # evidence: DARE/REVERSE/module-dare-core.md\n",
            "\n",
            "  @module:dare-core @parity @skeleton\n",
            "  Scenario: Observable behavior placeholder\n",
            "    Given the legacy module \"dare-core\" is available\n",
            "    When a critical user flow of \"dare-core\" is exercised\n",
            "    Then the target stack behavior matches the legacy outcomes\n",
        );
        assert_eq!(body, expected);
    }

    #[test]
    fn format_migrate_human_contains_mode_stacks() {
        let r = MigrateReport {
            schema_version: 1,
            mode: "check".into(),
            from_stacks: vec!["rust".into()],
            to_stack: "node-nestjs".into(),
            to_family: "node".into(),
            comparison: "cross_stack".into(),
            phases: build_phases(&["a".into()], &["rust".into()], "node-nestjs"),
            blocking_gaps: Vec::new(),
            module_ids: vec!["a".into()],
            written: Vec::new(),
            warnings: Vec::new(),
        };
        let human = format_migrate_human(&r);
        assert!(human.contains(MSG_CHECK));
        assert!(human.contains("fromStacks: rust"));
        assert!(human.contains("toStack: node-nestjs"));
        assert!(human.contains("mode: migrate"));

        let json = migrate_report_to_json(&r).unwrap();
        assert!(json.contains("\"schemaVersion\": 1"));
        assert!(json.contains("\"toStack\": \"node-nestjs\""));
    }

    #[test]
    fn migration_path_jail_rejects_escape() {
        let err = migration_safe_rel("DARE/../etc/passwd").unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput(_)));
        let err2 = migration_safe_rel("DARE/REVERSE/x.md").unwrap_err();
        assert!(matches!(err2, CoreError::InvalidInput(_)));
        assert!(migration_safe_rel(MIGRATION_MD_REL).is_ok());
    }
}
