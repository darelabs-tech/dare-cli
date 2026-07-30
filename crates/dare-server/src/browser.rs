//! Open local dashboard URL via SafeCommand (argv only — RS-06).

use dare_core::{CoreError, CoreResult, ProcessRunner, SafeCommand};

/// Allowlist: `^https?://(127\.0\.0\.1|localhost)(:\d+)?(/.*)?$`
pub fn is_allowed_browser_url(url: &str) -> bool {
    let rest = match url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")) {
        Some(r) => r,
        None => return false,
    };

    let after_host = if let Some(r) = rest.strip_prefix("127.0.0.1") {
        r
    } else if let Some(r) = rest.strip_prefix("localhost") {
        r
    } else {
        return false;
    };

    let path = if let Some(r) = after_host.strip_prefix(':') {
        let digits = r.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return false;
        }
        &r[digits..]
    } else {
        after_host
    };

    path.is_empty() || path.starts_with('/')
}

/// Open `url` in the system browser via argv-only [`SafeCommand`].
///
/// Rejects non-allowlisted URLs with [`CoreError::invalid_input`].
pub fn open_browser(url: &str, runner: &dyn ProcessRunner) -> CoreResult<()> {
    if !is_allowed_browser_url(url) {
        return Err(CoreError::invalid_input(format!(
            "browser URL not allowed (must be http(s)://127.0.0.1|localhost): {url}"
        )));
    }

    let cmd = browser_command(url);
    let out = runner.run(&cmd)?;
    if out.exit_code != 0 {
        return Err(CoreError::io(format!(
            "browser open failed with exit {}",
            out.exit_code
        )));
    }
    Ok(())
}

fn browser_command(url: &str) -> SafeCommand {
    #[cfg(windows)]
    {
        SafeCommand::new("cmd.exe").args(["/C", "start", "", url])
    }
    #[cfg(target_os = "macos")]
    {
        SafeCommand::new("open").arg(url)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        SafeCommand::new("xdg-open").arg(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::{MockProcessRunner, ProcessOutput};

    fn ok_out() -> ProcessOutput {
        ProcessOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            cancelled: false,
        }
    }

    #[test]
    fn url_reject_evil() {
        let mock = MockProcessRunner::new();
        mock.push(ok_out());
        let err = open_browser("https://evil.example/phish", &mock).expect_err("evil");
        assert!(matches!(err, CoreError::InvalidInput(_)));
        assert!(err.to_string().contains("not allowed"));

        let err = open_browser("http://127.0.0.1.evil.com/", &mock).expect_err("suffix");
        assert!(matches!(err, CoreError::InvalidInput(_)));

        let err = open_browser("file:///etc/passwd", &mock).expect_err("file");
        assert!(matches!(err, CoreError::InvalidInput(_)));

        assert!(!is_allowed_browser_url("http://192.168.1.1/dashboard"));
        assert!(!is_allowed_browser_url("http://localhost:/dashboard"));
    }

    #[test]
    fn open_browser_mock_ok() {
        let mock = MockProcessRunner::new();
        mock.push(ok_out());
        open_browser("http://127.0.0.1:4100/dashboard", &mock).expect("open");

        let mock = MockProcessRunner::new();
        mock.push(ok_out());
        open_browser("https://localhost/dashboard", &mock).expect("localhost");
    }

    #[test]
    fn allowlist_accepts_loopback_shapes() {
        assert!(is_allowed_browser_url("http://127.0.0.1"));
        assert!(is_allowed_browser_url("http://127.0.0.1:4100"));
        assert!(is_allowed_browser_url("http://127.0.0.1:4100/dashboard"));
        assert!(is_allowed_browser_url("https://localhost"));
        assert!(is_allowed_browser_url("https://localhost:3000/x"));
    }
}
