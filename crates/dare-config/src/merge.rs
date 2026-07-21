//! Deep merge CLI > env > file > defaults.

use dare_contracts::{ConfigObject, DareConfig};
use serde_json::{Map, Value};

use crate::r#override::{CliOverrides, EnvOverrides};

fn merge_extras(base: &Map<String, Value>, overlay: &Map<String, Value>) -> Map<String, Value> {
    let mut out = base.clone();
    for (k, v) in overlay {
        out.insert(k.clone(), v.clone());
    }
    out
}

fn merge_block(base: Option<&ConfigObject>, overlay: Option<&ConfigObject>) -> Option<ConfigObject> {
    match (base, overlay) {
        (None, None) => None,
        (Some(b), None) => Some(b.clone()),
        (None, Some(o)) => Some(o.clone()),
        (Some(b), Some(o)) => Some(ConfigObject {
            enabled: o.enabled.or(b.enabled),
            extra: merge_extras(&b.extra, &o.extra),
        }),
    }
}

fn apply_block_enabled(cfg: &mut DareConfig, block: &str, enabled: bool) {
    let target = match block {
        "project" => &mut cfg.project,
        "agent" => &mut cfg.agent,
        "guard" => &mut cfg.guard,
        "graph" => &mut cfg.graph,
        "hooks" => &mut cfg.hooks,
        _ => return,
    };
    let mut obj = target.clone().unwrap_or_default();
    obj.enabled = Some(enabled);
    *target = Some(obj);
}

fn apply_overrides(cfg: &mut DareConfig, ide: &Option<String>, blocks: &std::collections::BTreeMap<String, bool>) {
    if let Some(ide) = ide {
        cfg.ide = Some(ide.clone());
    }
    for (block, enabled) in blocks {
        apply_block_enabled(cfg, block, *enabled);
    }
}

/// Merge layers: defaults ← file ← env ← cli (cli wins).
pub fn merge_layers(
    defaults: &DareConfig,
    file: Option<&DareConfig>,
    env: &EnvOverrides,
    cli: &CliOverrides,
) -> DareConfig {
    let mut out = defaults.clone();
    if let Some(f) = file {
        out.ide = f.ide.clone().or(out.ide);
        out.project = merge_block(out.project.as_ref(), f.project.as_ref());
        out.agent = merge_block(out.agent.as_ref(), f.agent.as_ref());
        out.guard = merge_block(out.guard.as_ref(), f.guard.as_ref());
        out.graph = merge_block(out.graph.as_ref(), f.graph.as_ref());
        out.hooks = merge_block(out.hooks.as_ref(), f.hooks.as_ref());
        out.extra = merge_extras(&out.extra, &f.extra);
    }
    apply_overrides(&mut out, &env.ide, &env.block_enabled);
    apply_overrides(&mut out, &cli.ide, &cli.block_enabled);
    out
}
