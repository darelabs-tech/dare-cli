//! Persisted contract schemas and canonical readers/writers.

mod config;
mod dag;
mod graph;
mod io;
mod skills;
mod state;
mod telemetry;
mod update_manifest;
mod verification;

pub use config::{
    load_dare_config, save_dare_config, ConfigObject, DareConfig,
};
pub use dag::{
    load_dag, parse_dag_yaml, save_dag, DagDocument, DagLimits, DagTask, DagV21, LegacyDag,
    LegacyTask,
};
pub use graph::{
    canonical_edge_id, canonical_file_node_id, canonical_task_node_id, load_graph, save_graph,
    GraphDocument, GraphEdge, GraphNode,
};
pub use io::{
    from_json_slice, from_yaml_str, read_limited, write_json_atomic, write_yaml_atomic,
};
pub use skills::{load_skills_manifest, save_skills_manifest, SkillEntry, SkillsManifest};
pub use state::{
    load_runtime_state, runtime_state_from_str, save_runtime_state, AttemptRecord, RuntimeStateV1,
    TaskRuntimeState,
};
pub use telemetry::{
    telemetry_snapshot_from_str, telemetry_snapshot_to_canonical_json, TelemetrySnapshot,
};
pub use update_manifest::{
    load_update_manifest, save_update_manifest, update_manifest_from_str, UpdateManifestV1,
};
pub use verification::{
    load_verification_baseline, save_verification_baseline, verification_baseline_from_str,
    VerificationBaseline,
};

/// Schema version announced by this crate (not a disk field).
pub const CONTRACTS_SCHEMA_VERSION: &str = "0.1.0-contracts";

/// Maximum contract file size (bytes).
pub const MAX_CONTRACT_BYTES: u64 = 2 * 1024 * 1024;

/// Returns the version of schema announced by this crate.
pub fn schema_version() -> &'static str {
    CONTRACTS_SCHEMA_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_is_contracts_010() {
        assert_eq!(schema_version(), "0.1.0-contracts");
    }
}
