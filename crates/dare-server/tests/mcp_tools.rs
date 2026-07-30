//! In-process MCP tools tests (mp052-003).

use dare_core::ProjectRoot;
use dare_server::mcp::{McpHandler, TOOL_NAMES};
use dare_server::{MSG_INVALID_CONTEXT_TYPE, MSG_PATH_ESCAPE};
use rmcp::model::{ContentBlock, ErrorCode};
use serde_json::{json, Map, Value};

fn handler_for(dir: &tempfile::TempDir) -> McpHandler {
    let root = ProjectRoot::new(dir.path()).expect("project root");
    McpHandler::from_root(root)
}

fn args_obj(v: Value) -> Option<rmcp::model::JsonObject> {
    match v {
        Value::Object(map) => Some(map),
        _ => Some(Map::new()),
    }
}

fn text_json(result: &rmcp::model::CallToolResult) -> Value {
    assert_eq!(result.is_error, Some(false));
    let text = result
        .content
        .first()
        .and_then(ContentBlock::as_text)
        .map(|t| t.text.as_str())
        .expect("text content");
    serde_json::from_str(text).expect("envelope json")
}

#[test]
fn list_order_10() {
    let dir = tempfile::tempdir().unwrap();
    let handler = handler_for(&dir);
    let listed = handler.list_tools_now();
    let names: Vec<&str> = listed.tools.iter().map(|t| t.name.as_ref()).collect();
    assert_eq!(names, TOOL_NAMES.to_vec());
    assert_eq!(names.len(), 10);
    assert_eq!(
        names,
        vec![
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
        ]
    );
}

#[test]
fn call_project_ok() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("DARE")).unwrap();
    let handler = handler_for(&dir);
    let result = handler
        .call_tool_now("project", Some(Map::new()))
        .expect("call project");
    let envelope = text_json(&result);
    assert_eq!(envelope["schemaVersion"], 1);
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["tool"], "project");
    assert_eq!(envelope["data"]["schemaVersion"], 1);
    assert_eq!(envelope["data"]["dareDirPresent"], true);
}

#[test]
fn context_bad_type_invalid_params() {
    let dir = tempfile::tempdir().unwrap();
    let handler = handler_for(&dir);
    let err = handler
        .call_tool_now(
            "context_query",
            args_obj(json!({"type":"foo","query":"x"})),
        )
        .expect_err("bad type must be Err");
    assert_eq!(err.code, ErrorCode::INVALID_PARAMS);
    assert_eq!(err.message.as_ref(), MSG_INVALID_CONTEXT_TYPE);
}

#[test]
fn task_get_path_escape() {
    let dir = tempfile::tempdir().unwrap();
    let handler = handler_for(&dir);
    let err = handler
        .call_tool_now("task_get", args_obj(json!({"id":"../x"})))
        .expect_err("path escape must be Err");
    assert_eq!(err.code, ErrorCode::INVALID_PARAMS);
    assert_eq!(err.message.as_ref(), MSG_PATH_ESCAPE);
}
