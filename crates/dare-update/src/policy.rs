//! Apply-action policy for `dare update` (BLUEPRINT-022 §5.1).

use crate::classify::AssetUpdateStatus;

/// Outcome of [`resolve_action`] for one plan item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyAction {
    Keep,
    Replace,
}

/// Context passed to an interactive ask callback (customized paths, sorted POSIX).
pub struct AskContext {
    pub customized_paths: Vec<String>,
}

/// Interactive callback: `true` = replace all customized in this batch.
pub type AskFn = Box<dyn FnMut(&AskContext) -> bool + Send>;

/// Options controlling apply behaviour (`-y`, `--force`, TTY, ask callback).
pub struct ApplyOptions {
    pub yes: bool,
    pub force: bool,
    pub interactive: bool,
    pub ask: Option<AskFn>,
    pub cli_version: String,
}

impl std::fmt::Debug for ApplyOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplyOptions")
            .field("yes", &self.yes)
            .field("force", &self.force)
            .field("interactive", &self.interactive)
            .field("ask", &self.ask.as_ref().map(|_| "<AskFn>"))
            .field("cli_version", &self.cli_version)
            .finish()
    }
}

/// Resolve Keep vs Replace for one asset status (matrix BLUEPRINT-022 §5.1).
pub fn resolve_action(
    status: AssetUpdateStatus,
    opts: &ApplyOptions,
    batch_replace_customized: bool,
) -> ApplyAction {
    match status {
        AssetUpdateStatus::Identical => ApplyAction::Keep,
        AssetUpdateStatus::Missing | AssetUpdateStatus::Apply => ApplyAction::Replace,
        AssetUpdateStatus::Customized => {
            if opts.force {
                ApplyAction::Replace
            } else if opts.yes || !opts.interactive {
                ApplyAction::Keep
            } else if batch_replace_customized {
                ApplyAction::Replace
            } else {
                ApplyAction::Keep
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(force: bool, yes: bool, interactive: bool) -> ApplyOptions {
        ApplyOptions {
            yes,
            force,
            interactive,
            ask: None,
            cli_version: "0.1.0-alpha.0".into(),
        }
    }

    #[test]
    fn resolve_action_matrix_all_rows() {
        use AssetUpdateStatus::*;

        // identical | * | * | * | * → Keep
        for (force, yes, interactive, batch) in [
            (false, false, false, false),
            (true, true, true, true),
            (false, true, false, true),
        ] {
            assert_eq!(
                resolve_action(Identical, &opts(force, yes, interactive), batch),
                ApplyAction::Keep
            );
        }

        // missing / apply | * → Replace
        for status in [Missing, Apply] {
            for (force, yes, interactive, batch) in
                [(false, false, false, false), (true, true, true, true)]
            {
                assert_eq!(
                    resolve_action(status, &opts(force, yes, interactive), batch),
                    ApplyAction::Replace
                );
            }
        }

        // customized + force → Replace (force wins over yes)
        assert_eq!(
            resolve_action(Customized, &opts(true, false, false), false),
            ApplyAction::Replace
        );
        assert_eq!(
            resolve_action(Customized, &opts(true, true, true), false),
            ApplyAction::Replace
        );

        // customized + !force + yes → Keep
        assert_eq!(
            resolve_action(Customized, &opts(false, true, false), true),
            ApplyAction::Keep
        );
        assert_eq!(
            resolve_action(Customized, &opts(false, true, true), true),
            ApplyAction::Keep
        );

        // customized + !force + !yes + !interactive → Keep
        assert_eq!(
            resolve_action(Customized, &opts(false, false, false), true),
            ApplyAction::Keep
        );
        assert_eq!(
            resolve_action(Customized, &opts(false, false, false), false),
            ApplyAction::Keep
        );

        // customized + interactive + batch true/false
        assert_eq!(
            resolve_action(Customized, &opts(false, false, true), true),
            ApplyAction::Replace
        );
        assert_eq!(
            resolve_action(Customized, &opts(false, false, true), false),
            ApplyAction::Keep
        );
    }
}
