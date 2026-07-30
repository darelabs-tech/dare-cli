pub mod blueprint;
pub mod context;
pub mod dag;
pub mod dashboard;
pub mod graph;
pub mod health;
pub mod project;
pub mod steering;
pub mod tasks;
pub mod tools;

pub use health::health;

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

/// REST legacy surface (AppMode::Rest only).
pub fn rest_router() -> Router<AppState> {
    Router::new()
        .route("/tools", get(tools::tools))
        .route("/context/query", post(context::context_query))
        .route("/blueprint", get(blueprint::blueprint))
        .route("/dag", get(dag::dag))
        .route("/tasks/{id}", get(tasks::get_task).put(tasks::put_task))
        .route("/graph/locate", post(graph::graph_locate))
        .route("/graph/traverse", post(graph::graph_traverse))
        .route("/graph/map-requirement", post(graph::graph_map_requirement))
        .route("/project", get(project::project))
        .route("/steering", get(steering::steering))
}
