//! Migration plan domain — types, allowlist, compare, phases & gaps (microplano 039).

use dare_core::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

/// Frozen JSON schema version for migrate reports/facts.
pub const MIGRATE_SCHEMA_VERSION: u32 = 1;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
