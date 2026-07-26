//! Frontend composition integration tests (mp047-002).

use dare_core::ProjectRoot;
use dare_scaffold::{
    apply_scaffold, plan_scaffold, render_template, FrontendKind, PlanAction, PlanItemKind,
    ScaffoldRequest, Toolchain,
};
use std::fs;
use tempfile::tempdir;

fn frontend_req(stack_id: &str, frontend: FrontendKind) -> ScaffoldRequest {
    ScaffoldRequest {
        project_name: "demo-app".to_string(),
        stack_id: stack_id.to_string(),
        toolchain: Toolchain::None,
        transport: None,
        frontend: Some(frontend),
        conflict_policy: dare_scaffold::ConflictPolicy::FailFast,
        force: false,
        check: false,
    }
}

#[test]
fn frontend_compose_react_paths() {
    let dir = tempdir().expect("tempdir");
    let root = ProjectRoot::new(dir.path()).expect("project root");

    let plan = plan_scaffold(&root, &frontend_req("rust-axum", FrontendKind::React))
        .expect("plan with react frontend");

    assert_eq!(plan.frontend, Some(FrontendKind::React));

    let paths: Vec<_> = plan.items.iter().map(|i| i.path.as_str()).collect();
    assert!(paths.contains(&"frontend/package.json"));
    assert!(paths.contains(&"frontend/src/main.tsx"));
    assert!(paths.contains(&"frontend/README.md"));

    for path in ["frontend/package.json", "frontend/src/main.tsx", "frontend/README.md"] {
        let item = plan.items.iter().find(|i| i.path == path).expect("plan item");
        assert_eq!(item.kind, PlanItemKind::Template);
        assert_eq!(item.action, PlanAction::Create);
    }

    let report = apply_scaffold(&root, &plan).expect("apply frontend");
    assert!(report.created.contains(&"frontend/package.json".to_string()));
    assert!(report.created.contains(&"frontend/src/main.tsx".to_string()));
    assert!(report.created.contains(&"frontend/README.md".to_string()));

    let pkg = fs::read_to_string(dir.path().join("frontend/package.json")).expect("package.json");
    assert!(pkg.contains("\"name\": \"demo-app-web\""));
    assert!(pkg.contains("\"private\": true"));

    let readme = fs::read_to_string(dir.path().join("frontend/README.md")).expect("readme");
    assert!(readme.contains("rust-axum"));
}

#[test]
fn frontend_compose_vue_paths() {
    let dir = tempdir().expect("tempdir");
    let root = ProjectRoot::new(dir.path()).expect("project root");

    let plan = plan_scaffold(&root, &frontend_req("go-gin", FrontendKind::Vue))
        .expect("plan with vue frontend");

    let paths: Vec<_> = plan.items.iter().map(|i| i.path.as_str()).collect();
    assert!(paths.contains(&"frontend/package.json"));
    assert!(paths.contains(&"frontend/src/main.ts"));
    assert!(paths.contains(&"frontend/README.md"));
}

#[test]
fn frontend_secret_rejected() {
    let tpl = r#"{"name":"demo","secret":"api_key=bad"}"#;
    let err = render_template(tpl, "demo-app", "rust-axum").expect_err("secret scan");
    assert!(
        err.to_string().contains("forbidden secret pattern"),
        "expected secret rejection, got {err}"
    );
}
