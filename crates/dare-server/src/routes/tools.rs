//! `GET /tools` — static announcement of REST surface.

use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsList {
    pub schema_version: u32,
    pub tools: Vec<ToolEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolEntry {
    pub name: &'static str,
    pub method: &'static str,
    pub path: &'static str,
}

/// Frozen order (12 tools) — BLUEPRINT §5.6.
pub const FROZEN_TOOLS: &[ToolEntry] = &[
    ToolEntry {
        name: "health",
        method: "GET",
        path: "/health",
    },
    ToolEntry {
        name: "tools",
        method: "GET",
        path: "/tools",
    },
    ToolEntry {
        name: "context_query",
        method: "POST",
        path: "/context/query",
    },
    ToolEntry {
        name: "blueprint",
        method: "GET",
        path: "/blueprint",
    },
    ToolEntry {
        name: "dag",
        method: "GET",
        path: "/dag",
    },
    ToolEntry {
        name: "tasks_get",
        method: "GET",
        path: "/tasks/:id",
    },
    ToolEntry {
        name: "tasks_put",
        method: "PUT",
        path: "/tasks/:id",
    },
    ToolEntry {
        name: "graph_locate",
        method: "POST",
        path: "/graph/locate",
    },
    ToolEntry {
        name: "graph_map_requirement",
        method: "POST",
        path: "/graph/map-requirement",
    },
    ToolEntry {
        name: "graph_traverse",
        method: "POST",
        path: "/graph/traverse",
    },
    ToolEntry {
        name: "project",
        method: "GET",
        path: "/project",
    },
    ToolEntry {
        name: "steering",
        method: "GET",
        path: "/steering",
    },
];

pub async fn tools() -> Json<ToolsList> {
    Json(ToolsList {
        schema_version: 1,
        tools: FROZEN_TOOLS.to_vec(),
    })
}
