//! `.dare/state.json` v1.

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

use crate::io::{from_json_slice, read_limited, write_json_atomic};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AttemptRecord {
    pub n: u32,
    pub at: String,
    pub passed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_aspect: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskRuntimeState {
    pub status: String,
    #[serde(default)]
    pub output: String,
    #[serde(default)]
    pub error: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
    #[serde(default)]
    pub attempts: Vec<AttemptRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStateV1 {
    pub version: u32,
    pub updated_at: String,
    #[serde(default)]
    pub tasks: BTreeMap<String, TaskRuntimeState>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub fn load_runtime_state(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<RuntimeStateV1> {
    let bytes = read_limited(root, rel)?;
    let state: RuntimeStateV1 = from_json_slice(&bytes)?;
    if state.version != 1 {
        return Err(CoreError::config("unsupported state version"));
    }
    Ok(state)
}

pub fn save_runtime_state(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    state: &RuntimeStateV1,
) -> CoreResult<()> {
    if state.version != 1 {
        return Err(CoreError::config("unsupported state version"));
    }
    write_json_atomic(root, rel, state)
}

pub fn runtime_state_from_str(s: &str) -> CoreResult<RuntimeStateV1> {
    let state: RuntimeStateV1 = from_json_slice(s.as_bytes())?;
    if state.version != 1 {
        return Err(CoreError::config("unsupported state version"));
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_state_rejects_version_2() {
        let raw = r#"{"version":2,"updatedAt":"2026-01-01T00:00:00Z","tasks":{}}"#;
        let err = runtime_state_from_str(raw).unwrap_err();
        assert!(err.to_string().contains("unsupported state version"));
    }

    #[test]
    fn runtime_state_parses_failure_signature() {
        let raw = r#"{
          "version": 1,
          "updatedAt": "2026-01-01T00:00:00Z",
          "tasks": {
            "t1": {
              "status": "FAILED",
              "attempts": [{"n":1,"at":"t","passed":false,"failureSignature":"abcd1234"}]
            }
          }
        }"#;
        let st = runtime_state_from_str(raw).unwrap();
        let a = &st.tasks.get("t1").unwrap().attempts[0];
        assert_eq!(a.failure_signature.as_deref(), Some("abcd1234"));
    }
}
