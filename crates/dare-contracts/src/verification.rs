//! `.dare/verification/<id>.json`.

use dare_core::CoreResult;
use dare_core::{ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::io::{from_json_slice, read_limited, write_json_atomic};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct VerificationBaseline {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default)]
    pub aspects: Map<String, Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub fn load_verification_baseline(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
) -> CoreResult<VerificationBaseline> {
    let bytes = read_limited(root, rel)?;
    from_json_slice(&bytes)
}

pub fn save_verification_baseline(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    baseline: &VerificationBaseline,
) -> CoreResult<()> {
    write_json_atomic(root, rel, baseline)
}

pub fn verification_baseline_from_str(s: &str) -> CoreResult<VerificationBaseline> {
    from_json_slice(s.as_bytes())
}
