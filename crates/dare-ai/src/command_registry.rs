//! EnrichCommand registry v1 (BLUEPRINT-050 §0.4).

use dare_core::{CoreError, CoreResult};

use crate::ENRICHABLE;

/// Usage message template for unknown `--command` (BLUEPRINT-050 §0.1).
/// Concrete errors interpolate `{c}` with the unknown command id.
pub const MSG_UNKNOWN_COMMAND: &str = "unknown ai command: {c} (expected design|blueprint)";

/// Blueprint enrichable section ids — literals aligned with CLI `BP_ENRICHABLE`.
pub const BP_ENRICHABLE: &[&str] = &[
    "architecture-overview",
    "execution-phases",
    "api-contracts",
    "data-model",
];

/// Resolve enrichable section ids for a `--command` value.
///
/// Known: `design` → [`ENRICHABLE`], `blueprint` → [`BP_ENRICHABLE`].
/// Unknown → [`CoreError::usage`] with [`MSG_UNKNOWN_COMMAND`].
pub fn sections_for_command(command: &str) -> CoreResult<&'static [&'static str]> {
    match command {
        "design" => Ok(ENRICHABLE),
        "blueprint" => Ok(BP_ENRICHABLE),
        _ => Err(CoreError::usage(format!(
            "unknown ai command: {command} (expected design|blueprint)"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ErrorKind;

    #[test]
    fn sections_design_ok() {
        let sections = sections_for_command("design").expect("design");
        assert_eq!(
            sections,
            &[
                "description",
                "objectives",
                "functional-requirements",
                "stack",
            ]
        );
        assert_eq!(sections, ENRICHABLE);
    }

    #[test]
    fn sections_blueprint_ok() {
        let sections = sections_for_command("blueprint").expect("blueprint");
        assert_eq!(
            sections,
            &[
                "architecture-overview",
                "execution-phases",
                "api-contracts",
                "data-model",
            ]
        );
        assert_eq!(sections, BP_ENRICHABLE);
    }

    #[test]
    fn unknown_command_usage() {
        let err = sections_for_command("tasks").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Usage);
        assert_eq!(
            err.message(),
            "unknown ai command: tasks (expected design|blueprint)"
        );
        assert!(err.message().starts_with("unknown ai command:"));
        assert!(MSG_UNKNOWN_COMMAND.contains("expected design|blueprint"));
    }
}
