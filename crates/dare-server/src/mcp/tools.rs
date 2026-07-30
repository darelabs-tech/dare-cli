//! MCP tool definitions + dispatch (frozen order, microplano 052).

use std::sync::Arc;

use dare_graph::{DEFAULT_FANOUT, DEFAULT_MAX_HOPS};
use rmcp::handler::server::common::{schema_for_empty_input, schema_for_input};
use rmcp::model::{CallToolResult, ContentBlock, ErrorData, JsonObject, Tool};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::services::{
    context_query, dag_load_json, graph_locate, graph_map_requirement, graph_traverse,
    locate_defaults, project_snapshot, read_blueprint, steering_show, task_get, task_put,
    ServiceCtx,
};

use super::error_map::map_core_error;

/// Frozen `tools/list` order (Blueprint §0.4).
pub const TOOL_NAMES: [&str; 10] = [
    "project",
    "blueprint",
    "dag",
    "task_get",
    "task_put",
    "context_query",
    "graph_locate",
    "graph_traverse",
    "graph_map_requirement",
    "steering_show",
];

#[derive(Debug, Deserialize, JsonSchema)]
struct EmptyArgs {}

#[derive(Debug, Deserialize, JsonSchema)]
struct TaskGetArgs {
    /// Task id: `^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$` (reject `..` `/` `\`)
    id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct TaskPutArgs {
    id: String,
    /// One of: PENDING | RUNNING | DONE | FAILED | SKIPPED
    status: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ContextQueryArgs {
    /// One of: architecture | task | dependency
    #[serde(rename = "type")]
    kind: String,
    /// Trimmed length 1..=512
    query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct GraphLocateArgs {
    query: String,
    max_hops: Option<usize>,
    fanout: Option<usize>,
    limit: Option<usize>,
    decay: Option<f64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct GraphTraverseArgs {
    /// 1..=32 entries; each non-empty ≤256 chars
    seeds: Vec<String>,
    max_hops: Option<usize>,
    fanout: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct GraphMapRequirementArgs {
    query: String,
    max_hops: Option<usize>,
    fanout: Option<usize>,
    limit: Option<usize>,
    decay: Option<f64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SteeringShowArgs {
    /// Relative steering file path (`.env*` rejected)
    file: String,
}

fn schema_or_empty<T: JsonSchema + 'static>() -> Arc<JsonObject> {
    schema_for_input::<T>().unwrap_or_else(|_| schema_for_empty_input())
}

fn tool_def(name: &'static str, description: &'static str, input_schema: Arc<JsonObject>) -> Tool {
    Tool::new(name, description, input_schema)
}

/// Tool definitions in frozen order for `tools/list`.
pub fn tool_definitions() -> Vec<Tool> {
    vec![
        tool_def(
            "project",
            "Project snapshot (root, DARE dir, config, backend, graph presence)",
            schema_for_empty_input(),
        ),
        tool_def(
            "blueprint",
            "Read DARE/BLUEPRINT.md (size-capped)",
            schema_for_empty_input(),
        ),
        tool_def(
            "dag",
            "Load DARE/dare-dag.yaml as JSON",
            schema_for_empty_input(),
        ),
        tool_def(
            "task_get",
            "Get task status from DARE/TASKS.md by id",
            schema_or_empty::<TaskGetArgs>(),
        ),
        tool_def(
            "task_put",
            "Update task status in DARE/TASKS.md",
            schema_or_empty::<TaskPutArgs>(),
        ),
        tool_def(
            "context_query",
            "Query project context (architecture|task|dependency)",
            schema_or_empty::<ContextQueryArgs>(),
        ),
        tool_def(
            "graph_locate",
            "Locate nodes in the knowledge graph",
            schema_or_empty::<GraphLocateArgs>(),
        ),
        tool_def(
            "graph_traverse",
            "BFS traverse the knowledge graph from seeds",
            schema_or_empty::<GraphTraverseArgs>(),
        ),
        tool_def(
            "graph_map_requirement",
            "Locate preference for requirement nodes",
            schema_or_empty::<GraphMapRequirementArgs>(),
        ),
        tool_def(
            "steering_show",
            "Show steering file contents for a relative path",
            schema_or_empty::<SteeringShowArgs>(),
        ),
    ]
}

fn parse_args<T: for<'de> Deserialize<'de>>(
    arguments: Option<&JsonObject>,
) -> Result<T, ErrorData> {
    let value = match arguments {
        Some(obj) => Value::Object(obj.clone()),
        None => json!({}),
    };
    serde_json::from_value(value).map_err(|e| {
        ErrorData::invalid_params(
            format!("invalid tool arguments: {e}"),
            Some(json!({ "code": "invalid_input" })),
        )
    })
}

fn ok_result(tool: &str, data: impl serde::Serialize) -> Result<CallToolResult, ErrorData> {
    let envelope = json!({
        "schemaVersion": 1,
        "ok": true,
        "tool": tool,
        "data": data,
    });
    let text = serde_json::to_string(&envelope).map_err(|e| {
        ErrorData::internal_error(
            "failed to serialize tool result",
            Some(json!({ "code": "internal", "reason": e.to_string() })),
        )
    })?;
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

/// Dispatch a tool call to the shared services layer.
pub fn dispatch(
    ctx: &ServiceCtx,
    name: &str,
    arguments: Option<&JsonObject>,
) -> Result<CallToolResult, ErrorData> {
    match name {
        "project" => {
            let _: EmptyArgs = parse_args(arguments)?;
            let data = project_snapshot(ctx).map_err(map_core_error)?;
            ok_result("project", data)
        }
        "blueprint" => {
            let _: EmptyArgs = parse_args(arguments)?;
            let data = read_blueprint(ctx).map_err(map_core_error)?;
            ok_result("blueprint", data)
        }
        "dag" => {
            let _: EmptyArgs = parse_args(arguments)?;
            let data = dag_load_json(ctx).map_err(map_core_error)?;
            ok_result("dag", data)
        }
        "task_get" => {
            let args: TaskGetArgs = parse_args(arguments)?;
            let data = task_get(ctx, &args.id).map_err(map_core_error)?;
            ok_result("task_get", data)
        }
        "task_put" => {
            let args: TaskPutArgs = parse_args(arguments)?;
            let data = task_put(ctx, &args.id, &args.status).map_err(map_core_error)?;
            ok_result("task_put", data)
        }
        "context_query" => {
            let args: ContextQueryArgs = parse_args(arguments)?;
            let data = context_query(ctx, &args.kind, &args.query).map_err(map_core_error)?;
            ok_result("context_query", data)
        }
        "graph_locate" => {
            let args: GraphLocateArgs = parse_args(arguments)?;
            let opts = locate_defaults(
                args.query,
                args.max_hops,
                args.fanout,
                args.limit,
                args.decay,
            );
            let hits = graph_locate(ctx, opts).map_err(map_core_error)?;
            ok_result(
                "graph_locate",
                json!({ "schemaVersion": 1, "hits": hits }),
            )
        }
        "graph_traverse" => {
            let args: GraphTraverseArgs = parse_args(arguments)?;
            let nodes = graph_traverse(
                ctx,
                &args.seeds,
                args.max_hops.unwrap_or(DEFAULT_MAX_HOPS),
                args.fanout.unwrap_or(DEFAULT_FANOUT),
            )
            .map_err(map_core_error)?;
            ok_result(
                "graph_traverse",
                json!({ "schemaVersion": 1, "nodes": nodes }),
            )
        }
        "graph_map_requirement" => {
            let args: GraphMapRequirementArgs = parse_args(arguments)?;
            let opts = locate_defaults(
                args.query,
                args.max_hops,
                args.fanout,
                args.limit,
                args.decay,
            );
            let hits = graph_map_requirement(ctx, opts).map_err(map_core_error)?;
            ok_result(
                "graph_map_requirement",
                json!({ "schemaVersion": 1, "hits": hits }),
            )
        }
        "steering_show" => {
            let args: SteeringShowArgs = parse_args(arguments)?;
            let data = steering_show(ctx, &args.file).map_err(map_core_error)?;
            ok_result("steering_show", data)
        }
        other => Err(ErrorData::invalid_params(
            format!("unknown tool: {other}"),
            Some(json!({ "code": "invalid_input" })),
        )),
    }
}
