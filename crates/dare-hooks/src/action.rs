//! Closed allowlist of hook actions (BLUEPRINT-048 §0.3).

use dare_core::{CoreError, CoreResult};

/// Allowlisted actions that hooks may spawn via `current_exe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookAction {
    DareValidate,
    DareReview,
    GraphRegister,
    Lint,
    Test,
}

impl HookAction {
    /// Canonical YAML / report id for this action.
    pub fn as_str(self) -> &'static str {
        match self {
            HookAction::DareValidate => "dare-validate",
            HookAction::DareReview => "dare-review",
            HookAction::GraphRegister => "graph-register",
            HookAction::Lint => "lint",
            HookAction::Test => "test",
        }
    }

    /// Parse a case-sensitive action id.
    pub fn parse(action: &str) -> CoreResult<Self> {
        match action {
            "dare-validate" => Ok(HookAction::DareValidate),
            "dare-review" => Ok(HookAction::DareReview),
            "graph-register" => Ok(HookAction::GraphRegister),
            "lint" => Ok(HookAction::Lint),
            "test" => Ok(HookAction::Test),
            _ => Err(CoreError::invalid_input(format!(
                "unknown hook action: {action}"
            ))),
        }
    }
}

/// Argv fragments appended to `current_exe` for the given action.
pub fn action_argv(action: HookAction) -> &'static [&'static str] {
    match action {
        HookAction::DareValidate => &["validate"],
        HookAction::DareReview => &["review"],
        HookAction::GraphRegister => &["graph", "ingest"],
        HookAction::Lint => &["guard"],
        HookAction::Test => &["info"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ErrorKind;

    #[test]
    fn parse_actions() {
        assert_eq!(
            HookAction::parse("dare-validate").unwrap(),
            HookAction::DareValidate
        );
        assert_eq!(
            HookAction::parse("dare-review").unwrap(),
            HookAction::DareReview
        );
        assert_eq!(
            HookAction::parse("graph-register").unwrap(),
            HookAction::GraphRegister
        );
        assert_eq!(HookAction::parse("lint").unwrap(), HookAction::Lint);
        assert_eq!(HookAction::parse("test").unwrap(), HookAction::Test);

        assert_eq!(HookAction::DareValidate.as_str(), "dare-validate");
        assert_eq!(HookAction::DareReview.as_str(), "dare-review");
        assert_eq!(HookAction::GraphRegister.as_str(), "graph-register");
        assert_eq!(HookAction::Lint.as_str(), "lint");
        assert_eq!(HookAction::Test.as_str(), "test");

        assert_eq!(action_argv(HookAction::DareValidate), &["validate"]);
        assert_eq!(action_argv(HookAction::DareReview), &["review"]);
        assert_eq!(
            action_argv(HookAction::GraphRegister),
            &["graph", "ingest"]
        );
        assert_eq!(action_argv(HookAction::Lint), &["guard"]);
        assert_eq!(action_argv(HookAction::Test), &["info"]);

        let err = HookAction::parse("shell").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert!(err.message().contains("unknown hook action: shell"));

        let err = HookAction::parse("Dare-Validate").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }
}
