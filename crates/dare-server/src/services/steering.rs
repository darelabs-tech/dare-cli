//! Steering show via `dare_steering`.

use dare_core::CoreResult;
use dare_steering::{show_steering, SteeringShowReport};

use crate::services::ServiceCtx;

pub fn steering_show(ctx: &ServiceCtx, file: &str) -> CoreResult<SteeringShowReport> {
    show_steering(&ctx.root, file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ProjectRoot;

    #[test]
    fn env_file_forbidden() {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let ctx = ServiceCtx::new(root);
        let err = steering_show(&ctx, ".env").unwrap_err();
        assert!(err.message().contains(".env"));
    }
}
