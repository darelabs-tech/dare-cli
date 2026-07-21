//! Core types, errors, redaction, context, and tracing helpers for the DARE CLI workspace.

mod context;
mod error;
mod redact;
mod telemetry;

pub use context::{ColorMode, ExecutionContext};
pub use error::{exit_code, CoreError, CoreResult, ErrorKind};
pub use redact::redact;
pub use telemetry::{init_test_subscriber, init_tracing};

use serde_json::{Map, Value};

/// Valida que `name` não é vazio e não contém NUL.
pub fn validate_nonempty_name(name: &str) -> CoreResult<()> {
    if name.is_empty() {
        return Err(CoreError::invalid_input("name must not be empty"));
    }
    if name.contains('\0') {
        return Err(CoreError::invalid_input("name must not contain NUL"));
    }
    Ok(())
}

/// Serialize JSON with lexicographic key order at every object (ADR-002).
pub fn to_canonical_json_string(value: &Value) -> CoreResult<String> {
    let sorted = sort_value(value);
    serde_json::to_string(&sorted).map_err(|e| CoreError::internal(e.to_string()))
}

fn sort_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = Map::new();
            for k in keys {
                out.insert(k.clone(), sort_value(&map[k]));
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(sort_value).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validate_nonempty_name_ok() {
        assert_eq!(validate_nonempty_name("dare"), Ok(()));
    }

    #[test]
    fn validate_nonempty_name_empty_err() {
        assert!(matches!(
            validate_nonempty_name(""),
            Err(CoreError::InvalidInput(_))
        ));
    }

    #[test]
    fn validate_nonempty_name_nul_err() {
        assert!(matches!(
            validate_nonempty_name("a\0b"),
            Err(CoreError::InvalidInput(_))
        ));
    }

    #[test]
    fn canonical_json_sorts_object_keys_recursively() {
        let v = json!({"ok": true, "a": 1, "nested": {"z": 1, "m": 2}});
        let s = to_canonical_json_string(&v).expect("serialize");
        assert!(!s.contains('\u{1b}'), "{s}");
        // "a" before "nested" before "ok"; nested "m" before "z"
        assert_eq!(s, r#"{"a":1,"nested":{"m":2,"z":1},"ok":true}"#);
    }
}
