//! Integration round-trips under ProjectRoot.

use dare_contracts::{
    load_dag, load_dare_config, load_graph, load_runtime_state, load_skills_manifest,
    load_update_manifest, load_verification_baseline, parse_dag_yaml, save_dag, save_dare_config,
    telemetry_snapshot_from_str, DagDocument,
};
use dare_core::{ProjectRoot, SafeRelativePath};
use std::fs;
use tempfile::tempdir;

fn copy_fixture(name: &str, dest_dir: &std::path::Path, dest_name: &str) {
    let src = format!("tests/fixtures/{name}");
    // cargo test cwd is crate root
    fs::copy(&src, dest_dir.join(dest_name)).unwrap_or_else(|e| {
        panic!("copy {src}: {e}");
    });
}

#[test]
fn fixtures_parse_and_roundtrip() {
    let dir = tempdir().unwrap();
    let root = ProjectRoot::new(dir.path()).unwrap();

    copy_fixture("dare.config.json", dir.path(), "dare.config.json");
    let cfg_rel = SafeRelativePath::new("dare.config.json").unwrap();
    let cfg = load_dare_config(&root, &cfg_rel).unwrap();
    assert!(cfg.extra.contains_key("customExtension"));
    save_dare_config(&root, &cfg_rel, &cfg).unwrap();
    let cfg2 = load_dare_config(&root, &cfg_rel).unwrap();
    assert_eq!(cfg.extra, cfg2.extra);

    let v21 = fs::read_to_string("tests/fixtures/dare-dag.v21.yaml").unwrap();
    match parse_dag_yaml(&v21).unwrap() {
        DagDocument::V21(d) => assert_eq!(d.tasks[0].id, "task-001"),
        _ => panic!("v21"),
    }
    let legacy = fs::read_to_string("tests/fixtures/dare-dag.legacy.yaml").unwrap();
    match parse_dag_yaml(&legacy).unwrap() {
        DagDocument::Legacy(d) => assert!(d.tasks.contains_key("task-001")),
        _ => panic!("legacy"),
    }

    copy_fixture("dare-dag.v21.yaml", dir.path(), "dag.yaml");
    let dag_rel = SafeRelativePath::new("dag.yaml").unwrap();
    let doc = load_dag(&root, &dag_rel).unwrap();
    save_dag(&root, &dag_rel, &doc).unwrap();
    let _ = load_dag(&root, &dag_rel).unwrap();

    copy_fixture("state.v1.json", dir.path(), "state.json");
    let st = load_runtime_state(&root, &SafeRelativePath::new("state.json").unwrap()).unwrap();
    assert_eq!(st.version, 1);

    copy_fixture("dare-graph.yml", dir.path(), "dare-graph.yml");
    let g = load_graph(&root, &SafeRelativePath::new("dare-graph.yml").unwrap()).unwrap();
    assert!(!g.nodes.is_empty());

    copy_fixture("skills.yml", dir.path(), "skills.yml");
    let sk = load_skills_manifest(&root, &SafeRelativePath::new("skills.yml").unwrap()).unwrap();
    assert_eq!(sk.skills[0].id, "dare-ax");

    copy_fixture("verification.task.json", dir.path(), "ver.json");
    let vb =
        load_verification_baseline(&root, &SafeRelativePath::new("ver.json").unwrap()).unwrap();
    assert_eq!(vb.task_id.as_deref(), Some("task-001"));

    copy_fixture("UPDATE-MANIFEST.json", dir.path(), "um.json");
    let um = load_update_manifest(&root, &SafeRelativePath::new("um.json").unwrap()).unwrap();
    assert_eq!(um.schema_version, 1);

    let tel = fs::read_to_string("tests/fixtures/telemetry.snapshot.json").unwrap();
    let snap = telemetry_snapshot_from_str(&tel).unwrap();
    assert!(snap.dag.contains_key("done"));
}
