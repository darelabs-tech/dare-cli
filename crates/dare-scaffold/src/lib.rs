//! Scaffold contracts, stack registry and scaffolder trait (microplano 046).

mod registry;
mod trait_api;
mod types;

pub use registry::{
    list_stack_ids, scaffolder_for, MSG_HINT_RAILS, MSG_UNKNOWN_STACK, STACK_IDS,
};
pub use trait_api::{GenericScaffolder, StackScaffolder};
pub use types::{
    FrontendKind, PlanAction, PlanItemKind, ScaffoldApplyReport, ScaffoldPlan, ScaffoldPlanItem,
    ScaffoldRequest, StackKind, StackMetadata, Toolchain, Transport, ValidationReport,
    SCHEMA_VERSION,
};
