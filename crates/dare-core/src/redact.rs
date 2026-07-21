//! Secret redaction for errors and tracing (microplano 004).

/// Redacts known secret patterns. Total function; idempotent for already-redacted input.
pub fn redact(input: &str) -> String {
    let mut out = input.to_string();
    out = replace_ci_prefix_run(&out, "Bearer ", "Bearer [REDACTED]");
    out = replace_ci_prefix_run(&out, "Authorization:", "Authorization: [REDACTED]");
    out = replace_assignment(&out, &["password"], "password=[REDACTED]");
    out = replace_assignment(&out, &["api_key", "api-key"], "api_key=[REDACTED]");
    out = replace_assignment(&out, &["token"], "token=[REDACTED]");
    out = redact_akia_keys(&out);
    out
}

/// Find case-insensitive `prefix`, then consume following non-whitespace (or rest of token after colon).
fn replace_ci_prefix_run(input: &str, prefix: &str, replacement: &str) -> String {
    let needle = prefix.to_ascii_lowercase();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let lower: String = input.to_ascii_lowercase();
    let chars: Vec<char> = input.chars().collect();
    let lower_chars: Vec<char> = lower.chars().collect();

    while i < chars.len() {
        if matches_at(&lower_chars, i, &needle) {
            out.push_str(replacement);
            i += needle.chars().count();
            // skip optional spaces after Authorization:
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            while i < chars.len() && !chars[i].is_whitespace() {
                i += 1;
            }
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn matches_at(hay: &[char], start: usize, needle_lower: &str) -> bool {
    let n: Vec<char> = needle_lower.chars().collect();
    if start + n.len() > hay.len() {
        return false;
    }
    hay[start..start + n.len()] == n[..]
}

fn replace_assignment(input: &str, keys: &[&str], replacement: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let lower: Vec<char> = input.to_ascii_lowercase().chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;

    while i < chars.len() {
        let mut matched = false;
        for key in keys {
            let k: Vec<char> = key.to_ascii_lowercase().chars().collect();
            if matches_at(&lower, i, key) {
                let after = i + k.len();
                let mut j = after;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == '=' || chars[j] == ':') {
                    j += 1;
                    while j < chars.len() && chars[j] == ' ' {
                        j += 1;
                    }
                    while j < chars.len()
                        && !chars[j].is_whitespace()
                        && chars[j] != '&'
                        && chars[j] != '"'
                        && chars[j] != '\''
                    {
                        j += 1;
                    }
                    out.push_str(replacement);
                    i = j;
                    matched = true;
                    break;
                }
            }
        }
        if !matched {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn redact_akia_keys(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < chars.len() {
        if i + 20 <= chars.len() {
            let window: String = chars[i..i + 20].iter().collect();
            let ok = window.starts_with("AKIA")
                && window[4..].chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                && (i == 0 || !chars[i - 1].is_ascii_alphanumeric())
                && (i + 20 == chars.len() || !chars[i + 20].is_ascii_alphanumeric());
            if ok {
                out.push_str("[REDACTED]");
                i += 20;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_bearer_password_api_key_token_akia() {
        assert_eq!(redact(""), "");
        assert_eq!(redact("hello"), "hello");

        let s = redact(
            "Bearer secret-token-value password=hunter2 api_key=abc123 token=xyz Authorization: BasicZm9v AKIAIOSFODNN7EXAMPLE",
        );
        assert!(s.contains("Bearer [REDACTED]"), "{s}");
        assert!(s.contains("password=[REDACTED]"), "{s}");
        assert!(s.contains("api_key=[REDACTED]"), "{s}");
        assert!(s.contains("token=[REDACTED]"), "{s}");
        assert!(s.contains("Authorization: [REDACTED]"), "{s}");
        assert!(!s.contains("hunter2"), "{s}");
        assert!(!s.contains("secret-token-value"), "{s}");
        assert!(!s.contains("AKIAIOSFODNN7EXAMPLE"), "{s}");

        assert_eq!(redact(&s), s);
    }
}
