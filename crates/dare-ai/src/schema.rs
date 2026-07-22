use std::collections::BTreeMap;

use dare_core::{CoreError, CoreResult};
use serde_json::Value;

use crate::{BODY_MAX, ENRICHABLE};

pub fn parse_and_validate_sections(stdout: &str) -> CoreResult<BTreeMap<String, String>> {
    let value: Value = serde_json::from_str(stdout)
        .map_err(|_| CoreError::invalid_input("enrichment response is not JSON"))?;

    let sections = value
        .get("sections")
        .and_then(Value::as_object)
        .ok_or_else(|| CoreError::invalid_input("enrichment response missing sections object"))?;

    let mut out = BTreeMap::new();

    for id in ENRICHABLE {
        let body = sections
            .get(*id)
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::invalid_input(format!("missing enrichment section: {id}")))?;

        let trimmed = body.trim();
        if trimmed.is_empty() {
            return Err(CoreError::invalid_input(format!(
                "enrichment section {id} must not be empty"
            )));
        }
        if trimmed.len() > BODY_MAX {
            return Err(CoreError::invalid_input(format!(
                "enrichment section {id} exceeds maximum body size of {BODY_MAX} bytes"
            )));
        }
        if trimmed.contains("AGENT:BEGIN") || trimmed.contains("AGENT:END") {
            return Err(CoreError::invalid_input(format!(
                "enrichment section {id} must not contain AGENT markers"
            )));
        }

        out.insert((*id).to_string(), trimmed.to_string());
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fixture(name: &str) -> String {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("../../tests/fixtures/ai");
        p.push(name);
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
    }

    #[test]
    fn parse_valid_sections() {
        let raw = fixture("mock-sections-valid.json");
        let sections = parse_and_validate_sections(&raw).unwrap();
        assert_eq!(sections.len(), 4);
        for id in ENRICHABLE {
            assert!(sections.contains_key(*id), "missing key {id}");
        }
        assert_eq!(sections["description"], "API de pagamentos com Stripe");
    }

    #[test]
    fn parse_valid_sections_ignores_extras() {
        let mut value: Value = serde_json::from_str(&fixture("mock-sections-valid.json")).unwrap();
        value["sections"]["extra-section"] = Value::String("ignored".into());
        let raw = value.to_string();
        let sections = parse_and_validate_sections(&raw).unwrap();
        assert_eq!(sections.len(), 4);
        assert!(!sections.contains_key("extra-section"));
    }

    #[test]
    fn parse_rejects_missing_key() {
        let raw = fixture("mock-sections-missing-key.json");
        let err = parse_and_validate_sections(&raw).unwrap_err();
        assert!(err.to_string().contains("missing enrichment section"));
    }

    #[test]
    fn parse_rejects_oversize_body() {
        let raw = fixture("mock-sections-oversize.json");
        let err = parse_and_validate_sections(&raw).unwrap_err();
        assert!(err.to_string().contains("exceeds maximum body size"));
    }

    #[test]
    fn parse_rejects_nested_markers() {
        let nested = serde_json::json!({
            "sections": {
                "description": "ok",
                "objectives": "line with <!-- AGENT:BEGIN section=\"x\" -->",
                "functional-requirements": "| ID | Requisito |",
                "stack": "| Camada | Tech |"
            }
        });
        let err = parse_and_validate_sections(&nested.to_string()).unwrap_err();
        assert!(err.to_string().contains("must not contain AGENT markers"));
    }

    #[test]
    fn parse_rejects_non_json() {
        let err = parse_and_validate_sections("not json").unwrap_err();
        assert!(err.to_string().contains("not JSON"));
    }
}
