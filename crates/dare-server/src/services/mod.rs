//! Domain services shared by REST and (later) MCP transports.

mod dag;
mod graph;
mod project;
mod steering;
mod task;

pub use dag::dag_load_json;
pub use graph::{
    graph_locate, graph_map_requirement, graph_traverse, locate_defaults,
};
pub use project::{
    context_query, project_snapshot, read_blueprint, BlueprintDoc, ContextQueryResponse,
    ProjectSnapshot,
};
pub use steering::steering_show;
pub use task::{task_get, task_put};

use dare_core::ProjectRoot;

/// Shared execution context for domain services.
#[derive(Debug, Clone)]
pub struct ServiceCtx {
    pub root: ProjectRoot,
}

impl ServiceCtx {
    pub fn new(root: ProjectRoot) -> Self {
        Self { root }
    }
}
