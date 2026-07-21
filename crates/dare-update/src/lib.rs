//! Update planning domain (microplano 021) — dry-run classify + plan.

mod classify;
mod format;
mod manifest_v2;
mod plan;

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

/// Schema version of the `UpdatePlan` JSON payload.
pub const UPDATE_PLAN_SCHEMA_VERSION: u32 = 1;

/// Schema version of the desired-state update manifest (V2).
pub const UPDATE_MANIFEST_V2_SCHEMA: u32 = 2;
