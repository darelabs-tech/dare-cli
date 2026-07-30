//! DAG load as JSON value.

use dare_contracts::{load_dag, DagDocument};
use dare_core::{CoreError, CoreResult, SafeRelativePath};
use serde_json::Value;

use crate::http_map::DAG_REL;
use crate::services::ServiceCtx;

pub fn dag_load_json(ctx: &ServiceCtx) -> CoreResult<Value> {
    let rel = SafeRelativePath::new(DAG_REL)?;
    let doc = load_dag(&ctx.root, &rel)?;
    match doc {
        DagDocument::V21(d) => serde_json::to_value(d),
        DagDocument::Legacy(d) => serde_json::to_value(d),
    }
    .map_err(|e| CoreError::internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ProjectRoot;

    #[test]
    fn missing_dag_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let ctx = ServiceCtx::new(root);
        let err = dag_load_json(&ctx).unwrap_err();
        assert_eq!(err.kind(), dare_core::ErrorKind::NotFound);
    }
}
