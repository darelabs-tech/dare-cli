//! Text normalizer for golden parity (allowlist N-01..N-08 only).

use std::path::PathBuf;
use std::sync::OnceLock;

use regex::Regex;

/// Error / assertion message when over-normalization would hide contract changes.
pub const MSG_OVER_NORMALIZE: &str = "normalizer must not hide contract field changes";

const EPOCH_TS: &str = "1970-01-01T00:00:00Z";
const NIL_UUID_V4: &str = "00000000-0000-4000-8000-000000000000";

/// Context for path/version-aware normalization.
#[derive(Debug, Clone, Default)]
pub struct NormalizeCtx {
    pub temp_prefixes: Vec<PathBuf>,
    pub binary_version: Option<String>,
}

/// Apply the closed allowlist N-01..N-08. Does **not** touch exit codes, flag names, or JSON keys.
pub fn normalize_text(input: &str, ctx: &NormalizeCtx) -> String {
    let mut s = input.to_string();

    // N-04 — strip ANSI CSI before other pattern matches
    s = strip_ansi_csi(&s);

    // N-01 — ISO-8601 timestamps
    s = iso8601_re().replace_all(&s, EPOCH_TS).into_owned();

    // N-02 — UUID v4
    s = uuid_v4_re().replace_all(&s, NIL_UUID_V4).into_owned();

    // N-05 — path separators (enables stable prefix / drive matching)
    s = s.replace('\\', "/");

    // N-03 — temp prefixes → `$TMP/`
    s = replace_temp_prefixes(&s, &ctx.temp_prefixes);

    // N-06 — drive letter → `$DRIVE:`
    s = normalize_drive_letters(&s);

    // N-07 — binary version → `$VERSION`
    if let Some(ver) = ctx.binary_version.as_deref() {
        if !ver.is_empty() {
            s = s.replace(ver, "$VERSION");
        }
    }

    // N-08 — dare_core::redact, then unify placeholder to `$REDACTED`
    s = dare_core::redact(&s);
    s = s.replace("[REDACTED]", "$REDACTED");

    s
}

fn strip_ansi_csi(input: &str) -> String {
    ansi_csi_re().replace_all(input, "").into_owned()
}

fn replace_temp_prefixes(input: &str, prefixes: &[PathBuf]) -> String {
    let mut norms: Vec<String> = prefixes
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .map(|p| p.trim_end_matches('/').to_string())
        .filter(|p| !p.is_empty())
        .collect();
    norms.sort_by_key(|p| std::cmp::Reverse(p.len()));
    norms.dedup();

    let mut out = input.to_string();
    for prefix in &norms {
        out = replace_prefix_occurrences(&out, prefix, "$TMP/");
    }
    out
}

fn replace_prefix_occurrences(hay: &str, prefix: &str, with: &str) -> String {
    let mut out = String::with_capacity(hay.len());
    let mut rest = hay;
    while let Some(idx) = rest.find(prefix) {
        out.push_str(&rest[..idx]);
        out.push_str(with);
        rest = &rest[idx + prefix.len()..];
        if rest.starts_with('/') {
            rest = &rest[1..];
        }
    }
    out.push_str(rest);
    out
}

fn iso8601_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})")
            .expect("iso8601 regex compiles")
    })
}

fn uuid_v4_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}",
        )
        .expect("uuid v4 regex compiles")
    })
}

fn ansi_csi_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\x1b\[[0-9;]*m").expect("ansi csi regex compiles"))
}

fn normalize_drive_letters(input: &str) -> String {
    // Single letter + colon, not mid-identifier (avoids `http:` / `https:`).
    // Default `regex` crate has no lookbehind — keep the boundary in the capture.
    drive_re()
        .replace_all(input, |caps: &regex::Captures| {
            format!("{}$DRIVE:", &caps[1])
        })
        .into_owned()
}

fn drive_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(^|[^A-Za-z0-9])[A-Za-z]:").expect("drive regex compiles")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn empty_ctx() -> NormalizeCtx {
        NormalizeCtx::default()
    }

    #[test]
    fn n01_iso8601_timestamps() {
        let input = "ran at 2026-07-31T12:00:00Z ok";
        let out = normalize_text(input, &empty_ctx());
        assert_eq!(out, "ran at 1970-01-01T00:00:00Z ok");
        assert_eq!(
            normalize_text("2026-07-31T12:00:00.123+00:00", &empty_ctx()),
            EPOCH_TS
        );
    }

    #[test]
    fn n02_uuid_v4() {
        let input = "id=550e8400-e29b-41d4-a716-446655440000";
        let out = normalize_text(input, &empty_ctx());
        assert_eq!(out, format!("id={NIL_UUID_V4}"));
    }

    #[test]
    fn n03_temp_prefixes() {
        let ctx = NormalizeCtx {
            temp_prefixes: vec![PathBuf::from("/tmp/run-abc")],
            binary_version: None,
        };
        let out = normalize_text("/tmp/run-abc/out.txt", &ctx);
        assert_eq!(out, "$TMP/out.txt");
    }

    #[test]
    fn n04_strip_ansi_csi() {
        let input = "\x1b[31mred\x1b[0m plain";
        let out = normalize_text(input, &empty_ctx());
        assert_eq!(out, "red plain");
    }

    #[test]
    fn n05_backslash_to_slash() {
        let out = normalize_text(r"foo\bar\baz", &empty_ctx());
        assert_eq!(out, "foo/bar/baz");
    }

    #[test]
    fn n06_drive_letter() {
        let out = normalize_text(r"C:\Users\x\file.txt", &empty_ctx());
        assert_eq!(out, "$DRIVE:/Users/x/file.txt");
        let out2 = normalize_text("x:/tmp/a", &empty_ctx());
        assert_eq!(out2, "$DRIVE:/tmp/a");
        // Must not rewrite scheme colons
        assert!(normalize_text("https://example.com", &empty_ctx()).contains("https://"));
    }

    #[test]
    fn n07_binary_version() {
        let ctx = NormalizeCtx {
            temp_prefixes: vec![],
            binary_version: Some("0.1.0-alpha.0".into()),
        };
        let out = normalize_text("dare 0.1.0-alpha.0", &ctx);
        assert_eq!(out, "dare $VERSION");
    }

    #[test]
    fn n08_redact_to_dollar_redacted() {
        let out = normalize_text("password=hunter2 token=xyz", &empty_ctx());
        assert!(out.contains("password=$REDACTED"), "{out}");
        assert!(out.contains("token=$REDACTED"), "{out}");
        assert!(!out.contains("hunter2"), "{out}");
        assert!(!out.contains("xyz"), "{out}");
        assert!(!out.contains("[REDACTED]"), "{out}");
    }
}
