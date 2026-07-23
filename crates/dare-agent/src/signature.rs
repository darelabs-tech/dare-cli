//! Failure signature hashing for agent Stop / Fail stamps.

use sha2::{Digest, Sha256};

/// Normalize stderr before hashing: lowercase ASCII, strip CSI, collapse whitespace, trim.
pub fn normalize_stderr(stderr: &str) -> String {
    let stripped = strip_csi(stderr);
    let lower: String = stripped.chars().flat_map(|c| c.to_lowercase()).collect();
    collapse_whitespace(&lower).trim().to_string()
}

/// 8 hex chars of sha256(aspect || 0x00 || normalize_stderr(stderr)).
pub fn failure_signature(aspect: &str, stderr: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(aspect.as_bytes());
    hasher.update([0u8]);
    hasher.update(normalize_stderr(stderr).as_bytes());
    let digest = hasher.finalize();
    hex8(&digest[..4])
}

fn strip_csi(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1b {
            i += 1;
            if i < bytes.len() && bytes[i] == b'[' {
                i += 1;
                while i < bytes.len() {
                    let b = bytes[i];
                    i += 1;
                    if (0x40..=0x7e).contains(&b) {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_ws = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
        } else {
            out.push(c);
            prev_ws = false;
        }
    }
    out
}

fn hex8(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(8);
    for &b in bytes.iter().take(4) {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_golden_stable() {
        let a = failure_signature("agent", "MOCK Failure\n");
        let b = failure_signature("agent", "mock   failure");
        assert_eq!(a, b);
        assert_eq!(a.len(), 8);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        // CSI strip should not change signature of equivalent text
        let with_csi = failure_signature("agent", "\x1b[31mmock failure\x1b[0m");
        assert_eq!(a, with_csi);
    }

    #[test]
    fn normalize_collapses_and_lowers() {
        assert_eq!(normalize_stderr("  Foo\tBAR  "), "foo bar");
    }
}
