//! `dare init` — greenfield project bootstrap (microplano 047).

use std::path::{Path, PathBuf};

use dare_core::{CoreError, CoreResult};
use dare_scaffold::{FrontendKind, Toolchain, Transport};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const INIT_REPORT_SCHEMA: u32 = 1;

pub const MSG_STACK_AND_MCP: &str = "--stack and --mcp are mutually exclusive";
pub const MSG_NEED_STACK_OR_MCP: &str = "--non-interactive requires --stack or --mcp";
pub const MSG_FULLSTACK_NEEDS_STACK: &str = "--fullstack requires --stack";
pub const MSG_TRANSPORT_BACKEND: &str = "transport is only valid for mcp stacks";
pub const MSG_FULLSTACK_BACKEND_ONLY: &str = "fullstack is only valid with backend stacks";

pub const HARNESS_IDS: &[&str] = &["antigravity", "claude", "codex", "cursor"];

/// Raw CLI flags before resolution (testable without clap).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InitFlags {
    pub name: Option<String>,
    pub stack: Option<String>,
    pub mcp: Option<String>,
    pub fullstack: Option<String>,
    pub transport: Option<String>,
    pub toolchain: Option<String>,
    pub non_interactive: bool,
    pub force: bool,
    pub check: bool,
}

/// Resolved init domain request (BLUEPRINT-047 §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitRequest {
    pub project_name: String,
    pub stack_id: String,
    pub toolchain: Toolchain,
    pub transport: Option<Transport>,
    pub frontend: Option<FrontendKind>,
    pub force: bool,
    pub check: bool,
    pub non_interactive: bool,
}

/// Init execution report (schemaVersion 1, camelCase JSON).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitReport {
    pub schema_version: u32,
    pub mode: String,
    pub project_root: String,
    pub project_name: String,
    pub stack_id: String,
    pub frontend: Option<FrontendKind>,
    pub toolchain: Toolchain,
    pub transport: Option<Transport>,
    pub created: Vec<String>,
    pub replaced: Vec<String>,
    pub skipped: Vec<String>,
    pub harnesses_installed: Vec<String>,
    pub rolled_back: bool,
    pub check: bool,
}

pub struct InitCliOpts {
    pub name: Option<String>,
    pub dir: Option<PathBuf>,
    pub stack: Option<String>,
    pub mcp: Option<String>,
    pub fullstack: Option<String>,
    pub transport: Option<String>,
    pub toolchain: Option<String>,
    pub non_interactive: bool,
    pub force: bool,
    pub check: bool,
}

/// Resolve CLI flags into an [`InitRequest`] (pure — no filesystem).
pub fn resolve_init_flags(flags: &InitFlags) -> CoreResult<InitRequest> {
    if flags.stack.is_some() && flags.mcp.is_some() {
        return Err(CoreError::usage(MSG_STACK_AND_MCP));
    }

    if flags.non_interactive {
        if flags.name.is_none() {
            return Err(CoreError::usage("--non-interactive requires project name"));
        }
        if flags.stack.is_none() && flags.mcp.is_none() {
            return Err(CoreError::usage(MSG_NEED_STACK_OR_MCP));
        }
    }

    if let Some(ref fullstack) = flags.fullstack {
        match (&flags.stack, &flags.mcp) {
            (None, Some(_)) => {
                return Err(CoreError::invalid_input(MSG_FULLSTACK_BACKEND_ONLY));
            }
            (None, None) => {
                return Err(CoreError::invalid_input(MSG_FULLSTACK_NEEDS_STACK));
            }
            (Some(stack), _) => {
                let stack_id = resolve_stack_alias(stack);
                if is_mcp_stack_id(&stack_id) {
                    return Err(CoreError::invalid_input(MSG_FULLSTACK_BACKEND_ONLY));
                }
                let _ = fullstack;
            }
        }
    }

    let stack_id = match (&flags.stack, &flags.mcp) {
        (Some(stack), None) => resolve_stack_alias(stack),
        (None, Some(mcp)) => resolve_mcp_language(mcp)?,
        (None, None) if flags.non_interactive => {
            return Err(CoreError::usage(MSG_NEED_STACK_OR_MCP));
        }
        (None, None) => String::new(),
        (Some(_), Some(_)) => unreachable!("stack/mcp mutual exclusion checked above"),
    };

    if flags.transport.is_some() && !is_mcp_stack_id(&stack_id) {
        return Err(CoreError::invalid_input(MSG_TRANSPORT_BACKEND));
    }

    let frontend = flags
        .fullstack
        .as_deref()
        .map(resolve_fullstack)
        .transpose()?;

    let transport = flags
        .transport
        .as_deref()
        .map(parse_transport)
        .transpose()?;

    let toolchain = flags
        .toolchain
        .as_deref()
        .map(parse_toolchain)
        .transpose()?
        .unwrap_or(Toolchain::None);

    Ok(InitRequest {
        project_name: flags.name.clone().unwrap_or_default(),
        stack_id,
        toolchain,
        transport,
        frontend,
        force: flags.force,
        check: flags.check,
        non_interactive: flags.non_interactive,
    })
}

/// Map `--stack` value; `rails` → `ruby-rails-8`.
pub fn resolve_stack_alias(input: &str) -> String {
    if input == "rails" {
        "ruby-rails-8".to_string()
    } else {
        input.to_string()
    }
}

/// Map `--mcp` language alias (case-insensitive) → canonical stack id.
pub fn resolve_mcp_language(input: &str) -> CoreResult<String> {
    match input.to_ascii_lowercase().as_str() {
        "ts" | "node" | "typescript" | "mcp-node-ts" => Ok("mcp-node-ts".to_string()),
        "python" | "py" | "mcp-python" => Ok("mcp-python".to_string()),
        "rust" | "mcp-rust" => Ok("mcp-rust".to_string()),
        "go" | "mcp-go" => Ok("mcp-go".to_string()),
        other => Err(CoreError::invalid_input(format!(
            "unknown mcp language: {other}"
        ))),
    }
}

fn resolve_fullstack(input: &str) -> CoreResult<FrontendKind> {
    match input.to_ascii_lowercase().as_str() {
        "react" => Ok(FrontendKind::React),
        "vue" => Ok(FrontendKind::Vue),
        other => Err(CoreError::invalid_input(format!("unknown frontend: {other}"))),
    }
}

fn parse_toolchain(input: &str) -> CoreResult<Toolchain> {
    match input.to_ascii_lowercase().as_str() {
        "none" => Ok(Toolchain::None),
        "docker" => Ok(Toolchain::Docker),
        other => Err(CoreError::invalid_input(format!("unknown toolchain: {other}"))),
    }
}

fn parse_transport(input: &str) -> CoreResult<Transport> {
    match input.to_ascii_lowercase().as_str() {
        "stdio" => Ok(Transport::Stdio),
        "http" => Ok(Transport::Http),
        "sse" => Ok(Transport::Sse),
        other => Err(CoreError::invalid_input(format!("unknown transport: {other}"))),
    }
}

fn is_mcp_stack_id(stack_id: &str) -> bool {
    matches!(
        stack_id,
        "mcp-go" | "mcp-node-ts" | "mcp-python" | "mcp-rust"
    )
}

/// Stub init runner — full FS pipeline deferred to mp047-003/005.
pub fn run_init(parent: &Path, req: &InitRequest) -> CoreResult<InitReport> {
    let target = if req.project_name.is_empty() {
        parent.to_path_buf()
    } else {
        parent.join(&req.project_name)
    };
    Ok(InitReport {
        schema_version: INIT_REPORT_SCHEMA,
        mode: "init".to_string(),
        project_root: target.to_string_lossy().into_owned(),
        project_name: req.project_name.clone(),
        stack_id: req.stack_id.clone(),
        frontend: req.frontend,
        toolchain: req.toolchain,
        transport: req.transport,
        created: Vec::new(),
        replaced: Vec::new(),
        skipped: Vec::new(),
        harnesses_installed: if req.check {
            Vec::new()
        } else {
            HARNESS_IDS.iter().map(|s| (*s).to_string()).collect()
        },
        rolled_back: false,
        check: req.check,
    })
}

pub fn init_report_to_json(report: &InitReport) -> CoreResult<String> {
    serde_json::to_string_pretty(report)
        .map_err(|e| CoreError::io(format!("serialize init report: {e}")))
}

pub fn run_init_cmd(opts: InitCliOpts) -> CoreResult<(String, Value)> {
    let parent = opts
        .dir
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let flags = InitFlags {
        name: opts.name,
        stack: opts.stack,
        mcp: opts.mcp,
        fullstack: opts.fullstack,
        transport: opts.transport,
        toolchain: opts.toolchain,
        non_interactive: opts.non_interactive,
        force: opts.force,
        check: opts.check,
    };

    let req = resolve_init_flags(&flags)?;
    let report = run_init(&parent, &req)?;
    let json_str = init_report_to_json(&report)?;
    let data: Value = serde_json::from_str(&json_str)
        .map_err(|e| CoreError::io(format!("parse init report json: {e}")))?;
    let human = format!(
        "init: {} stack={} check={}",
        report.project_name, report.stack_id, report.check
    );
    Ok((human, data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ErrorKind;

    fn base_flags() -> InitFlags {
        InitFlags {
            name: Some("demo-app".into()),
            stack: Some("rust-axum".into()),
            non_interactive: true,
            ..InitFlags::default()
        }
    }

    #[test]
    fn resolve_rails_alias() {
        let mut flags = base_flags();
        flags.stack = Some("rails".into());
        let req = resolve_init_flags(&flags).expect("resolve");
        assert_eq!(req.stack_id, "ruby-rails-8");
    }

    #[test]
    fn resolve_mcp_map() {
        let cases = [
            ("ts", "mcp-node-ts"),
            ("TS", "mcp-node-ts"),
            ("node", "mcp-node-ts"),
            ("typescript", "mcp-node-ts"),
            ("python", "mcp-python"),
            ("py", "mcp-python"),
            ("rust", "mcp-rust"),
            ("go", "mcp-go"),
        ];
        for (input, want) in cases {
            let mut flags = base_flags();
            flags.stack = None;
            flags.mcp = Some(input.into());
            let req = resolve_init_flags(&flags).expect("resolve");
            assert_eq!(req.stack_id, want, "input={input}");
        }
    }

    #[test]
    fn reject_stack_and_mcp() {
        let mut flags = base_flags();
        flags.mcp = Some("ts".into());
        let err = resolve_init_flags(&flags).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Usage);
        assert_eq!(err.message(), MSG_STACK_AND_MCP);
    }

    #[test]
    fn reject_noninteractive_incomplete() {
        let flags = InitFlags {
            name: Some("demo".into()),
            non_interactive: true,
            ..InitFlags::default()
        };
        let err = resolve_init_flags(&flags).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Usage);
        assert_eq!(err.message(), MSG_NEED_STACK_OR_MCP);
    }

    #[test]
    fn reject_transport_on_backend() {
        let mut flags = base_flags();
        flags.transport = Some("stdio".into());
        let err = resolve_init_flags(&flags).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert_eq!(err.message(), MSG_TRANSPORT_BACKEND);
    }
}
