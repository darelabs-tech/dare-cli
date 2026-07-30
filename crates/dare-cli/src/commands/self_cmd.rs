//! `dare self` — CLI binary self-update / rollback / uninstall (microplano 053).
//!
//! Distinct from `dare update`, which refreshes **project** assets under a ProjectRoot.

use std::io::{self, IsTerminal, Write};

use dare_core::{
    CoreError, CoreResult, SystemProcessRunner,
};
use dare_self::{
    apply_update, plan_update, rollback, uninstall, Channel, CosignCliVerifier, DEFAULT_CHANNEL,
    RollbackOpts, UninstallOpts, UpdateOpts, UpdatePlan,
};
use serde_json::{json, Value};

/// CLI args for `dare self update`.
pub struct SelfUpdateCliOpts {
    pub channel: Option<String>,
    pub version: Option<String>,
    pub dry_run: bool,
    pub yes: bool,
    pub force_unlock: bool,
}

/// CLI args for `dare self rollback`.
pub struct SelfRollbackCliOpts {
    pub yes: bool,
}

/// CLI args for `dare self uninstall`.
pub struct SelfUninstallCliOpts {
    pub yes: bool,
}

/// Run `dare self update`.
pub fn run_self_update(opts: SelfUpdateCliOpts) -> CoreResult<(String, Value)> {
    let update_opts = build_update_opts(&opts)?;

    if opts.dry_run {
        let plan = plan_update(update_opts)?;
        let human = format_plan_human(&plan);
        let data = plan_to_json(&plan);
        return Ok((human, data));
    }

    confirm_proceed(opts.yes)?;

    let runner = SystemProcessRunner;
    let verifier = CosignCliVerifier::new(&runner);
    let report = apply_update(update_opts, &verifier, opts.force_unlock)?;

    let human = format!(
        "self update: ok\nchannel: {}\ncurrent: {}\ntarget: {}\nasset: {}\nbackup: {}\nreplaced: {}\nmode: {}",
        report.channel,
        report.current_version,
        report.target_tag,
        report.asset_name,
        report.backup_path.display(),
        report.replaced_path.display(),
        report.mode,
    );
    let data = json!({
        "schemaVersion": report.schema_version,
        "ok": report.ok,
        "mode": report.mode,
        "channel": report.channel,
        "currentVersion": report.current_version,
        "targetTag": report.target_tag,
        "targetTriple": report.target_triple,
        "assetName": report.asset_name,
        "backupPath": report.backup_path.to_string_lossy(),
        "replacedPath": report.replaced_path.to_string_lossy(),
    });
    Ok((human, data))
}

/// Run `dare self rollback`.
pub fn run_self_rollback(opts: SelfRollbackCliOpts) -> CoreResult<(String, Value)> {
    confirm_proceed(opts.yes)?;
    let report = rollback(RollbackOpts {
        home: None,
        current_exe: None,
        force_unlock: false,
    })?;
    let human = format!(
        "self rollback: ok\nbackup: {}\nrestored: {}\nmode: {}",
        report.backup_path.display(),
        report.restored_path.display(),
        report.mode,
    );
    let data = json!({
        "schemaVersion": report.schema_version,
        "ok": report.ok,
        "mode": report.mode,
        "backupPath": report.backup_path.to_string_lossy(),
        "restoredPath": report.restored_path.to_string_lossy(),
        "version": report.version,
    });
    Ok((human, data))
}

/// Run `dare self uninstall`.
pub fn run_self_uninstall(opts: SelfUninstallCliOpts) -> CoreResult<(String, Value)> {
    confirm_proceed(opts.yes)?;
    let report = uninstall(UninstallOpts { target: None })?;
    let human = format!(
        "self uninstall: ok\nremoved: {}\nmode: {}",
        report.removed_path.display(),
        report.mode,
    );
    let data = json!({
        "schemaVersion": report.schema_version,
        "ok": report.ok,
        "mode": report.mode,
        "removedPath": report.removed_path.to_string_lossy(),
        "version": report.version,
    });
    Ok((human, data))
}

fn build_update_opts(opts: &SelfUpdateCliOpts) -> CoreResult<UpdateOpts> {
    let channel_flag = opts.channel.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let version_flag = opts.version.as_deref().map(str::trim).filter(|s| !s.is_empty());

    match (channel_flag, version_flag) {
        (Some(_), Some(_)) => Err(CoreError::usage(
            "provide either --channel or --version, not both",
        )),
        (Some(ch), None) => {
            let channel = Channel::parse(ch).map_err(|e| CoreError::usage(e.to_string()))?;
            Ok(UpdateOpts {
                channel: Some(channel),
                version: None,
                current_version: Some(env!("CARGO_PKG_VERSION").to_string()),
                triple: None,
            })
        }
        (None, Some(ver)) => Ok(UpdateOpts {
            channel: None,
            version: Some(ver.to_string()),
            current_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            triple: None,
        }),
        (None, None) => Ok(UpdateOpts {
            channel: Some(DEFAULT_CHANNEL),
            version: None,
            current_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            triple: None,
        }),
    }
}

fn confirm_proceed(yes: bool) -> CoreResult<()> {
    if yes {
        return Ok(());
    }
    let tty = io::stdin().is_terminal() && io::stdout().is_terminal();
    if !tty {
        return Err(CoreError::invalid_input(
            "non-interactive session requires --yes",
        ));
    }
    let _ = write!(io::stderr(), "Proceed? [y/N] ");
    let _ = io::stderr().flush();
    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(0) => Err(CoreError::invalid_input("confirmation denied")),
        Ok(_) => {
            let t = line.trim();
            if t == "y" || t == "Y" || t == "yes" {
                Ok(())
            } else {
                Err(CoreError::invalid_input("confirmation denied"))
            }
        }
        Err(_) => Err(CoreError::invalid_input("confirmation denied")),
    }
}

fn format_plan_human(plan: &UpdatePlan) -> String {
    format!(
        "self update: dry-run\nchannel: {}\ncurrent: {}\ntarget: {}\nasset: {}\nactions: {}\nmode: update",
        plan.channel,
        plan.current_version,
        plan.target_tag,
        plan.asset_name,
        plan.actions.join(", "),
    )
}

fn plan_to_json(plan: &UpdatePlan) -> Value {
    json!({
        "schemaVersion": plan.schema_version,
        "ok": true,
        "mode": "update",
        "channel": plan.channel,
        "currentVersion": plan.current_version,
        "targetTag": plan.target_tag,
        "targetTriple": plan.target_triple,
        "assetName": plan.asset_name,
        "assetUrl": plan.asset_url,
        "sumsUrl": plan.sums_url,
        "sigUrl": plan.sig_url,
        "actions": plan.actions,
    })
}
