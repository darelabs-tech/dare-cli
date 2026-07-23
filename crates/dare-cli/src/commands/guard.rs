//! `dare guard` — unicode / injection / provenance security gate (microplano 034).

use std::path::PathBuf;
use std::process::ExitCode;

use dare_core::{CoreError, CoreResult, ProjectRoot};
use dare_guard::{
    format_human, process_exit_for_report, report_to_json, run_guard, sign_file, GuardOptions,
    UnicodeMode,
};
use dare_project::find_project_root;
use serde_json::Value;

use crate::output::OutputRenderer;

pub struct GuardCliOpts {
    pub target: Option<PathBuf>,
    pub staged: bool,
    pub all: bool,
    pub sign: bool,
    pub unicode: String,
    pub strict: bool,
    pub fail_on: String,
    pub format: String,
    pub comment: Option<String>,
}

pub fn run_guard_cmd(opts: GuardCliOpts, renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_guard_inner(opts) {
        Ok((human, data, exit)) => {
            let ok = exit == 0;
            if let Err(e) = renderer.write_report(&human, data, ok) {
                return exit_err(renderer, &e);
            }
            ExitCode::from(exit as u8)
        }
        Err(e) => exit_err(renderer, &e),
    }
}

fn exit_err(renderer: &OutputRenderer<'_>, e: &CoreError) -> ExitCode {
    let code = renderer.write_error(e);
    ExitCode::from(code as u8)
}

fn run_guard_inner(opts: GuardCliOpts) -> CoreResult<(String, Value, i32)> {
    let modes = [opts.staged, opts.all, opts.sign];
    if modes.iter().filter(|x| **x).count() > 1 {
        return Err(CoreError::usage(
            "flags --staged, --all, and --sign are mutually exclusive",
        ));
    }
    if opts.sign && opts.target.is_none() {
        return Err(CoreError::usage("--sign requires a target path"));
    }

    let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
    let Some(root_path) = find_project_root(&cwd) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&root_path)?;

    if opts.sign {
        let target = opts.target.as_ref().expect("checked");
        let abs = if target.is_absolute() {
            target.clone()
        } else {
            root.as_path().as_std_path().join(target)
        };
        if !abs.is_file() {
            return Err(CoreError::not_found(format!(
                "target not found: {}",
                target.display()
            )));
        }
        let key = std::env::var("DARE_GUARD_PRIVATE_KEY")
            .map_err(|_| CoreError::invalid_input("DARE_GUARD_PRIVATE_KEY required for --sign"))?;
        sign_file(&abs, &key)?;
        if let Some(c) = &opts.comment {
            let _ = c; // reserved; signature header is fixed in alpha
        }
        let human = format!("Signed {}", abs.display());
        let data = serde_json::json!({
            "action": "sign",
            "path": abs.to_string_lossy(),
        });
        return Ok((human, data, 0));
    }

    let unicode = UnicodeMode::parse(&opts.unicode)?;
    let fail_on_warn = opts.strict || opts.fail_on.eq_ignore_ascii_case("warn");
    if !matches!(opts.fail_on.as_str(), "fail" | "warn") {
        return Err(CoreError::invalid_input("--fail-on must be fail|warn"));
    }

    let mut guard_opts = GuardOptions {
        unicode_mode: unicode,
        fail_on_warn,
        ..GuardOptions::default()
    };

    // Optional signing verification from env/config-like env
    if let Ok(pk) = std::env::var("DARE_GUARD_PUBLIC_KEY") {
        if !pk.is_empty() {
            guard_opts.signing_enabled = std::env::var("DARE_GUARD_SIGNING_ENABLED")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            guard_opts.public_key_hex = Some(pk);
        }
    }

    let report = run_guard(
        &root,
        opts.target.as_deref(),
        opts.staged,
        opts.all,
        &guard_opts,
    )?;

    let human = format_human(&report);
    let mut data = report_to_json(&report)?;
    if let Some(obj) = data.as_object_mut() {
        obj.insert("action".into(), Value::String("guard".into()));
    }

    if opts.format != "text" && opts.format != "json" {
        return Err(CoreError::invalid_input("--format must be text|json"));
    }
    let exit = process_exit_for_report(&report, opts.strict);
    Ok((human, data, exit))
}
