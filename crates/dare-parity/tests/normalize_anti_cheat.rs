//! Anti over-normalize: contract fields must still fail comparison when they differ.

use dare_parity::{normalize_text, NormalizeCtx, MSG_OVER_NORMALIZE};
use serde_json::Value;
use std::collections::BTreeSet;

/// Compares exit codes independently of normalized stdout/stderr text.
fn assert_exit_contract(exit_a: i32, exit_b: i32) -> Result<(), &'static str> {
    if exit_a != exit_b {
        return Err(MSG_OVER_NORMALIZE);
    }
    Ok(())
}

/// Compares JSON object key sets; value normalization must not hide key renames.
fn assert_json_keys_contract(json_a: &str, json_b: &str) -> Result<(), &'static str> {
    let a: Value = serde_json::from_str(json_a).map_err(|_| MSG_OVER_NORMALIZE)?;
    let b: Value = serde_json::from_str(json_b).map_err(|_| MSG_OVER_NORMALIZE)?;
    let keys = |v: &Value| -> BTreeSet<String> {
        match v {
            Value::Object(map) => map.keys().cloned().collect(),
            _ => BTreeSet::new(),
        }
    };
    if keys(&a) != keys(&b) {
        return Err(MSG_OVER_NORMALIZE);
    }
    Ok(())
}

#[test]
fn over_normalize_does_not_hide_exit_code() {
    let ctx = NormalizeCtx::default();
    let stdout_a = "done at 2026-07-31T12:00:00Z";
    let stdout_b = "done at 2024-01-01T00:00:00.999Z";

    // Volatile fields collapse — text alone would look equal.
    assert_eq!(
        normalize_text(stdout_a, &ctx),
        normalize_text(stdout_b, &ctx)
    );

    // Exit is a contract field: helpers must still report the diff.
    assert_eq!(
        assert_exit_contract(0, 1).unwrap_err(),
        MSG_OVER_NORMALIZE
    );
    assert!(assert_exit_contract(0, 0).is_ok());
}

#[test]
fn over_normalize_does_not_hide_json_key() {
    let ctx = NormalizeCtx::default();
    // Same shape after value normalize, different contract key names.
    let a = r#"{"exitCode":0,"ts":"2026-07-31T12:00:00Z"}"#;
    let b = r#"{"status":0,"ts":"2024-01-01T00:00:00Z"}"#;

    let na = normalize_text(a, &ctx);
    let nb = normalize_text(b, &ctx);
    // Values may equalize; keys must still be checked separately.
    assert!(na.contains("1970-01-01T00:00:00Z"));
    assert!(nb.contains("1970-01-01T00:00:00Z"));

    assert_eq!(
        assert_json_keys_contract(a, b).unwrap_err(),
        MSG_OVER_NORMALIZE
    );

    let same_keys_a = r#"{"exitCode":0,"ts":"2026-07-31T12:00:00Z"}"#;
    let same_keys_b = r#"{"exitCode":1,"ts":"2024-01-01T00:00:00Z"}"#;
    assert!(assert_json_keys_contract(same_keys_a, same_keys_b).is_ok());
}
