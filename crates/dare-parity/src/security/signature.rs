//! Signature mismatch / signing-skipped fixtures via `dare_self` APIs.

use std::fs;
use std::path::Path;

use dare_core::{CoreError, CoreResult};
use dare_self::{
    reject_if_signing_skipped, verify_sha256, MSG_CHECKSUM_MISMATCH, MSG_SIGNING_SKIPPED,
    RejectSkippedVerifier, SignatureVerifier,
};

const BAD_SUMS: &str = "bad-SHA256SUMS";
const SKIPPED_SIG: &str = "signing-skipped.sig";
const ASSET_NAME: &str = "dare-asset.bin";

/// Assert checksum mismatch and signing-skipped rejection against fixtures in `dir`.
pub fn test_signature_mismatch_fixtures(dir: &Path) -> CoreResult<()> {
    let sums_path = dir.join(BAD_SUMS);
    let sig_path = dir.join(SKIPPED_SIG);

    let sums_text = fs::read_to_string(&sums_path).map_err(|e| {
        CoreError::io(format!("read {}: {e}", sums_path.display()))
    })?;
    let sig_bytes = fs::read(&sig_path).map_err(|e| {
        CoreError::io(format!("read {}: {e}", sig_path.display()))
    })?;

    if !sig_bytes.starts_with(b"signing skipped") {
        return Err(CoreError::invalid_input(
            "signing-skipped.sig must start with 'signing skipped'",
        ));
    }

    // Mismatch: asset bytes that do not match the all-zero digests in bad-SHA256SUMS.
    let asset = b"not-the-checksummed-bytes";
    let err = match verify_sha256(asset, &sums_text, ASSET_NAME) {
        Ok(()) => {
            return Err(CoreError::guard_fail(
                "verify_sha256 must fail on bad-SHA256SUMS mismatch",
            ));
        }
        Err(e) => e,
    };
    if err.message() != MSG_CHECKSUM_MISMATCH {
        return Err(CoreError::guard_fail(format!(
            "expected MSG_CHECKSUM_MISMATCH, got {}",
            err.message()
        )));
    }

    // Signing skipped path.
    let err = match reject_if_signing_skipped(&sig_path) {
        Ok(()) => {
            return Err(CoreError::guard_fail(
                "reject_if_signing_skipped must fail on signing-skipped.sig",
            ));
        }
        Err(e) => e,
    };
    if err.message() != MSG_SIGNING_SKIPPED {
        return Err(CoreError::guard_fail(format!(
            "expected MSG_SIGNING_SKIPPED, got {}",
            err.message()
        )));
    }

    let err = RejectSkippedVerifier
        .verify_sums(&sums_path, &sig_path)
        .err()
        .ok_or_else(|| {
            CoreError::guard_fail("RejectSkippedVerifier must reject signing-skipped.sig")
        })?;
    if err.message() != MSG_SIGNING_SKIPPED {
        return Err(CoreError::guard_fail(format!(
            "RejectSkippedVerifier expected MSG_SIGNING_SKIPPED, got {}",
            err.message()
        )));
    }

    Ok(())
}
