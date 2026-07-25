//! Scaffold contracts, stack registry and scaffolder trait (microplano 046).

mod apply;
mod ax;
mod plan;
mod registry;
mod render;
mod trait_api;
mod types;

pub use apply::{apply_scaffold, run_scaffold};
pub use ax::{ax_artifact_paths, generate_ax_files, AX_ARTIFACT_COUNT, OPENAPI_STUB_VERSION};
pub use plan::{plan_scaffold, validate_project_name, PROJECT_NAME_RE};
pub use registry::{
    list_stack_ids, scaffolder_for, MSG_HINT_RAILS, MSG_UNKNOWN_STACK, STACK_IDS,
};
pub use render::{render_template, scan_secrets, SECRET_SCAN_NEEDLES};
pub use trait_api::{GenericScaffolder, StackScaffolder};
pub use types::{
    FrontendKind, PlanAction, PlanItemKind, ScaffoldApplyReport, ScaffoldPlan, ScaffoldPlanItem,
    ScaffoldRequest, StackKind, StackMetadata, Toolchain, Transport, ValidationReport,
    SCHEMA_VERSION,
};
