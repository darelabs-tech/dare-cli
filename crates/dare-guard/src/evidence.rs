//! Evidence redaction for guard findings.

use dare_core::redact;

const MAX_EVIDENCE: usize = 160;

/// Truncate and redact a match snippet for safe reporting.
pub fn redact_evidence(snippet: &str) -> String {
    let redacted = redact(snippet);
    let mut out: String = redacted.chars().take(MAX_EVIDENCE).collect();
    if redacted.chars().count() > MAX_EVIDENCE {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_token_in_evidence() {
        let e = redact_evidence("token=supersecret value");
        assert!(e.contains("[REDACTED]"));
        assert!(!e.contains("supersecret"));
    }
}
