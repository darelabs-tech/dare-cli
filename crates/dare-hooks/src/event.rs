//! Closed set of hook events (BLUEPRINT-048 §0.2).

use dare_core::{CoreError, CoreResult};

/// Lifecycle / IDE events that may trigger hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookEvent {
    OnSave,
    OnFileCreate,
    OnTaskComplete,
    PreCommit,
}

impl HookEvent {
    /// Canonical CLI / YAML string for this event.
    pub fn as_str(self) -> &'static str {
        match self {
            HookEvent::OnSave => "on-save",
            HookEvent::OnFileCreate => "on-file-create",
            HookEvent::OnTaskComplete => "on-task-complete",
            HookEvent::PreCommit => "pre-commit",
        }
    }

    /// Parse a case-sensitive event id.
    pub fn parse(event: &str) -> CoreResult<Self> {
        match event {
            "on-save" => Ok(HookEvent::OnSave),
            "on-file-create" => Ok(HookEvent::OnFileCreate),
            "on-task-complete" => Ok(HookEvent::OnTaskComplete),
            "pre-commit" => Ok(HookEvent::PreCommit),
            _ => Err(CoreError::usage(format!("unknown hook event: {event}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ErrorKind;

    #[test]
    fn parse_events() {
        assert_eq!(HookEvent::parse("on-save").unwrap(), HookEvent::OnSave);
        assert_eq!(
            HookEvent::parse("on-file-create").unwrap(),
            HookEvent::OnFileCreate
        );
        assert_eq!(
            HookEvent::parse("on-task-complete").unwrap(),
            HookEvent::OnTaskComplete
        );
        assert_eq!(
            HookEvent::parse("pre-commit").unwrap(),
            HookEvent::PreCommit
        );
        assert_eq!(HookEvent::OnSave.as_str(), "on-save");
        assert_eq!(HookEvent::OnFileCreate.as_str(), "on-file-create");
        assert_eq!(HookEvent::OnTaskComplete.as_str(), "on-task-complete");
        assert_eq!(HookEvent::PreCommit.as_str(), "pre-commit");

        let err = HookEvent::parse("On-Save").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Usage);
        assert!(err.message().contains("unknown hook event: On-Save"));

        let err = HookEvent::parse("on_save").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Usage);
        assert!(err.message().contains("unknown hook event: on_save"));
    }
}
