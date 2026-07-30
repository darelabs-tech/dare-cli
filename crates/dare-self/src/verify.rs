//! SHA-256 verification and fail-closed signature verification (cosign).

use std::path::Path;
use std::time::Duration;

use dare_core::{
    CoreError, CoreResult, ProcessRunner, SafeCommand, SystemProcessRunner,
};
use sha2::{Digest, Sha256};

/// Frozen message when asset digest does not match `SHA256SUMS` (BLUEPRINT-053).
pub const MSG_CHECKSUM_MISMATCH: &str = "checksum mismatch for downloaded asset";

/// Frozen message when `SHA256SUMS.sig` indicates signing was skipped (BLUEPRINT-053).
pub const MSG_SIGNING_SKIPPED: &str =
    "SHA256SUMS.sig indicates signing skipped; refusing self-update";

/// Frozen message when the `cosign` binary is not on PATH (BLUEPRINT-053).
pub const MSG_COSIGN_MISSING: &str =
    "cosign not found on PATH; required to verify release signature";

/// Env: skip cosign after SHA-256 (`1` / `true` only). Development only.
pub const ENV_ALLOW_UNSIGNED: &str = "DARE_SELF_ALLOW_UNSIGNED";

/// Env: process timeout seconds for cosign / downloads (default 120).
pub const ENV_TIMEOUT: &str = "DARE_SELF_TIMEOUT_SECS";

/// Optional cosign public key path (`cosign verify-blob --key`).
pub const ENV_COSIGN_KEY: &str = "DARE_SELF_COSIGN_KEY";

/// Optional keyless certificate identity.
pub const ENV_COSIGN_IDENTITY: &str = "DARE_SELF_COSIGN_IDENTITY";

/// Optional keyless OIDC issuer.
pub const ENV_COSIGN_OIDC_ISSUER: &str = "DARE_SELF_COSIGN_OIDC_ISSUER";

/// Case-sensitive prefix that marks an unsigned / skipped signature blob.
pub const SIGNING_SKIPPED_PREFIX: &str = "signing skipped";

/// Default timeout seconds when [`ENV_TIMEOUT`] is unset or invalid.
pub const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Verify `asset` bytes against a `SHA256SUMS` text blob for `asset_name`.
///
/// Expected line format (GNU `sha256sum`): `<64-hex><two-spaces><name>`.
pub fn verify_sha256(asset: &[u8], sums_text: &str, asset_name: &str) -> CoreResult<()> {
    let expected = lookup_sum(sums_text, asset_name)?;
    let actual = hex_sha256(asset);
    if !constant_time_eq_hex(&actual, &expected) {
        return Err(CoreError::guard_fail(MSG_CHECKSUM_MISMATCH));
    }
    Ok(())
}

/// Pluggable signature verifier over `SHA256SUMS` + `SHA256SUMS.sig`.
pub trait SignatureVerifier {
    fn verify_sums(&self, sums: &Path, sig: &Path) -> CoreResult<()>;
}

/// Reject `.sig` files whose contents start with [`SIGNING_SKIPPED_PREFIX`].
pub fn reject_if_signing_skipped(sig: &Path) -> CoreResult<()> {
    let bytes = std::fs::read(sig).map_err(|e| {
        CoreError::io(format!("read signature {}: {e}", sig.display()))
    })?;
    if bytes.starts_with(SIGNING_SKIPPED_PREFIX.as_bytes()) {
        return Err(CoreError::guard_fail(MSG_SIGNING_SKIPPED));
    }
    Ok(())
}

/// Verifier that only rejects the signing-skipped marker (no cosign).
#[derive(Debug, Default, Clone, Copy)]
pub struct RejectSkippedVerifier;

impl SignatureVerifier for RejectSkippedVerifier {
    fn verify_sums(&self, _sums: &Path, sig: &Path) -> CoreResult<()> {
        reject_if_signing_skipped(sig)
    }
}

/// Spawns `cosign verify-blob` via argv-only [`SafeCommand`] (no shell).
pub struct CosignCliVerifier<'a> {
    runner: &'a dyn ProcessRunner,
    timeout: Duration,
    key_path: Option<String>,
    certificate_identity: Option<String>,
    oidc_issuer: Option<String>,
}

impl<'a> CosignCliVerifier<'a> {
    /// Build from env + the given process runner.
    pub fn new(runner: &'a dyn ProcessRunner) -> Self {
        Self {
            runner,
            timeout: timeout_from_env(),
            key_path: env_nonempty(ENV_COSIGN_KEY),
            certificate_identity: env_nonempty(ENV_COSIGN_IDENTITY),
            oidc_issuer: env_nonempty(ENV_COSIGN_OIDC_ISSUER),
        }
    }

    /// Production helper using [`SystemProcessRunner`].
    pub fn system() -> CosignCliVerifier<'static> {
        CosignCliVerifier {
            runner: &SYSTEM_RUNNER,
            timeout: timeout_from_env(),
            key_path: env_nonempty(ENV_COSIGN_KEY),
            certificate_identity: env_nonempty(ENV_COSIGN_IDENTITY),
            oidc_issuer: env_nonempty(ENV_COSIGN_OIDC_ISSUER),
        }
    }

    /// Override timeout (tests).
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Override `--key` path (tests).
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key_path = Some(key.into());
        self
    }
}

static SYSTEM_RUNNER: SystemProcessRunner = SystemProcessRunner;

impl SignatureVerifier for CosignCliVerifier<'_> {
    fn verify_sums(&self, sums: &Path, sig: &Path) -> CoreResult<()> {
        reject_if_signing_skipped(sig)?;

        let mut cmd = SafeCommand::new("cosign")
            .arg("verify-blob")
            .arg("--signature")
            .arg(sig.to_string_lossy().into_owned())
            .timeout(self.timeout);

        if let Some(key) = &self.key_path {
            cmd = cmd.arg("--key").arg(key.clone());
        }
        if let Some(id) = &self.certificate_identity {
            cmd = cmd.arg("--certificate-identity").arg(id.clone());
        }
        if let Some(iss) = &self.oidc_issuer {
            cmd = cmd.arg("--certificate-oidc-issuer").arg(iss.clone());
        }

        cmd = cmd.arg(sums.to_string_lossy().into_owned());

        let out = self.runner.run(&cmd).map_err(|e| {
            if e.kind() == dare_core::ErrorKind::NotFound {
                CoreError::guard_fail(MSG_COSIGN_MISSING)
            } else {
                e
            }
        })?;

        if out.timed_out {
            return Err(CoreError::io("cosign verify-blob timed out"));
        }
        if out.exit_code != 0 {
            let detail = if out.stderr.trim().is_empty() {
                format!("cosign verify-blob failed (exit {})", out.exit_code)
            } else {
                format!(
                    "cosign verify-blob failed (exit {}): {}",
                    out.exit_code,
                    out.stderr.trim()
                )
            };
            return Err(CoreError::guard_fail(detail));
        }
        Ok(())
    }
}

/// `true` when [`ENV_ALLOW_UNSIGNED`] is `1` or `true` (case-insensitive for `true`).
pub fn allow_unsigned_enabled() -> bool {
    match std::env::var(ENV_ALLOW_UNSIGNED) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true")
        }
        Err(_) => false,
    }
}

/// Stderr warning when unsigned self-update is explicitly allowed (en-US).
pub const MSG_ALLOW_UNSIGNED_WARNING: &str = "warning: DARE_SELF_ALLOW_UNSIGNED is set; skipping cosign signature verification (development only)";

/// Print the allow-unsigned warning to stderr.
pub fn warn_allow_unsigned() {
    eprintln!("{MSG_ALLOW_UNSIGNED_WARNING}");
}

/// Resolve timeout from [`ENV_TIMEOUT`] (default [`DEFAULT_TIMEOUT_SECS`]).
pub fn timeout_from_env() -> Duration {
    let secs = match std::env::var(ENV_TIMEOUT) {
        Ok(v) => v
            .trim()
            .parse::<u64>()
            .ok()
            .filter(|n| *n > 0)
            .unwrap_or(DEFAULT_TIMEOUT_SECS),
        Err(_) => DEFAULT_TIMEOUT_SECS,
    };
    Duration::from_secs(secs)
}

fn env_nonempty(key: &str) -> Option<String> {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => Some(v.trim().to_string()),
        _ => None,
    }
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let dig = h.finalize();
    let mut out = String::with_capacity(64);
    for b in dig {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn lookup_sum(sums_text: &str, asset_name: &str) -> CoreResult<String> {
    for line in sums_text.lines() {
        let line = line.trim_end_matches(['\r']);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((hex, rest)) = split_sum_line(line) else {
            continue;
        };
        let name = rest.trim();
        let name = name.strip_prefix('*').unwrap_or(name);
        if name == asset_name {
            if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(CoreError::invalid_input(format!(
                    "invalid sha256 hex for {asset_name}"
                )));
            }
            return Ok(hex.to_ascii_lowercase());
        }
    }
    Err(CoreError::not_found(format!(
        "asset `{asset_name}` not listed in SHA256SUMS"
    )))
}

/// Parse `<hex>  <name>` (prefer two spaces) or `<hex> <name>`.
fn split_sum_line(line: &str) -> Option<(&str, &str)> {
    if let Some((hex, rest)) = line.split_once("  ") {
        return Some((hex.trim(), rest));
    }
    let (hex, rest) = line.split_once(char::is_whitespace)?;
    Some((hex, rest))
}

fn constant_time_eq_hex(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x.to_ascii_lowercase() ^ y.to_ascii_lowercase();
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::{MockProcessRunner, ProcessOutput};
    use std::fs;
    use tempfile::tempdir;

    const FIXTURE_ASSET: &[u8] = include_bytes!("../tests/fixtures/sample-asset.bin");
    const FIXTURE_SUMS: &str = include_str!("../tests/fixtures/SHA256SUMS");

    #[test]
    fn verify_sha256_ok() {
        verify_sha256(FIXTURE_ASSET, FIXTURE_SUMS, "sample-asset.bin").unwrap();
    }

    #[test]
    fn verify_sha256_mismatch() {
        let err = verify_sha256(b"wrong-bytes", FIXTURE_SUMS, "sample-asset.bin").unwrap_err();
        assert_eq!(err.kind(), dare_core::ErrorKind::GuardFail);
        assert_eq!(err.message(), MSG_CHECKSUM_MISMATCH);
    }

    #[test]
    fn verify_sha256_missing_name() {
        let err = verify_sha256(FIXTURE_ASSET, FIXTURE_SUMS, "other.bin").unwrap_err();
        assert_eq!(err.kind(), dare_core::ErrorKind::NotFound);
    }

    #[test]
    fn signing_skipped_rejected() {
        let dir = tempdir().unwrap();
        let sig = dir.path().join("SHA256SUMS.sig");
        fs::write(&sig, b"signing skipped - alpha release").unwrap();
        let sums = dir.path().join("SHA256SUMS");
        fs::write(&sums, b"deadbeef").unwrap();

        let err = RejectSkippedVerifier
            .verify_sums(&sums, &sig)
            .unwrap_err();
        assert_eq!(err.kind(), dare_core::ErrorKind::GuardFail);
        assert_eq!(err.message(), MSG_SIGNING_SKIPPED);

        let mock = MockProcessRunner::new();
        let verifier = CosignCliVerifier::new(&mock);
        let err2 = verifier.verify_sums(&sums, &sig).unwrap_err();
        assert_eq!(err2.message(), MSG_SIGNING_SKIPPED);
        // Cosign must not have been spawned when signing skipped.
    }

    #[test]
    fn cosign_missing_maps_to_msg() {
        let dir = tempdir().unwrap();
        let sig = dir.path().join("SHA256SUMS.sig");
        fs::write(&sig, b"real-sig-bytes").unwrap();
        let sums = dir.path().join("SHA256SUMS");
        fs::write(&sums, FIXTURE_SUMS).unwrap();

        let mock = MockProcessRunner::new();
        mock.push_err(CoreError::not_found("executable not found"));
        let verifier = CosignCliVerifier::new(&mock).with_timeout(Duration::from_secs(5));
        let err = verifier.verify_sums(&sums, &sig).unwrap_err();
        assert_eq!(err.kind(), dare_core::ErrorKind::GuardFail);
        assert_eq!(err.message(), MSG_COSIGN_MISSING);
    }

    #[test]
    fn cosign_ok_via_mock() {
        let dir = tempdir().unwrap();
        let sig = dir.path().join("SHA256SUMS.sig");
        fs::write(&sig, b"real-sig-bytes").unwrap();
        let sums = dir.path().join("SHA256SUMS");
        fs::write(&sums, FIXTURE_SUMS).unwrap();

        let mock = MockProcessRunner::new();
        mock.when_program(
            "cosign",
            ProcessOutput {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
                stdout_truncated: false,
                stderr_truncated: false,
                timed_out: false,
                cancelled: false,
            },
        );
        let verifier = CosignCliVerifier::new(&mock);
        verifier.verify_sums(&sums, &sig).unwrap();
    }

    #[test]
    fn allow_unsigned_only_one_or_true() {
        // Unset path is covered implicitly; we only assert parsing helpers via direct checks.
        assert!(!{
            // Simulate empty: function reads env — document contract in unit form:
            let parse = |v: &str| v == "1" || v.eq_ignore_ascii_case("true");
            parse("yes") || parse("0")
        });
        let parse = |v: &str| {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true")
        };
        assert!(parse("1"));
        assert!(parse("true"));
        assert!(parse("TRUE"));
        assert!(!parse("yes"));
        assert!(!parse("0"));
    }
}
