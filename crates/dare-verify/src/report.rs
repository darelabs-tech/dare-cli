//! Advanced verification report types (`LoopVerdict`, `AspectResult`).

use serde::{Deserialize, Serialize};

/// Schema version for [`LoopVerdict`] JSON.
pub const LOOP_VERDICT_SCHEMA: u32 = 1;

/// Advanced verification aspects (distinct from Ralph [`crate::GateAspect`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvancedAspect {
    FailToPass,
    AntiTamper,
    Mutation,
    Formal,
}

impl AdvancedAspect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FailToPass => "fail-to-pass",
            Self::AntiTamper => "anti-tamper",
            Self::Mutation => "mutation",
            Self::Formal => "formal",
        }
    }
}

/// Outcome of one advanced aspect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AspectStatus {
    Pass,
    Fail,
    Skipped,
}

/// One advanced-aspect result row in a [`LoopVerdict`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AspectResult {
    pub aspect: AdvancedAspect,
    pub status: AspectStatus,
    pub score: Option<f64>,
    pub reason: Option<String>,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub stdout_tail: String,
    pub stderr_tail: String,
}

/// Candidate summary inside optional best-of-N payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BestOfCandidate {
    pub id: u32,
    pub aspects_passed: u32,
    pub mutation_score: f64,
    pub duration_ms: u64,
    pub ok: bool,
}

/// Optional best-of-N block on [`LoopVerdict`] (§4.1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BestOfSummary {
    pub n: u32,
    pub candidates: Vec<BestOfCandidate>,
    pub pareto_ids: Vec<u32>,
    pub winner_id: Option<u32>,
}

/// Aggregate advanced-verify verdict (`schemaVersion` 1, camelCase JSON).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LoopVerdict {
    pub schema_version: u32,
    pub task_id: String,
    pub ok: bool,
    pub ralph_ok: bool,
    pub policy: String,
    pub decay_action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_of: Option<BestOfSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub winner_id: Option<u32>,
    pub aspects: Vec<AspectResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_signature: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aspect_status_serde() {
        assert_eq!(
            serde_json::to_string(&AspectStatus::Pass).unwrap(),
            "\"pass\""
        );
        assert_eq!(
            serde_json::to_string(&AspectStatus::Fail).unwrap(),
            "\"fail\""
        );
        assert_eq!(
            serde_json::to_string(&AspectStatus::Skipped).unwrap(),
            "\"skipped\""
        );
        assert_eq!(
            serde_json::from_str::<AspectStatus>("\"pass\"").unwrap(),
            AspectStatus::Pass
        );
        assert_eq!(
            serde_json::from_str::<AspectStatus>("\"fail\"").unwrap(),
            AspectStatus::Fail
        );
        assert_eq!(
            serde_json::from_str::<AspectStatus>("\"skipped\"").unwrap(),
            AspectStatus::Skipped
        );

        assert_eq!(
            serde_json::to_string(&AdvancedAspect::FailToPass).unwrap(),
            "\"fail-to-pass\""
        );
        assert_eq!(
            serde_json::to_string(&AdvancedAspect::AntiTamper).unwrap(),
            "\"anti-tamper\""
        );
        assert_eq!(
            serde_json::from_str::<AdvancedAspect>("\"mutation\"").unwrap(),
            AdvancedAspect::Mutation
        );
        assert_eq!(
            serde_json::from_str::<AdvancedAspect>("\"formal\"").unwrap(),
            AdvancedAspect::Formal
        );
    }

    #[test]
    fn loop_verdict_roundtrip_minimal() {
        let v = LoopVerdict {
            schema_version: LOOP_VERDICT_SCHEMA,
            task_id: "mp049-001".into(),
            ok: true,
            ralph_ok: true,
            policy: "fixed".into(),
            decay_action: "done".into(),
            best_of: None,
            winner_id: None,
            aspects: vec![AspectResult {
                aspect: AdvancedAspect::FailToPass,
                status: AspectStatus::Skipped,
                score: None,
                reason: Some("no_ftp_list".into()),
                exit_code: None,
                duration_ms: 0,
                stdout_tail: String::new(),
                stderr_tail: String::new(),
            }],
            failure_signature: None,
        };
        let json = serde_json::to_string(&v).unwrap();
        assert!(json.contains("\"schemaVersion\":1"));
        assert!(json.contains("\"ralphOk\":true"));
        assert!(!json.contains("bestOf"));
        let back: LoopVerdict = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}
