//! Merge semantic review JSON from agent (`--from-agent`).

use dare_core::{CoreError, CoreResult};
use serde::Deserialize;

const AGENT_MAX_BYTES: usize = 65_536;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSemantic {
    pub passed: bool,
    #[serde(default)]
    pub unmet_criteria: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

pub fn load_agent_semantic(raw: &str) -> CoreResult<AgentSemantic> {
    if raw.len() > AGENT_MAX_BYTES {
        return Err(CoreError::invalid_input(
            "from-agent file exceeds 64KiB limit",
        ));
    }
    let v: AgentSemantic = serde_json::from_str(raw)
        .map_err(|e| CoreError::invalid_input(format!("invalid from-agent JSON: {e}")))?;
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_agent_unmet_merges() {
        let s =
            load_agent_semantic(r#"{"passed":false,"unmetCriteria":["missing test"],"notes":"n"}"#)
                .unwrap();
        assert!(!s.passed);
        assert_eq!(s.unmet_criteria, vec!["missing test"]);
    }
}
