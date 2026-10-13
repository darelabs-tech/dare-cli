//! Toolchain resolution and external scaffolders — port of npm `StackTool`.
//!
//! Commands are spawned argv-only (never through a shell) with inherited stdio,
//! exactly like npm's `runCmd`. Program lookup walks PATH honoring PATHEXT on
//! Windows, so `npm.cmd` / `npx.cmd` / `rails.bat` are found (fix B-04: npm's
//! `hasCommand` spawns `<cmd> --version` without a shell and never finds
//! `.cmd` shims, so `auto` always fell back to Docker on Windows).

use std::path::{Path, PathBuf};
use std::process::Command;

use dare_core::{CoreError, CoreResult};

use super::config::ToolchainMode;
use super::console::{Console, Tone};

/// Spawns external tools; swapped for a recorder in tests.
pub trait ToolRunner {
    /// npm `hasCommand`.
    fn has_command(&self, cmd: &str) -> bool;
    /// npm `runCmd` — fails on non-zero exit.
    fn run(&mut self, program: &str, args: &[String], cwd: &Path) -> CoreResult<()>;
    /// `--user uid:gid` for Docker on Unix (None on Windows).
    fn docker_user(&self, dir: &Path) -> Option<String>;
}

/// Resolve `program` on PATH (with PATHEXT on Windows). No process spawn.
pub fn resolve_program(program: &str) -> Option<PathBuf> {
    let path = Path::new(program);
    if path.is_absolute() || program.contains('/') || program.contains('\\') {
        return path.is_file().then(|| path.to_path_buf());
    }
    let path_env = std::env::var_os("PATH")?;
    #[cfg(windows)]
    let exts: Vec<String> = std::env::var("PATHEXT")
        .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into())
        .split(';')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    for dir in std::env::split_paths(&path_env) {
        #[cfg(windows)]
        {
            for ext in &exts {
                let cand = dir.join(format!("{program}{}", ext.to_ascii_lowercase()));
                if cand.is_file() {
                    return Some(cand);
                }
            }
        }
        let cand = dir.join(program);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// Real spawner (inherited stdio, argv-only).
#[derive(Debug, Default)]
pub struct SystemToolRunner;

impl ToolRunner for SystemToolRunner {
    fn has_command(&self, cmd: &str) -> bool {
        resolve_program(cmd).is_some()
    }

    fn run(&mut self, program: &str, args: &[String], cwd: &Path) -> CoreResult<()> {
        let resolved = resolve_program(program)
            .ok_or_else(|| CoreError::not_found(format!("spawn {program} ENOENT")))?;
        let status = Command::new(&resolved)
            .args(args)
            .current_dir(cwd)
            .status()
            .map_err(|e| CoreError::io(format!("spawn {program}: {e}")))?;
        if status.success() {
            Ok(())
        } else {
            let code = status
                .code()
                .map_or_else(|| "null".to_string(), |c| c.to_string());
            Err(CoreError::io(format!(
                "{program} {} exited with code {code}",
                args.join(" ")
            )))
        }
    }

    fn docker_user(&self, dir: &Path) -> Option<String> {
        docker_user_for(dir)
    }
}

#[cfg(unix)]
fn docker_user_for(dir: &Path) -> Option<String> {
    // The output dir was just created by this process, so its owner is the
    // effective uid/gid (avoids a libc dependency).
    use std::os::unix::fs::MetadataExt;
    let meta = std::fs::metadata(dir).ok()?;
    Some(format!("{}:{}", meta.uid(), meta.gid()))
}

#[cfg(not(unix))]
fn docker_user_for(_dir: &Path) -> Option<String> {
    None
}

/// One recorded invocation (tests / `--check`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCommand {
    pub cwd: PathBuf,
    pub argv: Vec<String>,
}

/// Records invocations instead of running them.
#[derive(Debug, Default)]
pub struct RecordingToolRunner {
    pub available: Vec<String>,
    pub commands: Vec<RecordedCommand>,
}

impl RecordingToolRunner {
    pub fn with_tools(tools: &[&str]) -> Self {
        Self {
            available: tools.iter().map(|s| (*s).to_string()).collect(),
            commands: Vec::new(),
        }
    }
}

impl ToolRunner for RecordingToolRunner {
    fn has_command(&self, cmd: &str) -> bool {
        self.available.iter().any(|t| t == cmd)
    }

    fn run(&mut self, program: &str, args: &[String], cwd: &Path) -> CoreResult<()> {
        let mut argv = vec![program.to_string()];
        argv.extend(args.iter().cloned());
        self.commands.push(RecordedCommand {
            cwd: cwd.to_path_buf(),
            argv,
        });
        Ok(())
    }

    fn docker_user(&self, _dir: &Path) -> Option<String> {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolMode {
    Native,
    Docker,
}

/// npm `StackTool` — a resolved native/Docker toolchain bound to a directory.
pub struct StackTool {
    native_cmd: &'static str,
    mode: ToolMode,
    dir: PathBuf,
    image: &'static str,
    image_has_entrypoint: bool,
}

/// npm `dockerizePath` (Windows → forward slashes).
fn dockerize_path(p: &Path) -> String {
    let s = p.to_string_lossy().into_owned();
    if cfg!(windows) {
        s.replace('\\', "/")
    } else {
        s
    }
}

pub struct ToolSpec {
    pub native_cmd: &'static str,
    pub native_hint: String,
    pub docker_image: &'static str,
    pub image_has_entrypoint: bool,
}

impl StackTool {
    /// npm `StackTool.resolve` — same decision table and messages.
    pub fn resolve(
        spec: ToolSpec,
        dir: &Path,
        mode: ToolchainMode,
        runner: &dyn ToolRunner,
        console: &mut dyn Console,
    ) -> CoreResult<StackTool> {
        let docker = |dir: &Path| StackTool {
            native_cmd: spec.native_cmd,
            mode: ToolMode::Docker,
            dir: dir.to_path_buf(),
            image: spec.docker_image,
            image_has_entrypoint: spec.image_has_entrypoint,
        };
        let native = |dir: &Path| StackTool {
            native_cmd: spec.native_cmd,
            mode: ToolMode::Native,
            dir: dir.to_path_buf(),
            image: spec.docker_image,
            image_has_entrypoint: false,
        };
        match mode {
            ToolchainMode::Docker => {
                if !runner.has_command("docker") {
                    return Err(CoreError::not_found(
                        "--toolchain=docker selected, but `docker` is not on PATH.\n  Install Docker Desktop: https://www.docker.com/products/docker-desktop/",
                    ));
                }
                console.log(
                    Tone::Cyan,
                    &format!("🐳 Using Docker ({}) — toolchain=docker", spec.docker_image),
                );
                Ok(docker(dir))
            }
            ToolchainMode::Native => {
                if !runner.has_command(spec.native_cmd) {
                    return Err(CoreError::not_found(format!(
                        "--toolchain=native selected, but `{}` is not on PATH.\n  {}",
                        spec.native_cmd, spec.native_hint
                    )));
                }
                console.log(
                    Tone::Green,
                    &format!("🔧 Using native {} — toolchain=native", spec.native_cmd),
                );
                Ok(native(dir))
            }
            ToolchainMode::Auto => {
                if runner.has_command(spec.native_cmd) {
                    console.log(
                        Tone::Green,
                        &format!("🔧 Using native {} — toolchain=auto", spec.native_cmd),
                    );
                    return Ok(native(dir));
                }
                if runner.has_command("docker") {
                    console.log(
                        Tone::Yellow,
                        &format!(
                            "⚠  {} not found on PATH — falling back to Docker ({}).",
                            spec.native_cmd, spec.docker_image
                        ),
                    );
                    return Ok(docker(dir));
                }
                Err(CoreError::not_found(format!(
                    "Required tool not found: {}\n  {}\n  …or install Docker (https://www.docker.com/products/docker-desktop/) — DARE will use the\n  {} image automatically.",
                    spec.native_cmd, spec.native_hint, spec.docker_image
                )))
            }
        }
    }

    pub fn using_docker(&self) -> bool {
        self.mode == ToolMode::Docker
    }

    fn docker_base(&self, runner: &dyn ToolRunner) -> Vec<String> {
        let mut base = vec![
            "run".to_string(),
            "--rm".to_string(),
            "-v".to_string(),
            format!("{}:/app", dockerize_path(&self.dir)),
            "-w".to_string(),
            "/app".to_string(),
        ];
        if let Some(user) = runner.docker_user(&self.dir) {
            base.push("--user".to_string());
            base.push(user);
        }
        base.push(self.image.to_string());
        base
    }

    fn log_and_run(
        &self,
        runner: &mut dyn ToolRunner,
        console: &mut dyn Console,
        program: &str,
        args: Vec<String>,
    ) -> CoreResult<()> {
        console.log(Tone::Gray, &format!("  $ {program} {}", args.join(" ")));
        runner.run(program, &args, &self.dir)
    }

    /// npm `StackTool.run`.
    pub fn run(
        &self,
        runner: &mut dyn ToolRunner,
        console: &mut dyn Console,
        args: &[&str],
    ) -> CoreResult<()> {
        let args: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
        if self.mode == ToolMode::Native {
            return self.log_and_run(runner, console, self.native_cmd, args);
        }
        let mut docker_args = self.docker_base(runner);
        if !self.image_has_entrypoint {
            docker_args.push(self.native_cmd.to_string());
        }
        docker_args.extend(args);
        self.log_and_run(runner, console, "docker", docker_args)
    }

    /// npm `StackTool.runOther`.
    pub fn run_other(
        &self,
        runner: &mut dyn ToolRunner,
        console: &mut dyn Console,
        other: &str,
        args: &[&str],
    ) -> CoreResult<()> {
        let args: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
        if self.mode == ToolMode::Native {
            return self.log_and_run(runner, console, other, args);
        }
        let mut docker_args = self.docker_base(runner);
        docker_args.push(other.to_string());
        docker_args.extend(args);
        self.log_and_run(runner, console, "docker", docker_args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::console::MemoryConsole;

    fn npm_spec() -> ToolSpec {
        ToolSpec {
            native_cmd: "npm",
            native_hint: "Install Node.js: https://nodejs.org/".into(),
            docker_image: "node:20-alpine",
            image_has_entrypoint: false,
        }
    }

    #[test]
    fn auto_prefers_native_then_docker_then_errors() {
        let mut c = MemoryConsole::default();
        let dir = Path::new("/w/app");
        let r = RecordingToolRunner::with_tools(&["npm", "docker"]);
        let t = StackTool::resolve(npm_spec(), dir, ToolchainMode::Auto, &r, &mut c).unwrap();
        assert!(!t.using_docker());
        let r = RecordingToolRunner::with_tools(&["docker"]);
        let t = StackTool::resolve(npm_spec(), dir, ToolchainMode::Auto, &r, &mut c).unwrap();
        assert!(t.using_docker());
        assert!(c
            .lines
            .last()
            .unwrap()
            .contains("falling back to Docker (node:20-alpine)"));
        let r = RecordingToolRunner::with_tools(&[]);
        let err = StackTool::resolve(npm_spec(), dir, ToolchainMode::Auto, &r, &mut c)
            .err()
            .expect("error");
        assert!(err.message().starts_with("Required tool not found: npm"));
    }

    #[test]
    fn docker_argv_matches_npm() {
        let mut c = MemoryConsole::default();
        let dir = Path::new("/w/app");
        let mut r = RecordingToolRunner::with_tools(&["docker"]);
        let t = StackTool::resolve(npm_spec(), dir, ToolchainMode::Docker, &r, &mut c).unwrap();
        t.run(&mut r, &mut c, &["install"]).unwrap();
        t.run_other(&mut r, &mut c, "npx", &["-y", "degit"])
            .unwrap();
        let mount = format!("{}:/app", dockerize_path(dir));
        assert_eq!(
            r.commands[0].argv,
            [
                "docker",
                "run",
                "--rm",
                "-v",
                &mount,
                "-w",
                "/app",
                "node:20-alpine",
                "npm",
                "install"
            ]
        );
        assert_eq!(
            r.commands[1].argv,
            [
                "docker",
                "run",
                "--rm",
                "-v",
                &mount,
                "-w",
                "/app",
                "node:20-alpine",
                "npx",
                "-y",
                "degit"
            ]
        );
    }

    #[test]
    fn native_mode_requires_tool() {
        let mut c = MemoryConsole::default();
        let r = RecordingToolRunner::with_tools(&["docker"]);
        let err = StackTool::resolve(
            npm_spec(),
            Path::new("/w"),
            ToolchainMode::Native,
            &r,
            &mut c,
        )
        .err()
        .expect("error");
        assert_eq!(
            err.message(),
            "--toolchain=native selected, but `npm` is not on PATH.\n  Install Node.js: https://nodejs.org/"
        );
    }
}
