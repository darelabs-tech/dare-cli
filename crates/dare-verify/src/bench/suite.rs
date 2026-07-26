//! Load and validate `suite.json` + case directories.

use std::fs;
use std::path::{Path, PathBuf};

use dare_core::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

/// Usage message prefix: `invalid bench suite: {reason}`.
pub const MSG_SUITE_INVALID: &str = "invalid bench suite";

fn suite_invalid(reason: impl AsRef<str>) -> CoreError {
    CoreError::usage(format!("{MSG_SUITE_INVALID}: {}", reason.as_ref()))
}

/// On-disk `suite.json` (`schemaVersion` 1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SuiteFile {
    pub schema_version: u32,
    pub name: String,
    pub cases: Vec<SuiteCase>,
}

/// One case entry in `suite.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SuiteCase {
    pub id: String,
    pub path: String,
}

/// Case directory validated against required artefacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedCase {
    pub id: String,
    /// Absolute path to the case directory.
    pub dir: PathBuf,
    /// Relative path as listed in suite.json.
    pub rel_path: String,
    pub stack: String,
}

/// Loaded suite with validated case dirs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSuite {
    pub name: String,
    pub suite_dir: PathBuf,
    pub cases: Vec<LoadedCase>,
}

const REQUIRED_FILES: &[&str] = &[
    "patch.diff",
    "fail_to_pass.txt",
    "pass_to_pass.txt",
];

/// Load `suite.json` from `suite_dir` and validate each case directory.
pub fn load_suite(suite_dir: &Path) -> CoreResult<LoadedSuite> {
    let suite_json = suite_dir.join("suite.json");
    if !suite_json.is_file() {
        return Err(suite_invalid(format!(
            "missing suite.json under {}",
            suite_dir.display()
        )));
    }
    let bytes = fs::read(&suite_json).map_err(|e| CoreError::io(e.to_string()))?;
    let file: SuiteFile = serde_json::from_slice(&bytes)
        .map_err(|e| suite_invalid(format!("malformed suite.json: {e}")))?;

    if file.schema_version != 1 {
        return Err(suite_invalid(format!(
            "unsupported schemaVersion {}",
            file.schema_version
        )));
    }
    if file.cases.is_empty() {
        return Err(suite_invalid("cases must not be empty"));
    }

    let mut cases = Vec::with_capacity(file.cases.len());
    for entry in &file.cases {
        if entry.id.trim().is_empty() {
            return Err(suite_invalid("case id must not be empty"));
        }
        if entry.path.trim().is_empty() {
            return Err(suite_invalid(format!(
                "case `{}` path must not be empty",
                entry.id
            )));
        }
        let case_dir = suite_dir.join(&entry.path);
        validate_case_dir(&entry.id, &case_dir)?;
        let stack = read_stack(&case_dir)?;
        cases.push(LoadedCase {
            id: entry.id.clone(),
            dir: case_dir,
            rel_path: entry.path.clone(),
            stack,
        });
    }

    Ok(LoadedSuite {
        name: file.name,
        suite_dir: suite_dir.to_path_buf(),
        cases,
    })
}

fn validate_case_dir(id: &str, case_dir: &Path) -> CoreResult<()> {
    if !case_dir.is_dir() {
        return Err(suite_invalid(format!(
            "case `{id}` directory missing: {}",
            case_dir.display()
        )));
    }
    for name in REQUIRED_FILES {
        let p = case_dir.join(name);
        if !p.is_file() {
            return Err(suite_invalid(format!(
                "case `{id}` missing required file `{name}`"
            )));
        }
    }
    let repo = case_dir.join("repo");
    if !repo.is_dir() {
        return Err(suite_invalid(format!(
            "case `{id}` missing required directory `repo/`"
        )));
    }
    Ok(())
}

fn read_stack(case_dir: &Path) -> CoreResult<String> {
    let stack_path = case_dir.join("stack.txt");
    if !stack_path.is_file() {
        return Ok("rust-axum".to_string());
    }
    let text = fs::read_to_string(&stack_path).map_err(|e| CoreError::io(e.to_string()))?;
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("rust-axum");
    Ok(line.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace root")
    }

    #[test]
    fn suite_loads_sample_ok_fixture() {
        let suite_dir = workspace_root().join("fixtures/bench");
        let loaded = load_suite(&suite_dir).expect("load fixtures/bench");
        assert_eq!(loaded.name, "dare-bench-default");
        assert_eq!(loaded.cases.len(), 1);
        assert_eq!(loaded.cases[0].id, "sample-ok");
        assert!(loaded.cases[0].dir.join("repo").is_dir());
        assert_eq!(loaded.cases[0].stack, "rust-axum");
    }

    #[test]
    fn suite_invalid_missing_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let suite_dir = tmp.path();
        fs::write(
            suite_dir.join("suite.json"),
            r#"{"schemaVersion":1,"name":"t","cases":[{"id":"bad","path":"cases/bad"}]}"#,
        )
        .expect("write suite");
        let case_dir = suite_dir.join("cases/bad");
        fs::create_dir_all(case_dir.join("repo")).expect("mkdir");
        fs::write(case_dir.join("patch.diff"), "").expect("patch");
        fs::write(case_dir.join("fail_to_pass.txt"), "a\n").expect("ftp");
        // intentionally omit pass_to_pass.txt

        let err = load_suite(suite_dir).expect_err("must fail");
        assert!(matches!(err, CoreError::Usage(_)));
        let msg = err.message();
        assert!(
            msg.starts_with("invalid bench suite:"),
            "msg={msg}"
        );
        assert!(msg.contains("pass_to_pass.txt"), "msg={msg}");
    }
}
