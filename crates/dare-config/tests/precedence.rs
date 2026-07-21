//! Precedence matrix P1–P5 / B1–B3 (BLUEPRINT-008 Appendix A).

use dare_config::{default_config, merge_layers, CliOverrides, EnvOverrides};
use dare_contracts::{ConfigObject, DareConfig};
use serde_json::json;
use std::collections::BTreeMap;

fn file_ide(ide: &str) -> DareConfig {
    DareConfig {
        ide: Some(ide.into()),
        ..Default::default()
    }
}

#[test]
fn p1_env_beats_file() {
    let env = EnvOverrides {
        ide: Some("claude".into()),
        ..Default::default()
    };
    let out = merge_layers(
        &default_config(),
        Some(&file_ide("cursor")),
        &env,
        &CliOverrides::default(),
    );
    assert_eq!(out.ide.as_deref(), Some("claude"));
}

#[test]
fn p2_cli_beats_file() {
    let cli = CliOverrides {
        ide: Some("windsurf".into()),
        ..Default::default()
    };
    let out = merge_layers(
        &default_config(),
        Some(&file_ide("cursor")),
        &EnvOverrides::default(),
        &cli,
    );
    assert_eq!(out.ide.as_deref(), Some("windsurf"));
}

#[test]
fn p3_cli_beats_env() {
    let env = EnvOverrides {
        ide: Some("claude".into()),
        ..Default::default()
    };
    let cli = CliOverrides {
        ide: Some("windsurf".into()),
        ..Default::default()
    };
    let out = merge_layers(
        &default_config(),
        Some(&file_ide("cursor")),
        &env,
        &cli,
    );
    assert_eq!(out.ide.as_deref(), Some("windsurf"));
}

#[test]
fn p4_all_none() {
    let out = merge_layers(
        &default_config(),
        None,
        &EnvOverrides::default(),
        &CliOverrides::default(),
    );
    assert!(out.ide.is_none());
}

#[test]
fn p5_file_only() {
    let out = merge_layers(
        &default_config(),
        Some(&file_ide("cursor")),
        &EnvOverrides::default(),
        &CliOverrides::default(),
    );
    assert_eq!(out.ide.as_deref(), Some("cursor"));
}

#[test]
fn b1_env_enables_guard() {
    let file = DareConfig {
        guard: Some(ConfigObject {
            enabled: Some(false),
            extra: Default::default(),
        }),
        ..Default::default()
    };
    let mut blocks = BTreeMap::new();
    blocks.insert("guard".into(), true);
    let env = EnvOverrides {
        block_enabled: blocks,
        ..Default::default()
    };
    let out = merge_layers(
        &default_config(),
        Some(&file),
        &env,
        &CliOverrides::default(),
    );
    assert_eq!(out.guard.as_ref().and_then(|g| g.enabled), Some(true));
}

#[test]
fn b2_cli_beats_env_on_guard() {
    let file = DareConfig {
        guard: Some(ConfigObject {
            enabled: Some(true),
            extra: Default::default(),
        }),
        ..Default::default()
    };
    let mut env_blocks = BTreeMap::new();
    env_blocks.insert("guard".into(), false);
    let env = EnvOverrides {
        block_enabled: env_blocks,
        ..Default::default()
    };
    let mut cli_blocks = BTreeMap::new();
    cli_blocks.insert("guard".into(), true);
    let cli = CliOverrides {
        block_enabled: cli_blocks,
        ..Default::default()
    };
    let out = merge_layers(&default_config(), Some(&file), &env, &cli);
    assert_eq!(out.guard.as_ref().and_then(|g| g.enabled), Some(true));
}

#[test]
fn preserves_file_extras() {
    let mut file = file_ide("cursor");
    file.extra.insert("customExtension".into(), json!({"x": 1}));
    let out = merge_layers(
        &default_config(),
        Some(&file),
        &EnvOverrides::default(),
        &CliOverrides::default(),
    );
    assert!(out.extra.contains_key("customExtension"));
}
