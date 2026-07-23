//! Ralph Loop verification: stack adapters + gated build → test → lint.

pub mod ralph;
pub mod stacks;
pub mod verification;

pub use ralph::{run_ralph, GateAspect, GateStep, RalphReport, RALPH_TIMEOUT_SECS};
pub use stacks::{gate_commands, resolve_stack};
pub use verification::{
    load_verification, task_id_is_path_safe, verification_from_ralph, write_verification,
    VerificationReport, VERIFICATION_DIR_REL,
};
