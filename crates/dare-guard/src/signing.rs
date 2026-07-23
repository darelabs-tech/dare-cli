//! Ed25519 sign / verify for control artifacts.

use std::path::Path;

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use dare_core::{CoreError, CoreResult};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

pub const SIG_EXT: &str = ".minisig";
const HEADER: &str = "untrusted comment: dare-guard ed25519";

fn signing_key_from_hex(hex: &str) -> CoreResult<SigningKey> {
    let bytes = decode_hex32(hex)?;
    Ok(SigningKey::from_bytes(&bytes))
}

fn verifying_key_from_hex(hex: &str) -> CoreResult<VerifyingKey> {
    let bytes = decode_hex32(hex)?;
    VerifyingKey::from_bytes(&bytes)
        .map_err(|e| CoreError::config(format!("invalid public key: {e}")))
}

fn decode_hex32(hex: &str) -> CoreResult<[u8; 32]> {
    let hex = hex.trim();
    if hex.len() != 64 {
        return Err(CoreError::invalid_input(
            "key must be 64 hex chars (32 bytes)",
        ));
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        let byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|_| CoreError::invalid_input("key hex decode failed"))?;
        out[i] = byte;
    }
    Ok(out)
}

fn content_hash(bytes: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().into()
}

/// Sign file contents; write `<path>.minisig`.
pub fn sign_file(path: &Path, private_key_hex: &str) -> CoreResult<()> {
    let bytes =
        std::fs::read(path).map_err(|e| CoreError::io(format!("read {}: {e}", path.display())))?;
    let sk = signing_key_from_hex(private_key_hex)?;
    let digest = content_hash(&bytes);
    let sig = sk.sign(&digest);
    let body = format!("{HEADER}\n{}\n", B64.encode(sig.to_bytes()));
    let sig_path = sig_path_for(path);
    std::fs::write(&sig_path, body)
        .map_err(|e| CoreError::io(format!("write {}: {e}", sig_path.display())))?;
    Ok(())
}

/// Verify `<path>.minisig` against public key hex.
pub fn verify_file(path: &Path, public_key_hex: &str) -> CoreResult<()> {
    let bytes =
        std::fs::read(path).map_err(|e| CoreError::io(format!("read {}: {e}", path.display())))?;
    let sig_path = sig_path_for(path);
    let sig_text = std::fs::read_to_string(&sig_path)
        .map_err(|_| CoreError::guard_fail(format!("missing signature: {}", sig_path.display())))?;
    let sig_b64 = parse_sig_body(&sig_text)?;
    let sig_bytes = B64
        .decode(sig_b64.trim())
        .map_err(|_| CoreError::guard_fail("invalid signature encoding"))?;
    if sig_bytes.len() != 64 {
        return Err(CoreError::guard_fail("invalid signature length"));
    }
    let mut arr = [0u8; 64];
    arr.copy_from_slice(&sig_bytes);
    let sig = Signature::from_bytes(&arr);
    let vk = verifying_key_from_hex(public_key_hex)?;
    let digest = content_hash(&bytes);
    vk.verify(&digest, &sig)
        .map_err(|_| CoreError::guard_fail("signature verification failed"))?;
    Ok(())
}

pub fn sig_path_for(path: &Path) -> std::path::PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(SIG_EXT);
    std::path::PathBuf::from(s)
}

fn parse_sig_body(text: &str) -> CoreResult<&str> {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let first = lines
        .next()
        .ok_or_else(|| CoreError::guard_fail("empty signature file"))?;
    if first.starts_with("untrusted comment:") {
        lines
            .next()
            .ok_or_else(|| CoreError::guard_fail("signature body missing"))
    } else {
        Ok(first)
    }
}

/// Derive public key hex from private key hex (for tests / docs).
pub fn public_key_hex_from_private(private_key_hex: &str) -> CoreResult<String> {
    let sk = signing_key_from_hex(private_key_hex)?;
    let pk = sk.verifying_key();
    Ok(hex_encode(pk.as_bytes()))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn sign_verify_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("control.md");
        std::fs::write(&path, b"hello control").unwrap();
        // Fixed test seed (not a real secret).
        let sk = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        sign_file(&path, sk).unwrap();
        let pk = public_key_hex_from_private(sk).unwrap();
        verify_file(&path, &pk).unwrap();
    }

    #[test]
    fn invalid_sig_fails() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("control.md");
        std::fs::write(&path, b"hello").unwrap();
        let sk = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        sign_file(&path, sk).unwrap();
        std::fs::write(&path, b"tampered").unwrap();
        let pk = public_key_hex_from_private(sk).unwrap();
        assert!(verify_file(&path, &pk).is_err());
    }
}
