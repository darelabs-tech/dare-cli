pub mod echo;

use serde_json::{json, Value};

/// Machine-readable tool inventory for `--json --list-tools` (M-03).
pub fn inventory_json() -> Value {
    json!({
        "tools": [
            {
                "name": "echo",
                "description": "Returns its input. Canonical smoke test for an MCP server.",
                "inputSchema": {
                    "type": "object",
                    "required": ["text"],
                    "properties": {
                        "text": { "type": "string", "minLength": 1 }
                    }
                }
            }
        ]
    })
}
