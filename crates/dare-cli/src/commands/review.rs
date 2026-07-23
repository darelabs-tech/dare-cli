//! `dare review` — static anti-stub review (microplano 032).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use dare_core::{CoreError, CoreResult, ProjectRoot};
use dare_project::find_project_root;
use dare_review::{
    format_github, format_human, report_to_json, run_review, should_fail_exit, FailOn,
    OutputFormat, ReviewOptions,
};
use serde_json::Value;

use crate::output::OutputRenderer;

pub struct ReviewCliArgs {
    pub task_id: String,
    pub strict: bool,
    pub errors_only: bool,
    pub files: Vec<PathBuf>,
    pub from_agent: Option<PathBuf>,
    pub format: String,
    pub comment: bool,
    pub fail_on: String,
    pub ai: bool,
    pub provider: Option<String>,
}

pub fn run_review_cmd(args: ReviewCliArgs, renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_review_inner(args) {
        Ok((human, data, report_ok, fail_exit)) => {
            // When format=json without global --json, emit raw report as human body already JSON.
            if let Err(e) = renderer.write_report(&human, data, report_ok && !fail_exit) {
                return exit_err(renderer, &e);
            }
            if fail_exit {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => exit_err(renderer, &e),
    }
}

fn exit_err(renderer: &OutputRenderer<'_>, e: &CoreError) -> ExitCode {
    let code = renderer.write_error(e);
    ExitCode::from(code as u8)
}

fn run_review_inner(args: ReviewCliArgs) -> CoreResult<(String, Value, bool, bool)> {
    let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
    let Some(root_path) = find_project_root(&cwd) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&root_path)?;

    let format = OutputFormat::parse(&args.format)
        .ok_or_else(|| CoreError::invalid_input("invalid --format (expected human|json|github)"))?;
    let fail_on = FailOn::parse(&args.fail_on).ok_or_else(|| {
        CoreError::invalid_input("invalid --fail-on (expected error|warning|never)")
    })?;

    if args.provider.is_some() && !args.ai {
        return Err(CoreError::invalid_input("--provider requires --ai"));
    }

    let files_override = if args.files.is_empty() {
        None
    } else {
        let mut list = Vec::new();
        for p in &args.files {
            let s = path_under_root(&root, p)?;
            list.push(s);
        }
        Some(list)
    };

    let from_agent = match args.from_agent {
        Some(p) => Some(path_under_root(&root, &p)?),
        None => None,
    };

    let opts = ReviewOptions {
        task_id: args.task_id,
        files_override,
        strict: args.strict,
        errors_only: args.errors_only,
        from_agent,
        format,
        comment: args.comment,
        fail_on,
        ai: args.ai,
    };

    let report = run_review(&root, &opts)?;
    let fail_exit = should_fail_exit(&report, fail_on);
    let data = report_to_json(&report)?;

    let human = match format {
        OutputFormat::Human => format_human(&report, args.errors_only),
        OutputFormat::Json => dare_core::to_canonical_json_string(&data)? + "\n",
        OutputFormat::Github => format_github(&report, args.errors_only),
    };

    Ok((human, data, report.ok, fail_exit))
}

fn path_under_root(root: &ProjectRoot, p: &Path) -> CoreResult<String> {
    if p.is_absolute() {
        let root_abs = root.as_path();
        let rel = p
            .strip_prefix(root_abs.as_std_path())
            .map_err(|_| CoreError::invalid_input("file path escapes project root"))?;
        Ok(rel.to_string_lossy().replace('\\', "/"))
    } else {
        Ok(p.to_string_lossy().replace('\\', "/"))
    }
}
