//! Update planning domain (microplano 021) + apply policy/backup (022).

mod apply;
mod classify;
mod format;
mod manifest_v2;
mod plan;
mod policy;
mod session_backup;

pub use apply::{
    apply_report_to_json, apply_update, format_apply_human, UpdateApplyReport, MODE_UPDATE,
};
pub use classify::{classify_path, AssetUpdateStatus};
pub use format::{format_human, plan_to_json};
pub use manifest_v2::{
    load_desired_manifest_v2_embedded, load_desired_manifest_v2_from_str, DesiredAsset,
    ReleaseEntry, UpdateManifestV2,
};
pub use plan::{
    parse_harness_target, plan_update, HarnessTarget, UpdateCounts, UpdateItem, UpdatePlan,
    UpdatePlanOptions, MODE_DRY_RUN,
};
pub use policy::{resolve_action, ApplyAction, ApplyOptions, AskContext, AskFn};
pub use session_backup::{
    ensure_backup_root, ensure_parent_dirs, rollback_session, session_backup_file, SessionJournal,
};

/// Schema version of the `UpdatePlan` JSON payload.
pub const UPDATE_PLAN_SCHEMA_VERSION: u32 = 1;

/// Schema version of the desired-state update manifest (V2).
pub const UPDATE_MANIFEST_V2_SCHEMA: u32 = 2;

/// Schema version of the apply report / session journal (BLUEPRINT-022).
pub const UPDATE_APPLY_SCHEMA_VERSION: u32 = 1;

/// Max bytes read when copying a file into the session backup.
pub const APPLY_READ_CAP: usize = 262_144;
