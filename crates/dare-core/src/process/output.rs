//! ProcessOutput and stream truncation.

pub const DEFAULT_STREAM_LIMIT: usize = 4000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub timed_out: bool,
    pub cancelled: bool,
}

/// Truncate to at most `limit` Unicode scalar values.
pub fn truncate_chars(s: String, limit: usize) -> (String, bool) {
    if s.chars().count() <= limit {
        (s, false)
    } else {
        (s.chars().take(limit).collect(), true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_chars_at_4000() {
        let s = "x".repeat(5000);
        let (out, truncated) = truncate_chars(s, DEFAULT_STREAM_LIMIT);
        assert!(truncated);
        assert_eq!(out.chars().count(), 4000);
    }
}
