//! Env-leak regression: secret placeholders must not appear in CLI output.

use std::path::Path;
use std::process::Command;

use dare_core::{CoreError, CoreResult};

const GITHUB_TOKEN_PROBE: &str = "ghp_TESTONLY_LEAK_PROBE";
const AI_API_KEY_PROBE: &str = "sk_TESTONLY";

/// Run `bin --help` with probe secrets in the environment and assert `secret`
/// (and the frozen placeholders) are absent from stdout/stderr.
pub fn test_env_leak_absent(bin: &Path, secret: &str) -> CoreResult<()> {
    if secret.is_empty() {
        return Err(CoreError::invalid_input(
            "env leak secret probe must be non-empty",
        ));
    }

    let output = Command::new(bin)
        .arg("--help")
        .env("GITHUB_TOKEN", GITHUB_TOKEN_PROBE)
        .env("DARE_AI_API_KEY", AI_API_KEY_PROBE)
        .output()
        .map_err(|e| CoreError::io(format!("spawn {}: {e}", bin.display())))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    for probe in [secret, GITHUB_TOKEN_PROBE, AI_API_KEY_PROBE] {
        if combined.contains(probe) {
            return Err(CoreError::guard_fail(
                "env leak: secret placeholder present in CLI output",
            ));
        }
    }

    Ok(())
}
