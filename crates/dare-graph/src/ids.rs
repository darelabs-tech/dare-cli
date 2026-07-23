//! Canonical GraphRAG node/edge IDs (parity with TS 3.18.1).

use dare_contracts::{
    canonical_edge_id as contracts_edge_id, canonical_file_node_id as contracts_file_id,
    canonical_task_node_id as contracts_task_id,
};

/// Normalize a filesystem path for graph IDs: `\` → `/`, lowercase Windows drive.
pub fn normalize_graph_path(path: &str) -> String {
    let mut posix = path.replace('\\', "/");
    let bytes = posix.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        let first = bytes[0].to_ascii_lowercase() as char;
        posix.replace_range(0..1, &first.to_string());
    }
    posix
}

pub fn to_qualified_name(file_path: &str, symbol: &str) -> String {
    format!("{}::{symbol}", normalize_graph_path(file_path))
}

pub fn canonical_task_node_id(task_id: &str) -> String {
    contracts_task_id(task_id)
}

pub fn canonical_file_node_id(posix_path: &str) -> String {
    contracts_file_id(&normalize_graph_path(posix_path))
}

pub fn canonical_code_symbol_node_id(file_path: &str, symbol: &str) -> String {
    format!("code_symbol:{}", to_qualified_name(file_path, symbol))
}

pub fn canonical_requirement_node_id(req_id: &str) -> String {
    format!("requirement:{req_id}")
}

pub fn canonical_pattern_node_id(pattern_id: &str) -> String {
    format!("pattern:{pattern_id}")
}

pub fn canonical_edge_id(kind: &str, from: &str, to: &str) -> String {
    contracts_edge_id(kind, from, to)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_match_legacy_shapes() {
        assert_eq!(canonical_task_node_id("task-001"), "task:task-001");
        assert_eq!(canonical_file_node_id(r"src\math.ts"), "file:src/math.ts");
        assert_eq!(
            canonical_code_symbol_node_id("src/math.ts", "add"),
            "code_symbol:src/math.ts::add"
        );
        assert_eq!(canonical_requirement_node_id("RF-01"), "requirement:RF-01");
        assert_eq!(
            canonical_pattern_node_id("naming-idiom:service-suffix"),
            "pattern:naming-idiom:service-suffix"
        );
        assert_eq!(
            canonical_edge_id("depends_on", "task:a", "task:b"),
            "depends_on:task:a->task:b"
        );
    }

    #[test]
    fn windows_drive_lowercased() {
        assert_eq!(normalize_graph_path(r"C:\Proj\a.ts"), "c:/Proj/a.ts");
    }
}
