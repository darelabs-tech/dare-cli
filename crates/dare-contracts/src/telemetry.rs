//! Dashboard `TelemetrySnapshot`.

use dare_core::CoreResult;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::io::from_json_slice;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct TelemetrySnapshot {
    #[serde(default)]
    pub dag: Map<String, Value>,
    #[serde(default)]
    pub gates: Map<String, Value>,
    #[serde(default)]
    pub cost: Map<String, Value>,
    #[serde(default)]
    pub best_of_n: Map<String, Value>,
    #[serde(default)]
    pub guard: Map<String, Value>,
    #[serde(default)]
    pub drift: Map<String, Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub fn telemetry_snapshot_from_str(s: &str) -> CoreResult<TelemetrySnapshot> {
    from_json_slice(s.as_bytes())
}

pub fn telemetry_snapshot_to_canonical_json(snap: &TelemetrySnapshot) -> CoreResult<String> {
    let v = serde_json::to_value(snap).map_err(|e| dare_core::CoreError::config(e.to_string()))?;
    dare_core::to_canonical_json_string(&v)
}
