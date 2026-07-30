//! Map domain errors to MCP JSON-RPC `ErrorData` (microplano 052).

use dare_core::{CoreError, ErrorKind};
use dare_steering::{MSG_ENV_EXCLUDED, MSG_PATH_ESCAPE as STEERING_PATH_ESCAPE};
use rmcp::model::{ErrorCode, ErrorData};
use serde_json::json;

use crate::error::HttpError;
use crate::http_map::MSG_GRAPH_DISABLED;
use crate::tasks_md::MSG_PATH_ESCAPE;

/// JSON-RPC application / server error band (custom MCP domain codes in `data.code`).
const APPLICATION_ERROR: ErrorCode = ErrorCode(-32000);

fn sanitize(msg: &str) -> String {
    dare_core::redact(msg)
}

fn with_code(code: &str) -> Option<serde_json::Value> {
    Some(json!({ "code": code }))
}

fn application(code: &str, message: impl Into<String>) -> ErrorData {
    let message = sanitize(&message.into());
    ErrorData::new(APPLICATION_ERROR, message, with_code(code))
}

/// Map [`CoreError`] → MCP [`ErrorData`].
///
/// - InvalidInput / Usage / path escape / env deny → `invalid_params` (−32602)
/// - NotFound → `invalid_params` with `data.code = "not_found"`
/// - graph unavailable → application `graph_unavailable`
/// - Io / Internal / GuardFail → `internal_error` (−32603)
pub fn map_core_error(err: CoreError) -> ErrorData {
    let msg = sanitize(err.message());

    if msg == MSG_GRAPH_DISABLED {
        return application("graph_unavailable", msg);
    }

    match err.kind() {
        ErrorKind::InvalidInput | ErrorKind::Usage | ErrorKind::Config => {
            let code = if msg == MSG_PATH_ESCAPE
                || msg == STEERING_PATH_ESCAPE
                || msg.contains("path escape")
                || msg.contains("path must be relative")
            {
                "path_escape"
            } else if msg == MSG_ENV_EXCLUDED {
                "forbidden"
            } else {
                "invalid_input"
            };
            ErrorData::invalid_params(msg, with_code(code))
        }
        ErrorKind::NotFound => ErrorData::invalid_params(msg, with_code("not_found")),
        ErrorKind::Io | ErrorKind::Internal | ErrorKind::GuardFail => {
            ErrorData::internal_error(msg, with_code("internal"))
        }
    }
}

/// Map HTTP Class A errors (when a route-shaped error surfaces near MCP).
pub fn map_http_error(err: &HttpError) -> ErrorData {
    let msg = sanitize(&err.body.error);
    match err.body.code.as_str() {
        "graph_unavailable" => application("graph_unavailable", msg),
        "not_found" => ErrorData::invalid_params(msg, with_code("not_found")),
        "path_escape" => ErrorData::invalid_params(msg, with_code("path_escape")),
        "forbidden" => ErrorData::invalid_params(msg, with_code("forbidden")),
        "invalid_input" | "unauthorized" => {
            ErrorData::invalid_params(msg, with_code("invalid_input"))
        }
        "body_too_large" | "internal" => {
            ErrorData::internal_error(msg, with_code("internal"))
        }
        other => ErrorData::internal_error(msg, with_code(other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_input_is_invalid_params() {
        let err = map_core_error(CoreError::invalid_input("bad"));
        assert_eq!(err.code, ErrorCode::INVALID_PARAMS);
        assert_eq!(err.data.as_ref().unwrap()["code"], "invalid_input");
    }

    #[test]
    fn graph_unavailable_is_application() {
        let err = map_core_error(CoreError::internal(MSG_GRAPH_DISABLED));
        assert_eq!(err.code, APPLICATION_ERROR);
        assert_eq!(err.data.as_ref().unwrap()["code"], "graph_unavailable");
    }

    #[test]
    fn not_found_carries_code() {
        let err = map_core_error(CoreError::not_found("missing"));
        assert_eq!(err.code, ErrorCode::INVALID_PARAMS);
        assert_eq!(err.data.as_ref().unwrap()["code"], "not_found");
    }
}
