//! Stack resolution and Ralph gate argv tables.

use std::time::Duration;

use dare_core::{CoreError, CoreResult, ProjectRoot, SafeCommand, SafeRelativePath};
use serde_json::Value;

use crate::ralph::{GateAspect, RALPH_TIMEOUT_SECS};

const CONFIG_REL: &str = "dare.config.json";
const CARGO_TOML_REL: &str = "Cargo.toml";

const KNOWN_UNIMPLEMENTED: &[&str] = &[
    "node-nestjs",
    "python-fastapi",
    "php-laravel",
    "go-gin",
    "go-stdlib",
    "react",
    "vue",
    "rust-leptos",
    "rust-leptos-csr",
    "mcp-node-ts",
];

/// Read `dare.config.json` `backend` (string); if absent and `Cargo.toml` exists → `"rust-axum"`;
/// otherwise `Err(invalid_input)` with `"unknown stack"`.
pub fn resolve_stack(root: &ProjectRoot) -> CoreResult<String> {
    if let Some(backend) = read_backend(root)? {
        let trimmed = backend.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    let cargo_rel = SafeRelativePath::new(CARGO_TOML_REL)?;
    let cargo_abs = root.resolve(&cargo_rel)?;
    if cargo_abs.as_path().is_file() {
        return Ok("rust-axum".to_string());
    }

    Err(CoreError::invalid_input("unknown stack"))
}

/// Return gate commands for an implemented stack.
///
/// - `rust-axum` / `rust` → build → test → lint (`SafeCommand`, timeout 600s)
/// - known-but-unimplemented → `Err` message contains `"not implemented"`
/// - other → `Err` message contains `"unknown stack"`
///
/// Commands are returned **without** cwd; [`crate::run_ralph`] applies
/// `cwd = ProjectRoot` (via `SafeRelativePath::new(".")`) before spawn.
pub fn gate_commands(stack: &str) -> CoreResult<Vec<(GateAspect, SafeCommand)>> {
    let id = stack.trim();
    match id {
        "rust-axum" | "rust" => Ok(rust_axum_gates()),
        other if KNOWN_UNIMPLEMENTED.contains(&other) => Err(CoreError::invalid_input(format!(
            "stack not implemented: {other}"
        ))),
        other => Err(CoreError::invalid_input(format!("unknown stack: {other}"))),
    }
}

fn rust_axum_gates() -> Vec<(GateAspect, SafeCommand)> {
    let timeout = Duration::from_secs(RALPH_TIMEOUT_SECS);
    vec![
        (
            GateAspect::Build,
            SafeCommand::new("cargo")
                .args(["build", "--workspace"])
                .timeout(timeout),
        ),
        (
            GateAspect::Test,
            SafeCommand::new("cargo")
                .args(["test", "--workspace"])
                .timeout(timeout),
        ),
        (
            GateAspect::Lint,
            SafeCommand::new("cargo")
                .args([
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--",
                    "-D",
                    "warnings",
                ])
                .timeout(timeout),
        ),
    ]
}

fn read_backend(root: &ProjectRoot) -> CoreResult<Option<String>> {
    let rel = SafeRelativePath::new(CONFIG_REL)?;
    let abs = root.resolve(&rel)?;
    if !abs.as_path().is_file() {
        return Ok(None);
    }
    let bytes =
        std::fs::read(abs.as_path().as_std_path()).map_err(|e| CoreError::io(e.to_string()))?;
    let v: Value = serde_json::from_slice(&bytes)
        .map_err(|e| CoreError::invalid_input(format!("invalid dare.config.json: {e}")))?;
    Ok(v.get("backend")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolve_stack_from_backend() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("dare.config.json"),
            r#"{"backend":"rust-axum"}"#,
        )
        .unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let stack = resolve_stack(&root).expect("stack");
        assert_eq!(stack, "rust-axum");
    }

    #[test]
    fn resolve_stack_fallback_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let stack = resolve_stack(&root).expect("stack");
        assert_eq!(stack, "rust-axum");
    }

    #[test]
    fn gate_commands_not_implemented_nestjs() {
        let err = gate_commands("node-nestjs").expect_err("not implemented");
        assert!(matches!(err, CoreError::InvalidInput(_)));
        assert!(err.to_string().contains("not implemented"), "msg={}", err);
    }

    #[test]
    fn gate_commands_rust_axum_argv() {
        let gates = gate_commands("rust-axum").expect("gates");
        assert_eq!(gates.len(), 3);
        assert_eq!(gates[0].0, GateAspect::Build);
        assert_eq!(gates[0].1.program(), "cargo");
        assert_eq!(gates[0].1.arg_list(), &["build", "--workspace"]);
        assert_eq!(gates[1].0, GateAspect::Test);
        assert_eq!(gates[1].1.arg_list(), &["test", "--workspace"]);
        assert_eq!(gates[2].0, GateAspect::Lint);
        assert_eq!(
            gates[2].1.arg_list(),
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings"
            ]
        );
    }

    #[test]
    fn gate_commands_unknown_stack() {
        let err = gate_commands("made-up-stack").expect_err("unknown");
        assert!(err.to_string().contains("unknown stack"));
    }
}
