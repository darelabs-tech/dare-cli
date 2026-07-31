//! Golden case YAML (`schemaVersion` 1): load + validate.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use dare_core::{CoreError, CoreResult};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::axis::CompareAxis;

/// Required `schemaVersion` for `case.yaml`.
pub const CASE_SCHEMA_VERSION: u32 = 1;

/// Error when a golden skip omits classification.
pub const MSG_SKIP_NEEDS_CLASS: &str = "golden skip requires class A|B|C|D";

const CASE_FILE: &str = "case.yaml";

fn case_id_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^[a-z0-9]+(\.[a-z0-9_-]+)+$").expect("case id regex compiles")
    })
}

/// Intentional-diff classification (A|B|C|D).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiffClass {
    A,
    B,
    C,
    D,
}

/// Skip metadata for a golden case (must include `class`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkipSpec {
    pub reason: String,
    pub class: DiffClass,
    pub adr_ref: Option<String>,
}

/// Expected file content mapping (`rel` path under cwd → snapshot file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentExpect {
    pub rel: String,
    pub file: PathBuf,
}

/// Expected HTTP probe for the `http` axis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpExpect {
    pub method: String,
    pub path: String,
    pub status: u16,
    pub body_file: Option<PathBuf>,
}

/// Loaded golden case specification (`schemaVersion` 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseSpec {
    pub schema_version: u32,
    pub id: String,
    pub command: Vec<String>,
    pub cwd_fixture: Option<String>,
    pub env: BTreeMap<String, String>,
    pub axes: Vec<CompareAxis>,
    pub expected_exit: Option<i32>,
    pub expected_stdout_path: Option<PathBuf>,
    pub expected_stderr_path: Option<PathBuf>,
    pub expected_tree_path: Option<PathBuf>,
    pub expected_content: Vec<ContentExpect>,
    pub expected_state_path: Option<PathBuf>,
    pub expected_http: Option<HttpExpect>,
    pub skip: Option<SkipSpec>,
}

#[derive(Debug, Deserialize)]
struct RawCase {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    id: String,
    command: Vec<String>,
    #[serde(default)]
    cwd_fixture: Option<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    axes: Vec<CompareAxis>,
    #[serde(default)]
    expected: Option<RawExpected>,
    #[serde(default)]
    skip: Option<RawSkip>,
}

#[derive(Debug, Deserialize)]
struct RawExpected {
    #[serde(default)]
    exit: Option<i32>,
    #[serde(default)]
    stdout_file: Option<PathBuf>,
    #[serde(default)]
    stderr_file: Option<PathBuf>,
    #[serde(default)]
    tree_file: Option<PathBuf>,
    #[serde(default)]
    content: Vec<RawContentExpect>,
    #[serde(default)]
    state_file: Option<PathBuf>,
    #[serde(default)]
    http: Option<RawHttpExpect>,
}

#[derive(Debug, Deserialize)]
struct RawContentExpect {
    rel: String,
    file: PathBuf,
}

#[derive(Debug, Deserialize)]
struct RawHttpExpect {
    method: String,
    path: String,
    status: u16,
    #[serde(default)]
    body_file: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct RawSkip {
    reason: String,
    #[serde(default)]
    class: Option<DiffClass>,
    #[serde(default)]
    adr_ref: Option<String>,
}

impl CaseSpec {
    /// Parse case YAML bytes without panicking on malformed input.
    pub fn try_from_yaml_bytes(bytes: &[u8]) -> CoreResult<Self> {
        let text = std::str::from_utf8(bytes)
            .map_err(|e| CoreError::invalid_input(format!("case.yaml is not utf-8: {e}")))?;
        Self::try_from_yaml_str(text)
    }

    /// Parse case YAML text into a [`CaseSpec`] (does not run full validate).
    pub fn try_from_yaml_str(text: &str) -> CoreResult<Self> {
        let raw: RawCase = serde_yaml::from_str(text)
            .map_err(|e| CoreError::invalid_input(format!("invalid case.yaml: {e}")))?;
        Self::from_raw(raw)
    }

    fn from_raw(raw: RawCase) -> CoreResult<Self> {
        let skip = match raw.skip {
            None => None,
            Some(s) => {
                let class = s
                    .class
                    .ok_or_else(|| CoreError::invalid_input(MSG_SKIP_NEEDS_CLASS))?;
                if s.reason.trim().is_empty() {
                    return Err(CoreError::invalid_input("skip.reason must be non-empty"));
                }
                Some(SkipSpec {
                    reason: s.reason,
                    class,
                    adr_ref: s.adr_ref,
                })
            }
        };

        let expected = raw.expected.unwrap_or(RawExpected {
            exit: None,
            stdout_file: None,
            stderr_file: None,
            tree_file: None,
            content: Vec::new(),
            state_file: None,
            http: None,
        });

        Ok(Self {
            schema_version: raw.schema_version,
            id: raw.id,
            command: raw.command,
            cwd_fixture: raw.cwd_fixture,
            env: raw.env,
            axes: raw.axes,
            expected_exit: expected.exit,
            expected_stdout_path: expected.stdout_file,
            expected_stderr_path: expected.stderr_file,
            expected_tree_path: expected.tree_file,
            expected_content: expected
                .content
                .into_iter()
                .map(|c| ContentExpect {
                    rel: c.rel,
                    file: c.file,
                })
                .collect(),
            expected_state_path: expected.state_file,
            expected_http: expected.http.map(|h| HttpExpect {
                method: h.method,
                path: h.path,
                status: h.status,
                body_file: h.body_file,
            }),
            skip,
        })
    }
}

/// Read `case_dir/case.yaml`, parse, and validate.
pub fn load_case(case_dir: &Path) -> CoreResult<CaseSpec> {
    let path = case_dir.join(CASE_FILE);
    let bytes = std::fs::read(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CoreError::not_found(format!("missing {}", path.display()))
        } else {
            CoreError::io(format!("read {}: {e}", path.display()))
        }
    })?;
    let spec = CaseSpec::try_from_yaml_bytes(&bytes)?;
    validate_case(&spec)?;
    Ok(spec)
}

/// Validate schema, id, command, axes, and skip classification.
pub fn validate_case(spec: &CaseSpec) -> CoreResult<()> {
    if spec.schema_version != CASE_SCHEMA_VERSION {
        return Err(CoreError::invalid_input(format!(
            "case schemaVersion must be {CASE_SCHEMA_VERSION}, got {}",
            spec.schema_version
        )));
    }

    if !case_id_re().is_match(&spec.id) {
        return Err(CoreError::invalid_input(format!(
            "invalid case id {:?}; expected ^[a-z0-9]+(\\.[a-z0-9_-]+)+$",
            spec.id
        )));
    }

    if spec.command.is_empty() {
        return Err(CoreError::invalid_input("case command must be non-empty"));
    }
    for arg in &spec.command {
        if arg.contains('\0') {
            return Err(CoreError::invalid_input(
                "case command elements must not contain NUL",
            ));
        }
    }

    if spec.axes.is_empty() {
        return Err(CoreError::invalid_input("case axes must be non-empty"));
    }
    let mut seen = BTreeSet::new();
    for axis in &spec.axes {
        if !seen.insert(*axis) {
            return Err(CoreError::invalid_input(format!(
                "duplicate compare axis: {axis:?}"
            )));
        }
    }

    if let Some(skip) = &spec.skip {
        if skip.reason.trim().is_empty() {
            return Err(CoreError::invalid_input("skip.reason must be non-empty"));
        }
        // DiffClass is closed; presence is enforced at parse. Class C ADR checks
        // happen when DiffLogIndex is available (runner phase).
        let _ = skip.class;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::CoreError;
    use std::fs;
    use tempfile::tempdir;

    fn write_case(dir: &Path, yaml: &str) {
        fs::write(dir.join(CASE_FILE), yaml).expect("write case.yaml");
    }

    #[test]
    fn case_rejects_bad_schema_version() {
        let dir = tempdir().expect("tempdir");
        write_case(
            dir.path(),
            r#"
schemaVersion: 99
id: golden.welcome.help
command: ["welcome", "--help"]
axes: [exit]
expected:
  exit: 0
"#,
        );
        let err = load_case(dir.path()).expect_err("schema 99 must fail");
        assert!(
            matches!(err, CoreError::InvalidInput(ref m) if m.contains("schemaVersion")),
            "{err:?}"
        );

        let mut spec = CaseSpec {
            schema_version: 2,
            id: "golden.welcome.help".into(),
            command: vec!["welcome".into()],
            cwd_fixture: None,
            env: BTreeMap::new(),
            axes: vec![CompareAxis::Exit],
            expected_exit: Some(0),
            expected_stdout_path: None,
            expected_stderr_path: None,
            expected_tree_path: None,
            expected_content: vec![],
            expected_state_path: None,
            expected_http: None,
            skip: None,
        };
        let err = validate_case(&spec).expect_err("bad schema");
        assert!(matches!(err, CoreError::InvalidInput(_)));
        spec.schema_version = CASE_SCHEMA_VERSION;
        validate_case(&spec).expect("schema 1 ok");
    }

    #[test]
    fn skip_without_class_fails_validate() {
        let dir = tempdir().expect("tempdir");
        write_case(
            dir.path(),
            r#"
schemaVersion: 1
id: golden.welcome.help
command: ["welcome", "--help"]
axes: [exit]
skip:
  reason: "fixture not materialized yet"
"#,
        );
        let err = load_case(dir.path()).expect_err("skip without class must fail");
        match err {
            CoreError::InvalidInput(m) => assert_eq!(m, MSG_SKIP_NEEDS_CLASS),
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn load_valid_case_ok() {
        let dir = tempdir().expect("tempdir");
        write_case(
            dir.path(),
            r#"
schemaVersion: 1
id: golden.welcome.help
command: ["welcome", "--help"]
cwd_fixture: empty-project
env: {}
axes: [exit, stdout]
expected:
  exit: 0
  stdout_file: expected/stdout.txt
skip:
  reason: "not ready"
  class: C
  adr_ref: ADR-001
"#,
        );
        let spec = load_case(dir.path()).expect("valid case");
        assert_eq!(spec.schema_version, 1);
        assert_eq!(spec.id, "golden.welcome.help");
        assert_eq!(spec.axes, vec![CompareAxis::Exit, CompareAxis::Stdout]);
        assert_eq!(spec.expected_exit, Some(0));
        assert_eq!(
            spec.skip.as_ref().map(|s| s.class),
            Some(DiffClass::C)
        );
    }
}
