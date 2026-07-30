//! Shared HTTP helpers for REST routes.

use dare_core::{CoreError, ErrorKind};
use dare_steering::{MSG_ENV_EXCLUDED, MSG_PATH_ESCAPE as STEERING_PATH_ESCAPE};

use crate::error::HttpError;
use crate::tasks_md::MSG_PATH_ESCAPE;

pub const MSG_GRAPH_DISABLED: &str = "graph unavailable";
pub const MSG_INVALID_CONTEXT_TYPE: &str =
    "invalid context type (expected architecture|task|dependency)";
pub const BLUEPRINT_REL: &str = "DARE/BLUEPRINT.md";
pub const DAG_REL: &str = "DARE/dare-dag.yaml";
pub const BLUEPRINT_MAX_BYTES: u64 = 2 * 1024 * 1024;

pub fn map_core_error(err: CoreError) -> HttpError {
    let msg = err.to_string();
    match err.kind() {
        ErrorKind::NotFound => HttpError::not_found(msg),
        ErrorKind::InvalidInput => {
            if msg == MSG_PATH_ESCAPE
                || msg == STEERING_PATH_ESCAPE
                || msg.contains("path escape")
                || msg.contains("path must be relative")
            {
                HttpError::path_escape(MSG_PATH_ESCAPE)
            } else if msg == MSG_ENV_EXCLUDED {
                HttpError::forbidden(msg)
            } else {
                HttpError::invalid_input(msg)
            }
        }
        ErrorKind::Config => HttpError::invalid_input(msg),
        ErrorKind::Io | ErrorKind::Internal | ErrorKind::GuardFail | ErrorKind::Usage => {
            HttpError::internal(msg)
        }
    }
}
