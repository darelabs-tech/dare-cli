//! Human and JSON formatting for [`UpdatePlan`].

use dare_core::CoreResult;
use serde_json::Value;

use crate::classify::AssetUpdateStatus;
use crate::plan::UpdatePlan;

fn status_tag(status: AssetUpdateStatus) -> &'static str {
    match status {
        AssetUpdateStatus::Identical => "identical",
        AssetUpdateStatus::Missing => "missing",
        AssetUpdateStatus::Apply => "apply",
        AssetUpdateStatus::Customized => "customized",
    }
}

/// Render a human-readable dry-run plan (en-US; no file bodies).
pub fn format_human(plan: &UpdatePlan) -> String {
    let target = plan.target.as_deref().unwrap_or("(all)");
    let mut out = String::new();
    out.push_str("update: dry-run\n");
    out.push_str(&format!("cliVersion: {}\n", plan.cli_version));
    out.push_str(&format!("target: {target}\n"));
    out.push_str(&format!("projectRoot: {}\n", plan.project_root));
    out.push_str(&format!(
        "counts: identical={} missing={} apply={} customized={}\n",
        plan.counts.identical, plan.counts.missing, plan.counts.apply, plan.counts.customized
    ));
    out.push_str("items:\n");
    for item in &plan.items {
        out.push_str(&format!(
            "  - [{}] {}\n",
            status_tag(item.status),
            item.path
        ));
    }
    if plan.counts.customized > 0 {
        out.push_str("customized:\n");
        for item in &plan.items {
            if item.status == AssetUpdateStatus::Customized {
                out.push_str(&format!("  - {} (sha mismatch, unmanaged)\n", item.path));
            }
        }
    }
    out.push_str("mode: dry-run (zero mutations)\n");
    out
}

/// Serialize an [`UpdatePlan`] to a JSON [`Value`] (camelCase fields).
pub fn plan_to_json(plan: &UpdatePlan) -> CoreResult<Value> {
    serde_json::to_value(plan)
        .map_err(|e| dare_core::CoreError::config(format!("failed to serialize UpdatePlan: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{UpdateCounts, UpdateItem, MODE_DRY_RUN};
    use crate::UPDATE_PLAN_SCHEMA_VERSION;

    #[test]
    fn format_human_includes_zero_mutations_and_customized_section() {
        let plan = UpdatePlan {
            schema_version: UPDATE_PLAN_SCHEMA_VERSION,
            mode: MODE_DRY_RUN.into(),
            project_root: "/tmp/proj".into(),
            target: None,
            cli_version: "0.1.0-alpha.0".into(),
            counts: UpdateCounts {
                identical: 0,
                missing: 0,
                apply: 0,
                customized: 1,
            },
            items: vec![UpdateItem {
                path: "CLAUDE.md".into(),
                status: AssetUpdateStatus::Customized,
                expected_sha256: "a".repeat(64),
                actual_sha256: Some("b".repeat(64)),
                applies_to: vec!["claude-code".into()],
            }],
        };
        let text = format_human(&plan);
        assert!(text.contains("mode: dry-run (zero mutations)"));
        assert!(text.contains("customized:"));
        assert!(text.contains("CLAUDE.md (sha mismatch, unmanaged)"));
        assert!(text.contains("target: (all)"));
        assert!(!text.contains("aaaaaaaa")); // no hash dump in human
    }

    #[test]
    fn plan_to_json_camel_case() {
        let plan = UpdatePlan {
            schema_version: UPDATE_PLAN_SCHEMA_VERSION,
            mode: MODE_DRY_RUN.into(),
            project_root: "/tmp/proj".into(),
            target: Some("codex".into()),
            cli_version: "0.1.0-alpha.0".into(),
            counts: UpdateCounts::default(),
            items: vec![],
        };
        let v = plan_to_json(&plan).unwrap();
        assert_eq!(v["schemaVersion"], 1);
        assert_eq!(v["mode"], "dry-run");
        assert_eq!(v["projectRoot"], "/tmp/proj");
        assert_eq!(v["cliVersion"], "0.1.0-alpha.0");
        assert!(v.get("schema_version").is_none());
    }
}
