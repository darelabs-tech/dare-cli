//! SystemProcessRunner — argv-only spawn with timeout/cancel.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use camino::Utf8PathBuf;

use crate::error::{CoreError, CoreResult};
use crate::path::{SafeRelativePath, PATH_ESCAPE_MSG};
use crate::process::env::{env_key_is_denied, sanitize_env};
use crate::process::kill::{kill_tree_force, kill_with_grace};
use crate::process::output::{truncate_chars, ProcessOutput};
use crate::process::SafeCommand;

pub trait ProcessRunner {
    fn run(&self, cmd: &SafeCommand) -> CoreResult<ProcessOutput>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
    fn run(&self, cmd: &SafeCommand) -> CoreResult<ProcessOutput> {
        if cmd.program.contains('\0') {
            return Err(CoreError::invalid_input("program must not contain NUL"));
        }
        for (k, _) in &cmd.extra_env {
            if env_key_is_denied(k) {
                return Err(CoreError::invalid_input(
                    "environment variable name denied",
                ));
            }
        }

        let program = resolve_program(cmd)?;
        let mut command = Command::new(&program);
        command.args(&cmd.args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.stdin(Stdio::null());

        if let Some(cwd) = &cmd.cwd {
            let abs = cwd.root.resolve(&cwd.rel)?;
            command.current_dir(abs.as_path().as_std_path());
        }

        command.env_clear();
        if cmd.clear_env {
            for (k, v) in &cmd.extra_env {
                command.env(k, v);
            }
        } else {
            for (k, v) in sanitize_env(std::env::vars()) {
                command.env(k, v);
            }
            for (k, v) in &cmd.extra_env {
                command.env(k, v);
            }
        }

        let mut child = match command.spawn() {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(CoreError::not_found("executable not found"));
            }
            Err(e) => return Err(CoreError::io(e.to_string())),
        };

        let pid = child.id();
        let mut stdout_pipe = child.stdout.take();
        let mut stderr_pipe = child.stderr.take();

        let stdout_handle = thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(ref mut r) = stdout_pipe {
                let _ = r.read_to_end(&mut buf);
            }
            buf
        });
        let stderr_handle = thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(ref mut r) = stderr_pipe {
                let _ = r.read_to_end(&mut buf);
            }
            buf
        });

        let deadline = cmd.timeout.map(|d| Instant::now() + d);
        let poll = Duration::from_millis(20);
        let mut timed_out = false;
        let mut cancelled = false;

        loop {
            if let Some(flag) = &cmd.cancel {
                if flag.load(Ordering::SeqCst) {
                    cancelled = true;
                    let _ = kill_with_grace(pid);
                    let _ = wait_with_grace(&mut child, pid);
                    break;
                }
            }
            if let Some(dl) = deadline {
                if Instant::now() >= dl {
                    timed_out = true;
                    let _ = kill_with_grace(pid);
                    let _ = wait_with_grace(&mut child, pid);
                    break;
                }
            }
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => thread::sleep(poll),
                Err(e) => return Err(CoreError::io(e.to_string())),
            }
        }

        let status = if timed_out || cancelled {
            // Ensure process is reaped
            let _ = child.try_wait();
            let _ = child.wait();
            None
        } else {
            Some(child.wait().map_err(|e| CoreError::io(e.to_string()))?)
        };

        let stdout_raw = stdout_handle
            .join()
            .unwrap_or_default();
        let stderr_raw = stderr_handle
            .join()
            .unwrap_or_default();

        let stdout = String::from_utf8_lossy(&stdout_raw).into_owned();
        let stderr = String::from_utf8_lossy(&stderr_raw).into_owned();
        let (stdout, stdout_truncated) = truncate_chars(stdout, cmd.stdout_limit);
        let (stderr, stderr_truncated) = truncate_chars(stderr, cmd.stderr_limit);

        if timed_out {
            return Ok(ProcessOutput {
                exit_code: 124,
                stdout,
                stderr,
                stdout_truncated,
                stderr_truncated,
                timed_out: true,
                cancelled: false,
            });
        }
        if cancelled {
            return Ok(ProcessOutput {
                exit_code: -1,
                stdout,
                stderr,
                stdout_truncated,
                stderr_truncated,
                timed_out: false,
                cancelled: true,
            });
        }

        let exit_code = status
            .and_then(|s| s.code())
            .unwrap_or(-1);

        Ok(ProcessOutput {
            exit_code,
            stdout,
            stderr,
            stdout_truncated,
            stderr_truncated,
            timed_out: false,
            cancelled: false,
        })
    }
}

fn wait_with_grace(child: &mut std::process::Child, pid: u32) -> CoreResult<()> {
    let grace = Duration::from_secs(2);
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return Ok(()),
            Ok(None) if start.elapsed() >= grace => {
                let _ = kill_tree_force(pid);
                let _ = child.kill();
                let _ = child.wait();
                return Ok(());
            }
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(CoreError::io(e.to_string())),
        }
    }
}

fn resolve_program(cmd: &SafeCommand) -> CoreResult<PathBuf> {
    let program = &cmd.program;
    let path = Path::new(program);

    if path.is_absolute() {
        let Some(cwd) = &cmd.cwd else {
            return Err(CoreError::invalid_input(
                "absolute program path must stay within the project",
            ));
        };
        let utf = Utf8PathBuf::from_path_buf(path.to_path_buf()).map_err(|_| {
            CoreError::invalid_input("program path is not valid UTF-8")
        })?;
        if !cwd.root.contains(&utf)? {
            return Err(CoreError::invalid_input(
                "absolute program path must stay within the project",
            ));
        }
        return Ok(path.to_path_buf());
    }

    let has_sep = program.contains('/') || program.contains('\\');
    if has_sep {
        let Some(cwd) = &cmd.cwd else {
            return Err(CoreError::invalid_input(PATH_ESCAPE_MSG));
        };
        let rel = SafeRelativePath::new(program)?;
        let abs = cwd.root.resolve(&rel)?;
        return Ok(abs.as_path().as_std_path().to_path_buf());
    }

    // Bare name — PATH lookup via Command::new
    Ok(PathBuf::from(program))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::{ProjectRoot, SafeRelativePath};
    use crate::process::env::env_key_is_denied;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn echo_hello() -> SafeCommand {
        #[cfg(windows)]
        {
            SafeCommand::new("cmd.exe").args(["/C", "echo", "hello"])
        }
        #[cfg(unix)]
        {
            SafeCommand::new("echo").arg("hello")
        }
    }

    #[test]
    fn extra_env_denied_key_is_invalid_input() {
        assert!(env_key_is_denied("API_TOKEN"));
        let runner = SystemProcessRunner;
        let cmd = SafeCommand::new("echo").env("API_TOKEN", "secret");
        let err = runner.run(&cmd).expect_err("denied");
        assert!(matches!(err, CoreError::InvalidInput(_)));
        assert!(err.to_string().contains("denied"));
    }

    #[test]
    fn system_runner_echo_ok() {
        let out = SystemProcessRunner
            .run(&echo_hello())
            .expect("echo");
        assert_eq!(out.exit_code, 0);
        assert!(out.stdout.to_lowercase().contains("hello"));
        assert!(!out.timed_out);
        assert!(!out.cancelled);
    }

    #[test]
    fn system_runner_truncates_stdout() {
        let cmd = {
            #[cfg(windows)]
            {
                SafeCommand::new("powershell.exe")
                    .args([
                        "-NoProfile",
                        "-Command",
                        "Write-Output (('x' * 5000))",
                    ])
                    .stdout_limit(4000)
            }
            #[cfg(unix)]
            {
                // Prefer python3, fall back to printf via awk-less python
                SafeCommand::new("python3")
                    .args(["-c", "print('x'*5000)"])
                    .stdout_limit(4000)
            }
        };
        let out = SystemProcessRunner.run(&cmd).expect("large out");
        assert!(out.stdout_truncated);
        assert!(out.stdout.chars().count() <= 4000);
    }

    #[test]
    fn system_runner_timeout_returns_124() {
        let cmd = {
            #[cfg(windows)]
            {
                SafeCommand::new("ping")
                    .args(["-n", "20", "127.0.0.1"])
                    .timeout(Duration::from_millis(400))
            }
            #[cfg(unix)]
            {
                SafeCommand::new("sleep")
                    .arg("30")
                    .timeout(Duration::from_millis(400))
            }
        };
        let out = SystemProcessRunner.run(&cmd).expect("timeout");
        assert!(out.timed_out);
        assert_eq!(out.exit_code, 124);
        assert!(!out.cancelled);
    }

    #[test]
    fn system_runner_cancel_sets_cancelled() {
        let flag = Arc::new(AtomicBool::new(false));
        let flag2 = Arc::clone(&flag);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(150));
            flag2.store(true, Ordering::SeqCst);
        });
        let cmd = {
            #[cfg(windows)]
            {
                SafeCommand::new("ping")
                    .args(["-n", "30", "127.0.0.1"])
                    .cancel_flag(Arc::clone(&flag))
            }
            #[cfg(unix)]
            {
                SafeCommand::new("sleep")
                    .arg("30")
                    .cancel_flag(Arc::clone(&flag))
            }
        };
        let out = SystemProcessRunner.run(&cmd).expect("cancel");
        let _ = handle.join();
        assert!(out.cancelled);
        assert_eq!(out.exit_code, -1);
        assert!(!out.timed_out);
    }

    #[test]
    fn system_runner_missing_exe_not_found() {
        let err = SystemProcessRunner
            .run(&SafeCommand::new(
                "__dare_definitely_missing_exe_9f3a2b__",
            ))
            .expect_err("missing");
        assert!(matches!(err, CoreError::NotFound(_)));
        assert!(err.to_string().contains("executable not found"));
    }

    #[test]
    fn relative_program_outside_root_rejected() {
        let dir = tempdir().expect("temp");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let _ = std::fs::create_dir_all(dir.path().join("sub"));
        let rel = SafeRelativePath::new("sub").expect("sub");
        let outside = std::env::temp_dir().join("__dare_outside_prog__");
        let prog = outside.to_string_lossy().into_owned();
        let cmd = SafeCommand::new(prog).cwd(root, rel);
        let err = SystemProcessRunner.run(&cmd).expect_err("outside");
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }

    #[test]
    fn relative_program_with_dotdot_rejected() {
        let dir = tempdir().expect("temp");
        let root = ProjectRoot::new(dir.path()).expect("root");
        let _ = std::fs::create_dir_all(dir.path().join("sub"));
        let rel = SafeRelativePath::new("sub").expect("sub");
        let cmd = SafeCommand::new("../etc/passwd").cwd(root, rel);
        let err = SystemProcessRunner.run(&cmd).expect_err("dotdot");
        assert!(matches!(err, CoreError::InvalidInput(_)));
    }
}
