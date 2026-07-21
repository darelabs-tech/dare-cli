//! Migration plan, dry-run and apply.

use dare_contracts::{load_dare_config, save_dare_config, DareConfig};
use dare_core::fs::backup;
use dare_core::{to_canonical_json_string, CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::defaults::default_config;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStepKind {
    Noop,
    SetEnabled { block: String, enabled: bool },
    WriteSchemaVersion { version: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationStep {
    pub id: String,
    pub pointer: String,
    pub description: String,
    pub kind: MigrationStepKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationPlan {
    pub source_path: String,
    pub steps: Vec<MigrationStep>,
    pub from_fingerprint: String,
    pub would_write_schema_version: bool,
}

#[derive(Debug, Clone)]
pub struct MigrateOptions {
    pub write_schema_version: bool,
    pub schema_version: u32,
}

impl Default for MigrateOptions {
    fn default() -> Self {
        Self {
            write_schema_version: false,
            schema_version: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrateDryRunReport {
    pub plan: MigrationPlan,
    pub before: DareConfig,
    pub after: DareConfig,
    pub writes: bool,
}

fn fingerprint(cfg: &DareConfig) -> String {
    let value = serde_json::to_value(cfg).unwrap_or(json!({}));
    let s = to_canonical_json_string(&value).unwrap_or_else(|_| "{}".into());
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn plan_migrate(current: &DareConfig, opts: &MigrateOptions) -> MigrationPlan {
    let mut steps = Vec::new();
    let mut would_write = false;
    if opts.write_schema_version {
        let version = opts.schema_version.max(1);
        would_write = true;
        steps.push(MigrationStep {
            id: "write-schema-version".into(),
            pointer: "/schemaVersion".into(),
            description: "write schemaVersion when authorized".into(),
            kind: MigrationStepKind::WriteSchemaVersion { version },
        });
    }
    MigrationPlan {
        source_path: "dare.config.json".into(),
        steps,
        from_fingerprint: fingerprint(current),
        would_write_schema_version: would_write,
    }
}

fn set_block_enabled(cfg: &mut DareConfig, block: &str, enabled: bool) {
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

pub fn apply_plan_in_memory(cfg: &DareConfig, plan: &MigrationPlan) -> DareConfig {
    let mut out = cfg.clone();
    for step in &plan.steps {
        match &step.kind {
            MigrationStepKind::Noop => {}
            MigrationStepKind::SetEnabled { block, enabled } => {
                set_block_enabled(&mut out, block, *enabled);
            }
            MigrationStepKind::WriteSchemaVersion { version } => {
                out.extra
                    .insert("schemaVersion".into(), json!(version));
            }
        }
    }
    out
}

fn load_or_default(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<(DareConfig, bool)> {
    match load_dare_config(root, rel) {
        Ok(cfg) => Ok((cfg, true)),
        Err(CoreError::NotFound(_)) => Ok((default_config(), false)),
        Err(e) => Err(e),
    }
}

pub fn dry_run_migrate(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    opts: &MigrateOptions,
) -> CoreResult<MigrateDryRunReport> {
    let (before, _) = load_or_default(root, rel)?;
    let plan = plan_migrate(&before, opts);
    let after = apply_plan_in_memory(&before, &plan);
    Ok(MigrateDryRunReport {
        plan,
        before,
        after,
        writes: false,
    })
}

pub fn apply_migrate(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    opts: &MigrateOptions,
) -> CoreResult<MigrationPlan> {
    let (before, existed) = load_or_default(root, rel)?;
    let plan = plan_migrate(&before, opts);
    if plan.steps.is_empty() {
        return Ok(plan);
    }
    if existed {
        backup(root, rel)?;
    }
    let after = apply_plan_in_memory(&before, &plan);
    save_dare_config(root, rel, &after)?;
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn dry_run_does_not_write() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("dare.config.json").unwrap();
        let raw = r#"{"ide":"cursor","customExtension":{"x":1}}"#;
        std::fs::write(dir.path().join("dare.config.json"), raw).unwrap();
        let before = std::fs::read(dir.path().join("dare.config.json")).unwrap();
        let opts = MigrateOptions {
            write_schema_version: true,
            schema_version: 1,
        };
        let report = dry_run_migrate(&root, &rel, &opts).unwrap();
        assert!(!report.writes);
        assert!(report.after.extra.contains_key("schemaVersion"));
        let after = std::fs::read(dir.path().join("dare.config.json")).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn apply_creates_backup_and_writes() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("dare.config.json").unwrap();
        let raw = r#"{"ide":"cursor"}"#;
        std::fs::write(dir.path().join("dare.config.json"), raw).unwrap();
        let opts = MigrateOptions {
            write_schema_version: true,
            schema_version: 1,
        };
        let plan = apply_migrate(&root, &rel, &opts).unwrap();
        assert!(!plan.steps.is_empty());
        let cfg = load_dare_config(&root, &rel).unwrap();
        assert_eq!(cfg.extra.get("schemaVersion"), Some(&json!(1)));
        let backups = std::fs::read_dir(dir.path().join(".dare/backups")).unwrap();
        assert!(backups.count() >= 1);
    }
}
