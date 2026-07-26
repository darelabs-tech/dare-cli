//! Scaffold planning (BLUEPRINT-046 §0.5 / mp046-004, mp047-002).

use std::collections::BTreeMap;

use dare_assets::EmbeddedAssets;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};

use crate::ax::ax_artifact_paths;
use crate::registry::scaffolder_for;
use crate::types::{
    ConflictPolicy, FrontendKind, PlanAction, PlanItemKind, ScaffoldPlan, ScaffoldPlanItem,
    ScaffoldRequest, SCHEMA_VERSION,
};

pub const PROJECT_NAME_RE: &str = r"^[a-z][a-z0-9_-]{0,63}$";
const MSG_INVALID_NAME: &str = "invalid project name";

/// Validate `project_name` against [`PROJECT_NAME_RE`].
pub fn validate_project_name(name: &str) -> CoreResult<()> {
    if name.is_empty() || name.len() > 64 {
        return Err(CoreError::InvalidInput(MSG_INVALID_NAME.to_string()));
    }
    let mut chars = name.chars();
    let first = chars.next().expect("non-empty");
    if !first.is_ascii_lowercase() {
        return Err(CoreError::InvalidInput(MSG_INVALID_NAME.to_string()));
    }
    for c in chars {
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-') {
            return Err(CoreError::InvalidInput(MSG_INVALID_NAME.to_string()));
        }
    }
    Ok(())
}

fn path_exists(root: &ProjectRoot, path: &str) -> CoreResult<bool> {
    let rel = SafeRelativePath::new(path)?;
    let abs = root.resolve(&rel)?;
    Ok(abs.as_path().exists())
}

fn template_kind(dest: &str) -> PlanItemKind {
    if dest == "dare.config.json" {
        PlanItemKind::Meta
    } else {
        PlanItemKind::Template
    }
}

fn asset_dest_path(stack_prefix: &str, asset_path: &str) -> Option<String> {
    let rel = asset_path.strip_prefix(stack_prefix)?;
    if rel.is_empty() {
        return None;
    }
    if rel.ends_with(".tpl") {
        Some(rel.strip_suffix(".tpl")?.to_string())
    } else {
        Some(rel.to_string())
    }
}

fn collect_embedded_paths(stack_id: &str) -> CoreResult<BTreeMap<String, PlanItemKind>> {
    let stack_prefix = format!("stacks/{stack_id}/");
    let mut map = BTreeMap::new();
    for asset_path in EmbeddedAssets::iter() {
        let asset_path = asset_path.as_ref();
        if asset_path == "stacks/README.md" {
            continue;
        }
        let Some(dest) = asset_dest_path(&stack_prefix, asset_path) else {
            continue;
        };
        let kind = template_kind(&dest);
        map.insert(dest, kind);
    }
    if map.is_empty() {
        return Err(CoreError::Internal(format!(
            "no embedded template assets for stack `{stack_id}`"
        )));
    }
    Ok(map)
}

fn frontend_asset_id(kind: FrontendKind) -> &'static str {
    match kind {
        FrontendKind::React => "react",
        FrontendKind::Vue => "vue",
    }
}

fn collect_frontend_paths(frontend: FrontendKind) -> CoreResult<BTreeMap<String, PlanItemKind>> {
    let fe_id = frontend_asset_id(frontend);
    let stack_prefix = format!("stacks/_frontend/{fe_id}/");
    let mut map = BTreeMap::new();
    for asset_path in EmbeddedAssets::iter() {
        let asset_path = asset_path.as_ref();
        let Some(rel) = asset_dest_path(&stack_prefix, asset_path) else {
            continue;
        };
        let dest = format!("frontend/{rel}");
        map.insert(dest, PlanItemKind::Template);
    }
    if map.is_empty() {
        return Err(CoreError::Internal(format!(
            "no embedded template assets for frontend `{fe_id}`"
        )));
    }
    Ok(map)
}

fn resolve_action(
    path: &str,
    exists: bool,
    force: bool,
    policy: ConflictPolicy,
) -> CoreResult<PlanAction> {
    if exists {
        if force {
            Ok(PlanAction::Replace)
        } else if policy == ConflictPolicy::SkipExisting {
            Ok(PlanAction::Skip)
        } else {
            Err(CoreError::InvalidInput(format!(
                "path already exists: {path}"
            )))
        }
    } else {
        Ok(PlanAction::Create)
    }
}

/// Build a sorted scaffold plan for `req` under `root`.
pub fn plan_scaffold(root: &ProjectRoot, req: &ScaffoldRequest) -> CoreResult<ScaffoldPlan> {
    validate_project_name(&req.project_name)?;
    let scaffolder = scaffolder_for(&req.stack_id)?;

    let meta = scaffolder.metadata();
    let mut paths = collect_embedded_paths(&req.stack_id)?;

    if let Some(frontend) = req.frontend {
        paths.extend(collect_frontend_paths(frontend)?);
    }

    for ax_path in ax_artifact_paths(meta) {
        paths.insert(ax_path, PlanItemKind::Ax);
    }

    let mut items = Vec::with_capacity(paths.len());
    for (path, kind) in paths {
        let exists = path_exists(root, &path)?;
        let action = resolve_action(&path, exists, req.force, req.conflict_policy)?;
        items.push(ScaffoldPlanItem { path, action, kind });
    }

    items.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(ScaffoldPlan {
        schema_version: SCHEMA_VERSION,
        stack_id: req.stack_id.clone(),
        project_name: req.project_name.clone(),
        frontend: req.frontend,
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FrontendKind, Toolchain};
    use dare_core::CoreError;
    use tempfile::tempdir;

    fn sample_req(stack_id: &str) -> ScaffoldRequest {
        ScaffoldRequest {
            project_name: "demo-app".to_string(),
            stack_id: stack_id.to_string(),
            toolchain: Toolchain::None,
            transport: None,
            frontend: None,
            conflict_policy: ConflictPolicy::FailFast,
            force: false,
            check: false,
        }
    }

    #[test]
    fn plan_sorted() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let plan = plan_scaffold(&root, &sample_req("rust-axum")).expect("plan");

        let paths: Vec<_> = plan.items.iter().map(|i| i.path.as_str()).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted, "items must be sorted path ASC");

        let kinds: Vec<_> = plan.items.iter().map(|i| i.kind).collect();
        assert!(
            kinds.contains(&PlanItemKind::Template),
            "expected at least one template item"
        );
        assert!(
            kinds.contains(&PlanItemKind::Ax),
            "expected AX items"
        );
        assert!(
            kinds.contains(&PlanItemKind::Meta),
            "expected dare.config.json meta item"
        );
        assert!(!plan.items.is_empty());
    }

    #[test]
    fn plan_rejects_invalid_name() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let mut req = sample_req("go-gin");
        req.project_name = "Bad-Name".to_string();
        let err = plan_scaffold(&root, &req).unwrap_err();
        match err {
            CoreError::InvalidInput(msg) => assert_eq!(msg, MSG_INVALID_NAME),
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn plan_fail_fast_path_exists() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("dare.config.json").unwrap();
        dare_core::fs::atomic_write(&root, &rel, b"{}").unwrap();

        let err = plan_scaffold(&root, &sample_req("go-gin")).unwrap_err();
        match err {
            CoreError::InvalidInput(msg) => {
                assert_eq!(msg, "path already exists: dare.config.json");
            }
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn plan_skip_existing_path_exists() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("dare.config.json").unwrap();
        dare_core::fs::atomic_write(&root, &rel, b"{}").unwrap();

        let mut req = sample_req("go-gin");
        req.conflict_policy = ConflictPolicy::SkipExisting;
        let plan = plan_scaffold(&root, &req).expect("plan with skip");
        let item = plan
            .items
            .iter()
            .find(|i| i.path == "dare.config.json")
            .expect("dare.config.json in plan");
        assert_eq!(item.action, PlanAction::Skip);
    }

    #[test]
    fn plan_includes_frontend_react() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let mut req = sample_req("rust-axum");
        req.frontend = Some(FrontendKind::React);
        let plan = plan_scaffold(&root, &req).expect("plan with frontend");

        assert_eq!(plan.frontend, Some(FrontendKind::React));
        let paths: Vec<_> = plan.items.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains(&"frontend/package.json"));
        assert!(paths.contains(&"frontend/src/main.tsx"));
        assert!(paths.contains(&"frontend/README.md"));
    }
}
