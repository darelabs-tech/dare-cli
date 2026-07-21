//! Environment denylist (SECRET|TOKEN|KEY|PASSWORD).

const DENY_SUBSTR: &[&str] = &["SECRET", "TOKEN", "KEY", "PASSWORD"];

/// True if the env key name contains a denied substring (ASCII case-insensitive).
pub fn env_key_is_denied(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    DENY_SUBSTR.iter().any(|s| upper.contains(s))
}

/// Filter inherited environment by removing denied keys.
pub fn sanitize_env(vars: impl IntoIterator<Item = (String, String)>) -> Vec<(String, String)> {
    vars.into_iter()
        .filter(|(k, _)| !env_key_is_denied(k))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_env_strips_token_secret_key_password() {
        let cleaned = sanitize_env([
            ("API_TOKEN".into(), "x".into()),
            ("MY_SECRET".into(), "x".into()),
            ("PASSWORD".into(), "x".into()),
            ("AWS_SECRET_ACCESS_KEY".into(), "x".into()),
            ("keyring".into(), "x".into()),
            ("PATH".into(), "/bin".into()),
        ]);
        let keys: Vec<_> = cleaned.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["PATH"]);
    }

    #[test]
    fn sanitize_env_keeps_path_and_home() {
        let cleaned = sanitize_env([
            ("PATH".into(), "/usr/bin".into()),
            ("HOME".into(), "/home/u".into()),
            ("DARE_FOO".into(), "1".into()),
            ("FOO_BAR".into(), "1".into()),
        ]);
        assert_eq!(cleaned.len(), 4);
    }
}
