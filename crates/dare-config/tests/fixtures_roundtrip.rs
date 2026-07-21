//! Golden fixtures round-trip (microplano 008 / RF-17).

use dare_config::{
    default_config, load_effective, merge_layers, validate, CliOverrides, EnvOverrides,
    DEFAULT_CONFIG_REL,
};
use dare_contracts::load_dare_config;
use dare_core::{ProjectRoot, SafeRelativePath};
use std::path::PathBuf;
use tempfile::tempdir;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn stage_fixture(name: &str) -> (tempfile::TempDir, ProjectRoot, SafeRelativePath) {
    let dir = tempdir().expect("tempdir");
    let src = fixture_path(name);
    let dest = dir.path().join(DEFAULT_CONFIG_REL);
    std::fs::copy(&src, &dest).unwrap_or_else(|e| {
        panic!("copy {} -> {}: {e}", src.display(), dest.display());
    });
    let root = ProjectRoot::new(dir.path()).expect("ProjectRoot");
    let rel = SafeRelativePath::new(DEFAULT_CONFIG_REL).expect("rel");
    (dir, root, rel)
}

#[test]
fn legacy_fixture_preserves_unknown_and_guard_off() {
    let (_dir, root, rel) = stage_fixture("legacy.config.json");
    let cfg = load_dare_config(&root, &rel).expect("load");
    assert_eq!(cfg.ide.as_deref(), Some("cursor"));
    assert_eq!(cfg.guard.as_ref().and_then(|g| g.enabled), Some(false));
    assert!(
        cfg.extra.contains_key("customExtension"),
        "unknown root key must survive: {:?}",
        cfg.extra
    );
    validate(&cfg).expect("validate");
    let merged = merge_layers(
        &default_config(),
        Some(&cfg),
        &EnvOverrides::default(),
        &CliOverrides::default(),
    );
    assert!(merged.extra.contains_key("customExtension"));
}

#[test]
fn with_extras_fixture_preserves_unknown_root_and_nested() {
    let (_dir, root, rel) = stage_fixture("with_extras.config.json");
    let cfg = load_effective(
        &root,
        &rel,
        &EnvOverrides::default(),
        &CliOverrides::default(),
    )
    .expect("load_effective");
    assert!(cfg.extra.contains_key("unknownRoot"));
    assert_eq!(
        cfg.agent
            .as_ref()
            .and_then(|a| a.extra.get("custom"))
            .and_then(|v| v.as_str()),
        Some("a")
    );
    let merged = merge_layers(
        &default_config(),
        Some(&cfg),
        &EnvOverrides::default(),
        &CliOverrides::default(),
    );
    assert!(merged.extra.contains_key("unknownRoot"));
}

#[test]
fn enabled_false_fixture_validates_without_deep_fail() {
    let (_dir, root, rel) = stage_fixture("enabled_false.config.json");
    let cfg = load_dare_config(&root, &rel).expect("load");
    assert_eq!(cfg.guard.as_ref().and_then(|g| g.enabled), Some(false));
    assert_eq!(cfg.graph.as_ref().and_then(|g| g.enabled), Some(false));
    validate(&cfg).expect("enabled:false must skip deep validation");
}
