//! Basename deny for `.env` / `.env.*` steering targets.

/// Returns true when `basename` is `.env` or starts with `.env.` (case-sensitive).
/// `.envrc` is NOT excluded.
pub fn is_env_excluded(basename: &str) -> bool {
    basename == ".env" || basename.starts_with(".env.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_deny_dot_env() {
        assert!(is_env_excluded(".env"));
        assert!(!is_env_excluded(".envrc"));
        assert!(!is_env_excluded("env"));
        assert!(!is_env_excluded(".ENV"));
    }

    #[test]
    fn env_deny_dot_env_local() {
        assert!(is_env_excluded(".env.local"));
        assert!(is_env_excluded(".env.production"));
        assert!(!is_env_excluded(".environment"));
    }
}
