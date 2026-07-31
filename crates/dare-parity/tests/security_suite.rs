//! Integration: security suite surfaces (injection / env / archive / sig / bidi).

use std::path::PathBuf;
use std::process::Command;

use assert_cmd::cargo::cargo_bin;
use dare_core::{
    CoreResult, MockProcessRunner, ProcessOutput, ProcessRunner, ProjectRoot, SafeCommand,
};
use dare_parity::{
    test_archive_traversal_fixtures, test_bidi_path_rejected, test_command_injection_payloads,
    test_env_leak_absent, test_signature_mismatch_fixtures,
};
use tempfile::tempdir;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

/// Resolve the `dare` CLI binary (assert_cmd when present; else build into CARGO_TARGET_DIR).
fn dare_bin() -> PathBuf {
    let candidate = cargo_bin("dare");
    if candidate.is_file() {
        return candidate;
    }

    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("target"));
    #[cfg(windows)]
    let bin = target_dir.join("debug").join("dare.exe");
    #[cfg(not(windows))]
    let bin = target_dir.join("debug").join("dare");

    if !bin.is_file() {
        let status = Command::new("cargo")
            .args(["build", "-p", "dare-cli", "--bin", "dare"])
            .current_dir(workspace_root())
            .status()
            .expect("spawn cargo build -p dare-cli");
        assert!(status.success(), "cargo build -p dare-cli --bin dare failed");
    }
    assert!(
        bin.is_file(),
        "dare binary missing at {}",
        bin.display()
    );
    bin
}

/// Runner that echoes the single argv payload (proves no shell concatenation).
struct ArgvEchoRunner;

impl ProcessRunner for ArgvEchoRunner {
    fn run(&self, cmd: &SafeCommand) -> CoreResult<ProcessOutput> {
        assert_eq!(cmd.program(), "echo", "injection must use bare echo program");
        assert_eq!(
            cmd.arg_list().len(),
            1,
            "payload must be a single argv element"
        );
        let payload = cmd.arg_list()[0].clone();
        Ok(ProcessOutput {
            exit_code: 0,
            stdout: payload,
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        })
    }
}

#[test]
fn security_injection_payloads_argv_only() {
    test_command_injection_payloads(&ArgvEchoRunner).expect("injection");
}

#[test]
fn security_env_leak_absent_from_help() {
    let bin = dare_bin();
    test_env_leak_absent(&bin, "ghp_TESTONLY_LEAK_PROBE").expect("github token");
    test_env_leak_absent(&bin, "sk_TESTONLY").expect("api key");
}

#[test]
fn security_archive_traversal_blocked() {
    let dir = workspace_root().join("tests/security/archives");
    test_archive_traversal_fixtures(&dir).expect("archive");
}

#[test]
fn security_signature_mismatch_and_skipped() {
    let dir = workspace_root().join("tests/security/signatures");
    test_signature_mismatch_fixtures(&dir).expect("signature");
}

#[test]
fn security_bidi_path_rejected() {
    let tmp = tempdir().expect("temp");
    let root = ProjectRoot::new(tmp.path()).expect("root");
    test_bidi_path_rejected(&root).expect("bidi");
}

#[test]
fn security_injection_also_accepts_mock_runner() {
    let mock = MockProcessRunner::new();
    // Six payloads in payloads.txt — push six echo-style successes.
    for _ in 0..6 {
        mock.push(ProcessOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        });
    }
    test_command_injection_payloads(&mock).expect("mock injection");
}
