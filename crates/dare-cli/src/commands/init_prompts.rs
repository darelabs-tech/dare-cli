//! Interactive `dare init` prompts (BLUEPRINT-047 Fase D / mp047-004).

use std::io::IsTerminal;

use dare_core::{CoreError, CoreResult};
use dare_scaffold::{list_stack_ids, validate_project_name};
use dialoguer::{Input, Select};

use crate::commands::init::{
    is_mcp_stack_id, resolve_mcp_language, resolve_stack_alias, InitFlags, MSG_STACK_AND_MCP,
};

pub const MSG_INTERACTIVE_REQUIRES_TTY: &str =
    "interactive mode requires a TTY (use --non-interactive)";
const MSG_PROMPT_CANCELED: &str = "prompt canceled";

/// Resolve missing init flags via dialoguer when not in non-interactive mode.
pub fn prepare_init_flags(flags: InitFlags) -> CoreResult<InitFlags> {
    prepare_init_flags_with_tty(flags, std::io::stdin().is_terminal())
}

/// Same as [`prepare_init_flags`] but accepts an explicit TTY probe (for tests).
pub fn prepare_init_flags_with_tty(mut flags: InitFlags, is_tty: bool) -> CoreResult<InitFlags> {
    if flags.non_interactive {
        return Ok(flags);
    }
    if !is_tty {
        return Err(CoreError::invalid_input(MSG_INTERACTIVE_REQUIRES_TTY));
    }
    prompt_init_interactive(&mut flags)?;
    Ok(flags)
}

fn prompt_init_interactive(flags: &mut InitFlags) -> CoreResult<()> {
    if flags.stack.is_some() && flags.mcp.is_some() {
        return Err(CoreError::usage(MSG_STACK_AND_MCP));
    }

    if flags.name.is_none() {
        flags.name = Some(prompt_project_name()?);
    }

    if flags.stack.is_none() && flags.mcp.is_none() {
        flags.stack = Some(prompt_stack_select()?);
    }

    let stack_id = peek_stack_id(flags)?;

    if flags.fullstack.is_none() && !is_mcp_stack_id(&stack_id) {
        flags.fullstack = prompt_fullstack_opt()?;
    }

    if flags.transport.is_none() && is_mcp_stack_id(&stack_id) {
        flags.transport = Some(prompt_transport()?);
    }

    if flags.toolchain.is_none() {
        flags.toolchain = Some(prompt_toolchain()?);
    }

    Ok(())
}

fn peek_stack_id(flags: &InitFlags) -> CoreResult<String> {
    match (&flags.stack, &flags.mcp) {
        (Some(stack), None) => Ok(resolve_stack_alias(stack)),
        (None, Some(mcp)) => resolve_mcp_language(mcp),
        (None, None) => Err(CoreError::invalid_input("stack required")),
        (Some(_), Some(_)) => Err(CoreError::usage(MSG_STACK_AND_MCP)),
    }
}

fn prompt_project_name() -> CoreResult<String> {
    Input::new()
        .with_prompt("Project name")
        .validate_with(|input: &String| {
            validate_project_name(input).map_err(|e| e.message().to_string())
        })
        .interact_text()
        .map_err(map_dialoguer_err)
}

fn prompt_stack_select() -> CoreResult<String> {
    let stacks: Vec<&str> = list_stack_ids().to_vec();
    let idx = Select::new()
        .with_prompt("Stack")
        .items(&stacks)
        .default(0)
        .interact_opt()
        .map_err(map_dialoguer_err)?
        .ok_or_else(|| CoreError::invalid_input(MSG_PROMPT_CANCELED))?;
    Ok(stacks[idx].to_string())
}

fn prompt_fullstack_opt() -> CoreResult<Option<String>> {
    const OPTIONS: &[&str] = &["none", "react", "vue"];
    let idx = Select::new()
        .with_prompt("Fullstack frontend")
        .items(OPTIONS)
        .default(0)
        .interact_opt()
        .map_err(map_dialoguer_err)?
        .ok_or_else(|| CoreError::invalid_input(MSG_PROMPT_CANCELED))?;
    if OPTIONS[idx] == "none" {
        Ok(None)
    } else {
        Ok(Some(OPTIONS[idx].to_string()))
    }
}

fn prompt_transport() -> CoreResult<String> {
    const OPTIONS: &[&str] = &["stdio", "http", "sse"];
    let idx = Select::new()
        .with_prompt("Transport")
        .items(OPTIONS)
        .default(0)
        .interact_opt()
        .map_err(map_dialoguer_err)?
        .ok_or_else(|| CoreError::invalid_input(MSG_PROMPT_CANCELED))?;
    Ok(OPTIONS[idx].to_string())
}

fn prompt_toolchain() -> CoreResult<String> {
    const OPTIONS: &[&str] = &["none", "docker"];
    let idx = Select::new()
        .with_prompt("Toolchain")
        .items(OPTIONS)
        .default(0)
        .interact_opt()
        .map_err(map_dialoguer_err)?
        .ok_or_else(|| CoreError::invalid_input(MSG_PROMPT_CANCELED))?;
    Ok(OPTIONS[idx].to_string())
}

fn map_dialoguer_err(err: dialoguer::Error) -> CoreError {
    CoreError::invalid_input(format!("prompt failed: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ErrorKind;

    #[test]
    fn reject_interactive_without_tty() {
        let flags = InitFlags {
            non_interactive: false,
            ..InitFlags::default()
        };
        let err = prepare_init_flags_with_tty(flags, false).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert_eq!(err.message(), MSG_INTERACTIVE_REQUIRES_TTY);
    }

    #[test]
    fn non_interactive_skips_tty_check() {
        let flags = InitFlags {
            name: Some("demo".into()),
            non_interactive: true,
            ..InitFlags::default()
        };
        let out = prepare_init_flags_with_tty(flags.clone(), false).expect("non-interactive");
        assert_eq!(out, flags);
    }
}
