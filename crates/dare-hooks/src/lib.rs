//! Closed hook events, allowlisted actions, defs load, trust helpers, idempotency, and run.
//!
//! Microplano 048 — domain crate; CLI wiring is separate.

mod action;
mod config;
mod defs;
mod event;
mod idempotency;
mod list_validate;
mod report;
mod run;

pub use action::{action_argv, HookAction};
pub use config::{hooks_enabled, hooks_trusted, MSG_HOOKS_DISABLED, MSG_HOOKS_TRUST};
pub use defs::{
    load_hooks_defs, load_hooks_from_str, HookDef, HooksFile, DEFAULT_HOOKS_YML, HOOKS_OVERLAY_REL,
    HOOKS_READ_CAP, HOOKS_SCHEMA_VERSION,
};
pub use event::HookEvent;
pub use idempotency::{
    digest_key, marker_exists, marker_rel, prune_if_needed, write_marker, IDEMPOTENCY_CAP,
    IDEMPOTENCY_DIR_REL,
};
pub use list_validate::{list_hooks, validate_hooks};
pub use report::{
    HookActionResult, HookListItem, HooksListReport, HooksRunReport, HooksValidateReport,
    HOOKS_LIST_SCHEMA, HOOKS_RUN_SCHEMA, HOOKS_VALIDATE_SCHEMA,
};
pub use run::{run_hooks, RunHooksRequest};
