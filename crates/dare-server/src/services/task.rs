//! Task get/put wrapping `tasks_md`.

use dare_core::CoreResult;

use crate::services::ServiceCtx;
use crate::tasks_md::{
    get_task_view, put_task_status, reject_path_escape_id, validate_task_id, TaskView,
};

fn check_id(id: &str) -> CoreResult<()> {
    reject_path_escape_id(id)?;
    validate_task_id(id)
}

pub fn task_get(ctx: &ServiceCtx, id: &str) -> CoreResult<TaskView> {
    check_id(id)?;
    get_task_view(&ctx.root, id)
}

pub fn task_put(ctx: &ServiceCtx, id: &str, status: &str) -> CoreResult<TaskView> {
    check_id(id)?;
    put_task_status(&ctx.root, id, status.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tasks_md::MSG_PATH_ESCAPE;
    use dare_core::ProjectRoot;

    #[test]
    fn path_escape_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let ctx = ServiceCtx::new(root);
        let err = task_get(&ctx, "a..b").unwrap_err();
        assert_eq!(err.message(), MSG_PATH_ESCAPE);
    }

    #[test]
    fn put_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let dare = dir.path().join("DARE");
        std::fs::create_dir_all(&dare).unwrap();
        std::fs::write(
            dare.join("TASKS.md"),
            "| id | title | status |\n| mp052-001 | Services | ⏳ PENDING |\n",
        )
        .unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let ctx = ServiceCtx::new(root);
        let view = task_put(&ctx, "mp052-001", "DONE").unwrap();
        assert_eq!(view.status, "DONE");
        let got = task_get(&ctx, "mp052-001").unwrap();
        assert_eq!(got.status, "DONE");
    }
}
