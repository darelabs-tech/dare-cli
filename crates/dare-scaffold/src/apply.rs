//! Scaffold apply, journal, and rollback (BLUEPRINT-046 §0.5 / mp046-004).

use std::collections::HashMap;

use dare_assets::EmbeddedAssets;
use dare_core::fs::{atomic_write, restore};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};

use crate::ax::generate_ax_files;
use crate::plan::plan_scaffold;
use crate::render::render_template;
use crate::types::{
    FrontendKind, PlanAction, PlanItemKind, ScaffoldApplyReport, ScaffoldPlan, ScaffoldRequest,
    SCHEMA_VERSION,
};

const JOURNAL_DIR_PREFIX: &str = ".dare/scaffold-session";

/// In-memory journal for one scaffold apply session.
#[derive(Debug, Clone, Default)]
struct ScaffoldJournal {
    backup_root: String,
    backed_up: Vec<(String, String)>,
    created: Vec<String>,
}

impl ScaffoldJournal {
    fn has_writes(&self) -> bool {
        !self.backed_up.is_empty() || !self.created.is_empty()
    }
}

fn utc_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    const SECS_PER_DAY: u64 = 86400;
    let days = secs / SECS_PER_DAY;
    let rem = secs % SECS_PER_DAY;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;
    let (y, m, d) = civil_from_days(days as i64);
    format!("{y:04}{m:02}{d:02}T{hour:02}{min:02}{sec:02}Z")
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

fn ensure_journal_root(root: &ProjectRoot) -> CoreResult<SafeRelativePath> {
    let stamp = utc_stamp();
    let rel = SafeRelativePath::new(&format!("{JOURNAL_DIR_PREFIX}-{stamp}"))?;
    let abs = root.resolve(&rel)?;
    std::fs::create_dir_all(abs.as_path().as_std_path())
        .map_err(|e| CoreError::io(e.to_string()))?;
    Ok(rel)
}

fn read_file_bytes(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<Vec<u8>> {
    let abs = root.resolve(rel)?;
    std::fs::read(abs.as_path().as_std_path()).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CoreError::not_found(format!("file not found: {}", rel.as_str()))
        } else {
            CoreError::io(e.to_string())
        }
    })
}

fn backup_file(
    root: &ProjectRoot,
    journal_root: &SafeRelativePath,
    dest_rel: &str,
    journal: &mut ScaffoldJournal,
) -> CoreResult<()> {
    let dest = SafeRelativePath::new(dest_rel)?;
    let bytes = read_file_bytes(root, &dest)?;
    let bak = SafeRelativePath::new(&format!(
        "{}/{}",
        journal_root.as_str(),
        dest_rel
    ))?;
    atomic_write(root, &bak, &bytes)?;
    journal
        .backed_up
        .push((dest_rel.to_string(), bak.as_str().to_string()));
    Ok(())
}

fn rollback_journal(root: &ProjectRoot, journal: &ScaffoldJournal) -> CoreResult<()> {
    for (dest, bak) in journal.backed_up.iter().rev() {
        let dest_rel = SafeRelativePath::new(dest)
            .map_err(|e| CoreError::internal(format!("rollback invalid dest `{dest}`: {e}")))?;
        let bak_rel = SafeRelativePath::new(bak)
            .map_err(|e| CoreError::internal(format!("rollback invalid backup `{bak}`: {e}")))?;
        restore(root, &bak_rel, &dest_rel).map_err(|e| {
            CoreError::internal(format!("rollback restore failed for `{dest}`: {e}"))
        })?;
    }

    for rel in journal.created.iter().rev() {
        let Ok(p) = SafeRelativePath::new(rel) else {
            continue;
        };
        let Ok(abs) = root.resolve(&p) else {
            continue;
        };
        if abs.as_path().is_file() {
            let _ = std::fs::remove_file(abs.as_path().as_std_path());
        }
    }
    Ok(())
}

fn remove_journal_dir(root: &ProjectRoot, journal_root: &SafeRelativePath) -> CoreResult<()> {
    let abs = root.resolve(journal_root)?;
    if abs.as_path().is_dir() {
        std::fs::remove_dir_all(abs.as_path().as_std_path())
            .map_err(|e| CoreError::io(e.to_string()))?;
    }
    Ok(())
}

fn frontend_asset_id(kind: FrontendKind) -> &'static str {
    match kind {
        FrontendKind::React => "react",
        FrontendKind::Vue => "vue",
    }
}

fn embedded_frontend_asset_key(frontend: FrontendKind, dest_path: &str) -> CoreResult<String> {
    let rel = dest_path.strip_prefix("frontend/").ok_or_else(|| {
        CoreError::internal(format!(
            "frontend asset dest must start with `frontend/`: `{dest_path}`"
        ))
    })?;
    let fe_id = frontend_asset_id(frontend);
    let tpl = format!("stacks/_frontend/{fe_id}/{rel}.tpl");
    if EmbeddedAssets::get(&tpl).is_some() {
        return Ok(tpl);
    }
    let plain = format!("stacks/_frontend/{fe_id}/{rel}");
    if EmbeddedAssets::get(&plain).is_some() {
        return Ok(plain);
    }
    Err(CoreError::not_found(format!(
        "embedded frontend asset missing for `{dest_path}` (frontend `{fe_id}`)"
    )))
}

fn embedded_asset_key(stack_id: &str, dest_path: &str) -> CoreResult<String> {
    let tpl = format!("stacks/{stack_id}/{dest_path}.tpl");
    if EmbeddedAssets::get(&tpl).is_some() {
        return Ok(tpl);
    }
    let plain = format!("stacks/{stack_id}/{dest_path}");
    if EmbeddedAssets::get(&plain).is_some() {
        return Ok(plain);
    }
    Err(CoreError::not_found(format!(
        "embedded asset missing for `{dest_path}` (stack `{stack_id}`)"
    )))
}

fn resolve_item_bytes(
    item_kind: PlanItemKind,
    stack_id: &str,
    project_name: &str,
    dest_path: &str,
    ax_index: &HashMap<String, String>,
    frontend: Option<FrontendKind>,
) -> CoreResult<Vec<u8>> {
    match item_kind {
        PlanItemKind::Ax => {
            let content = ax_index.get(dest_path).ok_or_else(|| {
                CoreError::internal(format!("AX content missing for `{dest_path}`"))
            })?;
            Ok(content.as_bytes().to_vec())
        }
        PlanItemKind::Meta | PlanItemKind::Template => {
            let key = if dest_path.starts_with("frontend/") {
                let fe = frontend.ok_or_else(|| {
                    CoreError::internal(format!(
                        "frontend asset `{dest_path}` requires plan.frontend"
                    ))
                })?;
                embedded_frontend_asset_key(fe, dest_path)?
            } else {
                embedded_asset_key(stack_id, dest_path)?
            };
            let file = EmbeddedAssets::get(&key).ok_or_else(|| {
                CoreError::not_found(format!("embedded asset missing: {key}"))
            })?;
            let text = std::str::from_utf8(file.data.as_ref()).map_err(|e| {
                CoreError::Internal(format!("invalid UTF-8 in template `{key}`: {e}"))
            })?;
            let rendered = render_template(text, project_name, stack_id)?;
            Ok(rendered.into_bytes())
        }
    }
}

#[cfg(test)]
struct Failpoint {
    after_n: usize,
    successful_writes: usize,
}

#[cfg(test)]
impl Failpoint {
    fn new(after_n: usize) -> Self {
        Self {
            after_n,
            successful_writes: 0,
        }
    }

    fn after_successful_write(&mut self) -> CoreResult<()> {
        self.successful_writes += 1;
        if self.successful_writes >= self.after_n {
            return Err(CoreError::io(format!(
                "failpoint: simulated failure after {} write(s)",
                self.after_n
            )));
        }
        Ok(())
    }
}

fn note_write(#[cfg(test)] failpoint: &mut Option<Failpoint>) -> CoreResult<()> {
    #[cfg(test)]
    if let Some(fp) = failpoint.as_mut() {
        return fp.after_successful_write();
    }
    Ok(())
}

fn apply_scaffold_inner(
    root: &ProjectRoot,
    plan: &ScaffoldPlan,
    #[cfg(test)] failpoint: &mut Option<Failpoint>,
) -> CoreResult<ScaffoldApplyReport> {
    let meta = crate::registry::scaffolder_for(&plan.stack_id)?.metadata();
    let ax_files = generate_ax_files(meta, &plan.project_name)?;
    let ax_index: HashMap<String, String> = ax_files.into_iter().collect();

    let mut journal = ScaffoldJournal::default();
    let journal_root = ensure_journal_root(root)?;
    journal.backup_root = journal_root.as_str().to_string();

    let mut created = Vec::new();
    let mut replaced = Vec::new();
    let mut skipped = Vec::new();

    let mut apply_body = || -> CoreResult<()> {
        for item in &plan.items {
            if item.action == PlanAction::Skip {
                skipped.push(item.path.clone());
                continue;
            }

            let dest = SafeRelativePath::new(&item.path)?;
            let existed = root.resolve(&dest)?.as_path().is_file();

            if item.action == PlanAction::Replace || existed {
                backup_file(root, &journal_root, &item.path, &mut journal)?;
            }

            let bytes = resolve_item_bytes(
                item.kind,
                &plan.stack_id,
                &plan.project_name,
                &item.path,
                &ax_index,
                plan.frontend,
            )?;
            atomic_write(root, &dest, &bytes)?;

            if existed {
                replaced.push(item.path.clone());
            } else {
                journal.created.push(item.path.clone());
                created.push(item.path.clone());
            }

            note_write(
                #[cfg(test)]
                failpoint,
            )?;
        }
        Ok(())
    };

    if let Err(e) = apply_body() {
        if journal.has_writes() {
            match rollback_journal(root, &journal) {
                Ok(()) => return Err(e),
                Err(rb) => {
                    return Err(CoreError::internal(format!(
                        "apply failed ({e}); rollback also failed ({rb})"
                    )));
                }
            }
        }
        return Err(e);
    }

    created.sort();
    replaced.sort();
    skipped.sort();
    remove_journal_dir(root, &journal_root)?;

    Ok(ScaffoldApplyReport {
        schema_version: SCHEMA_VERSION,
        stack_id: plan.stack_id.clone(),
        created,
        replaced,
        skipped,
        rolled_back: false,
        check: false,
    })
}

/// Apply a [`ScaffoldPlan`]: journal backups, atomic writes, rollback on failure.
pub fn apply_scaffold(root: &ProjectRoot, plan: &ScaffoldPlan) -> CoreResult<ScaffoldApplyReport> {
    #[cfg(test)]
    {
        apply_scaffold_inner(root, plan, &mut None)
    }
    #[cfg(not(test))]
    {
        apply_scaffold_inner(root, plan)
    }
}

#[cfg(test)]
pub(crate) fn apply_scaffold_with_failpoint(
    root: &ProjectRoot,
    plan: &ScaffoldPlan,
    after_n: usize,
) -> CoreResult<ScaffoldApplyReport> {
    apply_scaffold_inner(root, plan, &mut Some(Failpoint::new(after_n)))
}

/// Plan then apply; `req.check` performs a dry-run with zero filesystem writes.
pub fn run_scaffold(
    root: &ProjectRoot,
    req: &ScaffoldRequest,
) -> CoreResult<ScaffoldApplyReport> {
    let plan = plan_scaffold(root, req)?;
    if req.check {
        let mut skipped: Vec<String> = plan.items.iter().map(|i| i.path.clone()).collect();
        skipped.sort();
        return Ok(ScaffoldApplyReport {
            schema_version: SCHEMA_VERSION,
            stack_id: plan.stack_id,
            created: vec![],
            replaced: vec![],
            skipped,
            rolled_back: false,
            check: true,
        });
    }
    apply_scaffold(root, &plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::plan_scaffold;
    use crate::types::Toolchain;
    use dare_core::fs::atomic_write;
    use std::fs;
    use tempfile::tempdir;

    fn sample_req(stack_id: &str, force: bool, check: bool) -> ScaffoldRequest {
        ScaffoldRequest {
            project_name: "demo-app".to_string(),
            stack_id: stack_id.to_string(),
            toolchain: Toolchain::None,
            transport: None,
            frontend: None,
            conflict_policy: crate::types::ConflictPolicy::FailFast,
            force,
            check,
        }
    }

    #[test]
    fn check_zero_write() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let req = sample_req("go-gin", false, true);

        let report = run_scaffold(&root, &req).expect("check run");
        assert!(report.check);
        assert!(report.created.is_empty());
        assert!(report.replaced.is_empty());
        assert!(!report.skipped.is_empty());
        assert!(!report.rolled_back);

        assert!(!dir.path().join("dare.config.json").exists());
        let dare_dir = dir.path().join(".dare");
        if dare_dir.exists() {
            let names: Vec<_> = fs::read_dir(&dare_dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            assert!(
                names.iter().all(|n| !n.starts_with("scaffold-session")),
                "check mode must not create scaffold journal, found: {names:?}"
            );
        }
    }

    #[test]
    fn force_replace() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("dare.config.json").unwrap();
        atomic_write(&root, &rel, br#"{"old":true}"#).unwrap();

        let mut req = sample_req("go-gin", true, false);
        let plan = plan_scaffold(&root, &req).expect("plan");
        let item = plan
            .items
            .iter()
            .find(|i| i.path == "dare.config.json")
            .expect("dare.config.json in plan");
        assert_eq!(item.action, PlanAction::Replace);

        let report = apply_scaffold(&root, &plan).expect("apply");
        assert!(report.replaced.contains(&"dare.config.json".to_string()));
        let content = fs::read_to_string(dir.path().join("dare.config.json")).unwrap();
        assert!(content.contains("demo-app"));
        assert!(!content.contains("\"old\":true"));
        let _ = &mut req;
    }

    #[test]
    fn rollback_restores() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("dare.config.json").unwrap();
        atomic_write(&root, &rel, br#"{"seed":"original"}"#).unwrap();

        let req = sample_req("go-gin", true, false);
        let plan = plan_scaffold(&root, &req).expect("plan");
        let err = apply_scaffold_with_failpoint(&root, &plan, 2).expect_err("must fail at failpoint");
        assert!(
            err.to_string().contains("failpoint"),
            "expected failpoint error, got {err}"
        );

        assert_eq!(
            fs::read(dir.path().join("dare.config.json")).unwrap(),
            br#"{"seed":"original"}"#
        );
        assert!(
            !dir.path().join(".env.example").exists(),
            "created .env.example should be rolled back"
        );
    }
}
