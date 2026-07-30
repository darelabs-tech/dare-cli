//! SHA-256 verification against `SHA256SUMS` (sha256sum two-space format).

use dare_core::{CoreError, CoreResult};
use sha2::{Digest, Sha256};

/// Frozen message when asset digest does not match `SHA256SUMS` (BLUEPRINT-053).
pub const MSG_CHECKSUM_MISMATCH: &str = "checksum mismatch for downloaded asset";

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
        // sha256sum two-space (or space-star binary mode): split on first whitespace run after hex
        let Some((hex, rest)) = split_sum_line(line) else {
            continue;
        };
        let name = rest.trim();
        // Binary mode prefix `*` is optional
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
}
