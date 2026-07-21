//! Update planning: filter, classify, sort, and emit an `UpdatePlan` (dry-run).

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};

use crate::classify::{classify_path_detailed, AssetUpdateStatus};
use crate::manifest_v2::UpdateManifestV2;
use crate::UPDATE_PLAN_SCHEMA_VERSION;

/// Dry-run mode string frozen in UpdatePlan schema 1.
pub const MODE_DRY_RUN: &str = "dry-run";

/// Accepted `--target` harness ids (echo of `UPDATE_HARNESS_IDES`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarnessTarget {
    ClaudeCode,
    Cursor,
    Codex,
    Antigravity,
    Hybrid,
    ClaudeHybrid,
}

impl HarnessTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            HarnessTarget::ClaudeCode => "claude-code",
            HarnessTarget::Cursor => "cursor",
            HarnessTarget::Codex => "codex",
            HarnessTarget::Antigravity => "antigravity",
            HarnessTarget::Hybrid => "hybrid",
            HarnessTarget::ClaudeHybrid => "claude-hybrid",
        }
    }

    /// Atomic harness ids used to filter `appliesTo`.
    pub fn expanded_ids(self) -> &'static [&'static str] {
        match self {
            HarnessTarget::ClaudeCode => &["claude-code"],
            HarnessTarget::Cursor => &["cursor"],
            HarnessTarget::Codex => &["codex"],
            HarnessTarget::Antigravity => &["antigravity"],
            HarnessTarget::Hybrid => &["cursor", "antigravity"],
            HarnessTarget::ClaudeHybrid => &["claude-code", "cursor"],
        }
    }
}

/// Parse CLI `--target` into a harness target (not a semver).
pub fn parse_harness_target(s: &str) -> CoreResult<HarnessTarget> {
    match s {
        "claude-code" => Ok(HarnessTarget::ClaudeCode),
        "cursor" => Ok(HarnessTarget::Cursor),
        "codex" => Ok(HarnessTarget::Codex),
        "antigravity" => Ok(HarnessTarget::Antigravity),
        "hybrid" => Ok(HarnessTarget::Hybrid),
        "claude-hybrid" => Ok(HarnessTarget::ClaudeHybrid),
        other => Err(CoreError::invalid_input(format!(
            "invalid --target harness: {other}"
        ))),
    }
}

/// Options passed into `plan_update` (CLI supplies `cli_version`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatePlanOptions {
    pub target: Option<HarnessTarget>,
    pub cli_version: String,
}

/// One classified asset in an update plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateItem {
    pub path: String,
    pub status: AssetUpdateStatus,
    pub expected_sha256: String,
    pub actual_sha256: Option<String>,
    pub applies_to: Vec<String>,
}

/// Aggregated status counts (must match `items`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCounts {
    pub identical: u32,
    pub missing: u32,
    pub apply: u32,
    pub customized: u32,
}

/// Deterministic dry-run update plan (schemaVersion 1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlan {
    pub schema_version: u32,
    pub mode: String,
    pub project_root: String,
    pub target: Option<String>,
    pub cli_version: String,
    pub counts: UpdateCounts,
    pub items: Vec<UpdateItem>,
}

fn item_matches(applies_to: &[String], target: Option<HarnessTarget>) -> bool {
    if applies_to.iter().any(|a| a == "*") {
        return true;
    }
    let Some(t) = target else {
        return true;
    };
    let ids = t.expanded_ids();
    applies_to.iter().any(|a| ids.contains(&a.as_str()))
}

fn count_statuses(items: &[UpdateItem]) -> UpdateCounts {
    let mut counts = UpdateCounts::default();
    for item in items {
        match item.status {
            AssetUpdateStatus::Identical => counts.identical += 1,
            AssetUpdateStatus::Missing => counts.missing += 1,
            AssetUpdateStatus::Apply => counts.apply += 1,
            AssetUpdateStatus::Customized => counts.customized += 1,
        }
    }
    counts
}

/// Build a dry-run [`UpdatePlan`] for `root` against a validated V2 manifest.
///
/// Performs zero filesystem writes.
pub fn plan_update(
    root: &ProjectRoot,
    manifest: &UpdateManifestV2,
    opts: &UpdatePlanOptions,
) -> CoreResult<UpdatePlan> {
    let mut items = Vec::new();

    for asset in &manifest.assets {
        if !item_matches(&asset.applies_to, opts.target) {
            continue;
        }
        let rel = SafeRelativePath::new(&asset.path)?;
        let (status, actual_sha256) = classify_path_detailed(root, &rel, &asset.sha256)?;
        items.push(UpdateItem {
            path: asset.path.clone(),
            status,
            expected_sha256: asset.sha256.clone(),
            actual_sha256,
            applies_to: asset.applies_to.clone(),
        });
    }

    items.sort_by(|a, b| a.path.cmp(&b.path));
    let counts = count_statuses(&items);

    Ok(UpdatePlan {
        schema_version: UPDATE_PLAN_SCHEMA_VERSION,
        mode: MODE_DRY_RUN.to_string(),
        project_root: root.to_posix(),
        target: opts.target.map(|t| t.as_str().to_string()),
        cli_version: opts.cli_version.clone(),
        counts,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_desired_manifest_v2_from_str;
    use dare_core::fs::atomic_write;
    use dare_core::ErrorKind;
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn valid_sha() -> String {
        "a".repeat(64)
    }

    fn tiny_manifest_unsorted() -> UpdateManifestV2 {
        let raw = format!(
            r#"{{
              "schemaVersion": 2,
              "cliVersion": "0.1.0-alpha.0",
              "releases": [{{"version":"0.1.0-alpha.0","notes":""}}],
              "assets": [
                {{"path":"z-last.md","sha256":"{}","appliesTo":["*"]}},
                {{"path":"AGENTS.md","sha256":"{}","appliesTo":["codex"]}},
                {{"path":"a-first.md","sha256":"{}","appliesTo":["*"]}}
              ]
            }}"#,
            valid_sha(),
            valid_sha(),
            valid_sha()
        );
        load_desired_manifest_v2_from_str(&raw).unwrap()
    }

    fn harness_coverage_manifest() -> UpdateManifestV2 {
        let raw = format!(
            r#"{{
              "schemaVersion": 2,
              "cliVersion": "0.1.0-alpha.0",
              "releases": [{{"version":"0.1.0-alpha.0","notes":""}}],
              "assets": [
                {{"path":"AGENTS.md","sha256":"{}","appliesTo":["codex"]}},
                {{"path":"CLAUDE.md","sha256":"{}","appliesTo":["claude-code"]}},
                {{"path":".cursorrules","sha256":"{}","appliesTo":["cursor"]}},
                {{"path":".antigravityrules","sha256":"{}","appliesTo":["antigravity"]}},
                {{"path":"templates/common.md","sha256":"{}","appliesTo":["*"]}}
              ]
            }}"#,
            valid_sha(),
            valid_sha(),
            valid_sha(),
            valid_sha(),
            valid_sha()
        );
        load_desired_manifest_v2_from_str(&raw).unwrap()
    }

    fn list_rel_paths(dir: &Path) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        fn walk(base: &Path, cur: &Path, out: &mut BTreeSet<String>) {
            let Ok(entries) = fs::read_dir(cur) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(base, &path, out);
                } else if let Ok(rel) = path.strip_prefix(base) {
                    out.insert(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        walk(dir, dir, &mut out);
        out
    }

    fn opts(target: Option<HarnessTarget>) -> UpdatePlanOptions {
        UpdatePlanOptions {
            target,
            cli_version: "0.1.0-alpha.0".into(),
        }
    }

    #[test]
    fn parse_target_rejects_semver() {
        let err = parse_harness_target("3.2.0").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(
            err.message().contains("invalid --target harness:"),
            "{}",
            err.message()
        );
    }

    #[test]
    fn expanded_ids_hybrids() {
        assert_eq!(
            HarnessTarget::Hybrid.expanded_ids(),
            &["cursor", "antigravity"]
        );
        assert_eq!(
            HarnessTarget::ClaudeHybrid.expanded_ids(),
            &["claude-code", "cursor"]
        );
        assert_eq!(HarnessTarget::Codex.expanded_ids(), &["codex"]);
    }

    #[test]
    fn plan_sorts_by_path() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let plan = plan_update(&root, &tiny_manifest_unsorted(), &opts(None)).unwrap();
        let paths: Vec<_> = plan.items.iter().map(|i| i.path.as_str()).collect();
        assert_eq!(paths, vec!["AGENTS.md", "a-first.md", "z-last.md"]);
        assert_eq!(plan.mode, MODE_DRY_RUN);
        assert_eq!(plan.schema_version, UPDATE_PLAN_SCHEMA_VERSION);
    }

    #[test]
    fn plan_filter_target_codex() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let plan = plan_update(
            &root,
            &harness_coverage_manifest(),
            &opts(Some(HarnessTarget::Codex)),
        )
        .unwrap();
        assert!(!plan.items.is_empty());
        for item in &plan.items {
            let ok = item.applies_to.iter().any(|a| a == "*" || a == "codex");
            assert!(
                ok,
                "unexpected item {} appliesTo={:?}",
                item.path, item.applies_to
            );
        }
        assert_eq!(plan.target.as_deref(), Some("codex"));
    }

    #[test]
    fn plan_includes_codex_paths() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let plan = plan_update(
            &root,
            &harness_coverage_manifest(),
            &opts(Some(HarnessTarget::Codex)),
        )
        .unwrap();
        assert!(
            plan.items.iter().any(|i| i.path == "AGENTS.md"),
            "AGENTS.md must be present for codex target"
        );
    }

    #[test]
    fn plan_no_target_includes_all_harnesses() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let plan = plan_update(&root, &harness_coverage_manifest(), &opts(None)).unwrap();
        let paths: BTreeSet<_> = plan.items.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains("AGENTS.md"), "codex");
        assert!(paths.contains("CLAUDE.md"), "claude-code");
        assert!(paths.contains(".cursorrules"), "cursor");
        assert!(paths.contains(".antigravityrules"), "antigravity");
        assert!(plan.target.is_none());
    }

    #[test]
    fn plan_zero_writes() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("DARE")).unwrap();
        fs::write(dir.path().join("dare.config.json"), r#"{"version":1}"#).unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("CLAUDE.md").unwrap();
        atomic_write(&root, &rel, b"<!-- dare:managed -->\nx\n").unwrap();

        let before = list_rel_paths(dir.path());
        let _plan = plan_update(&root, &harness_coverage_manifest(), &opts(None)).unwrap();
        let after = list_rel_paths(dir.path());
        assert_eq!(before, after);
    }

    #[test]
    fn plan_counts_coherent() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        // identical
        let body = b"same";
        let sha = dare_assets::sha256_hex(body);
        let raw = format!(
            r#"{{
              "schemaVersion": 2,
              "cliVersion": "0.1.0-alpha.0",
              "releases": [{{"version":"0.1.0-alpha.0","notes":""}}],
              "assets": [
                {{"path":"AGENTS.md","sha256":"{}","appliesTo":["codex"]}},
                {{"path":"identical.md","sha256":"{}","appliesTo":["*"]}},
                {{"path":"managed.md","sha256":"{}","appliesTo":["*"]}},
                {{"path":"custom.md","sha256":"{}","appliesTo":["*"]}}
              ]
            }}"#,
            valid_sha(),
            sha,
            valid_sha(),
            valid_sha()
        );
        let manifest = load_desired_manifest_v2_from_str(&raw).unwrap();
        atomic_write(&root, &SafeRelativePath::new("identical.md").unwrap(), body).unwrap();
        atomic_write(
            &root,
            &SafeRelativePath::new("managed.md").unwrap(),
            b"<!-- dare:managed -->\nstale\n",
        )
        .unwrap();
        atomic_write(
            &root,
            &SafeRelativePath::new("custom.md").unwrap(),
            b"# customized\n",
        )
        .unwrap();

        let plan = plan_update(&root, &manifest, &opts(None)).unwrap();
        let sum = plan.counts.identical
            + plan.counts.missing
            + plan.counts.apply
            + plan.counts.customized;
        assert_eq!(sum as usize, plan.items.len());
        assert_eq!(plan.counts.identical, 1);
        assert_eq!(plan.counts.missing, 1);
        assert_eq!(plan.counts.apply, 1);
        assert_eq!(plan.counts.customized, 1);
        assert_eq!(count_statuses(&plan.items), plan.counts);
    }
}
