//! Closed hook events, allowlisted actions, and defs load (embed + overlay).
//!
//! Microplano 048 — domain crate only; no CLI / spawn / trust gate here.

mod action;
mod defs;
mod event;

pub use action::{action_argv, HookAction};
pub use defs::{
    load_hooks_defs, load_hooks_from_str, HookDef, HooksFile, DEFAULT_HOOKS_YML, HOOKS_OVERLAY_REL,
    HOOKS_READ_CAP, HOOKS_SCHEMA_VERSION,
};
pub use event::HookEvent;
