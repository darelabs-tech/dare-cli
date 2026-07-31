//! Command-injection payloads as argv-only `SafeCommand` args (never shell).

use std::fs;
use std::path::PathBuf;

use dare_core::{CoreError, CoreResult, ProcessRunner, SafeCommand};

/// Relative path from workspace root to injection payloads fixture.
const PAYLOADS_REL: &str = "tests/security/injection/payloads.txt";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

fn load_payloads() -> CoreResult<Vec<String>> {
    let path = workspace_root().join(PAYLOADS_REL);
    let text = fs::read_to_string(&path).map_err(|e| {
        CoreError::io(format!(
            "read injection payloads {}: {e}",
            path.display()
        ))
    })?;
    let lines: Vec<String> = text
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    if lines.is_empty() {
        return Err(CoreError::invalid_input(
            "injection payloads.txt must contain at least one payload",
        ));
    }
    Ok(lines)
}

/// For each line in `payloads.txt`, build a `SafeCommand` with the payload as a
/// **single argv element** (no shell concatenation) and run it via `runner`.
///
/// Accepts `Ok` (argv spawned literally) or `InvalidInput` (path escape / denied).
/// Any other error fails the suite.
pub fn test_command_injection_payloads(runner: &dyn ProcessRunner) -> CoreResult<()> {
    let payloads = load_payloads()?;
    for payload in payloads {
        // Bare program + one arg — never `sh -c`, never string concat into program.
        let cmd = SafeCommand::new("echo").arg(payload.clone());
        if cmd.arg_list().len() != 1 || cmd.arg_list()[0] != payload {
            return Err(CoreError::internal(
                "injection test: SafeCommand must carry payload as a single argv element",
            ));
        }
        if cmd.program().contains(';')
            || cmd.program().contains('|')
            || cmd.program().contains('`')
            || cmd.program().contains('$')
        {
            return Err(CoreError::internal(
                "injection test: program must not embed shell metacharacters",
            ));
        }

        match runner.run(&cmd) {
            Ok(out) => {
                // If the runner echoed the arg, stdout must contain the literal payload
                // (shell metacharacters not expanded into extra commands).
                if !out.stdout.is_empty() && !out.stdout.contains(&payload) {
                    return Err(CoreError::guard_fail(format!(
                        "injection payload {payload:?} was not preserved as argv literal"
                    )));
                }
            }
            // Path escape / denied env / missing bare `echo` on some hosts — still no shell.
            Err(CoreError::InvalidInput(_)) | Err(CoreError::NotFound(_)) => {}
            Err(e) => {
                return Err(CoreError::internal(format!(
                    "injection payload {payload:?} unexpected error: {e}"
                )));
            }
        }
    }
    Ok(())
}
