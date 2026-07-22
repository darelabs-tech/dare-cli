//! Deterministic DAG visualization (Mermaid / DOT / Excalidraw) — microplano 027.

use std::collections::BTreeMap;

use dare_contracts::{DagDocument, RuntimeStateV1};
use serde_json::{json, Value};

use crate::graph::{compute_ranks, iter_task_views, DagGraphError};
use crate::status::TaskStatus;

/// Soft cap for rendered body size (bytes).
pub const OUTPUT_CAP: usize = 2_097_152;

/// Default Unicode scalar truncation for node titles.
pub const TITLE_MAX_DEFAULT: usize = 40;

pub const EXCAL_W: f64 = 120.0;
pub const EXCAL_H: f64 = 60.0;
pub const EXCAL_DX: f64 = 200.0;
pub const EXCAL_DY: f64 = 100.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VizFormat {
    Mermaid,
    Dot,
    Excalidraw,
}

impl VizFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            VizFormat::Mermaid => "mermaid",
            VizFormat::Dot => "dot",
            VizFormat::Excalidraw => "excalidraw",
        }
    }

    /// Exact lowercase only (`mermaid` | `dot` | `excalidraw`).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "mermaid" => Some(VizFormat::Mermaid),
            "dot" => Some(VizFormat::Dot),
            "excalidraw" => Some(VizFormat::Excalidraw),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VizOptions {
    pub title_max: usize,
    pub state: Option<RuntimeStateV1>,
}

impl Default for VizOptions {
    fn default() -> Self {
        Self {
            title_max: TITLE_MAX_DEFAULT,
            state: None,
        }
    }
}

#[derive(Debug, Clone)]
struct VizNode {
    id: String,
    title: String,
    complexity: String,
    status: TaskStatus,
    rank: u32,
    alias: String,
}

struct VizGraph {
    nodes: Vec<VizNode>,
    edges: Vec<(String, String)>,
}

/// Render `doc` in `format`. Pure (no FS). Propagates cycle / missing-dep from ranks.
pub fn render(
    doc: &DagDocument,
    format: VizFormat,
    opts: &VizOptions,
) -> Result<String, DagGraphError> {
    let graph = build_viz_graph(doc, opts)?;
    let body = match format {
        VizFormat::Mermaid => render_mermaid(&graph),
        VizFormat::Dot => render_dot(&graph),
        VizFormat::Excalidraw => render_excalidraw(&graph),
    };
    apply_output_cap(body, OUTPUT_CAP)
}

fn apply_output_cap(body: String, cap: usize) -> Result<String, DagGraphError> {
    if body.len() > cap {
        return Err(DagGraphError::InvalidDag {
            message: "viz output too large".into(),
        });
    }
    Ok(body)
}

fn build_viz_graph(doc: &DagDocument, opts: &VizOptions) -> Result<VizGraph, DagGraphError> {
    let ranks = compute_ranks(doc)?;
    let complexity = complexity_map(doc);
    let views = iter_task_views(doc);

    let mut nodes: Vec<VizNode> = views
        .iter()
        .map(|t| {
            let rank = *ranks.get(&t.id).unwrap_or(&0);
            VizNode {
                id: t.id.clone(),
                title: truncate_title(&t.title, opts.title_max),
                complexity: complexity
                    .get(&t.id)
                    .cloned()
                    .unwrap_or_else(|| "UNKNOWN".into()),
                status: status_of(opts, &t.id),
                rank,
                alias: mermaid_alias(&t.id),
            }
        })
        .collect();

    nodes.sort_by(|a, b| a.rank.cmp(&b.rank).then_with(|| a.id.cmp(&b.id)));

    let mut edges: Vec<(String, String)> = Vec::new();
    for t in &views {
        for dep in &t.depends_on {
            edges.push((dep.clone(), t.id.clone()));
        }
    }
    edges.sort();

    Ok(VizGraph { nodes, edges })
}

fn status_of(opts: &VizOptions, id: &str) -> TaskStatus {
    opts.state
        .as_ref()
        .and_then(|s| s.tasks.get(id))
        .and_then(|t| TaskStatus::parse(&t.status).ok())
        .unwrap_or(TaskStatus::Pending)
}

fn complexity_map(doc: &DagDocument) -> BTreeMap<String, String> {
    match doc {
        DagDocument::V21(d) => d
            .tasks
            .iter()
            .map(|t| (t.id.clone(), t.complexity.clone()))
            .collect(),
        DagDocument::Legacy(d) => d
            .tasks
            .iter()
            .map(|(id, t)| (id.clone(), t.complexity.clone()))
            .collect(),
    }
}

fn truncate_title(title: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let count = title.chars().count();
    if count <= max {
        return title.to_string();
    }
    let keep = max.saturating_sub(1);
    let mut out: String = title.chars().take(keep).collect();
    out.push('…');
    out
}

/// Mermaid/DOT id: `^[a-zA-Z_][a-zA-Z0-9_]*$`
fn ident_ok(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn mermaid_alias(id: &str) -> String {
    let replaced: String = id
        .chars()
        .map(|c| match c {
            '-' => '_',
            c if c.is_ascii_alphanumeric() || c == '_' => c,
            _ => '_',
        })
        .collect();
    if ident_ok(&replaced) {
        replaced
    } else {
        format!("n_{replaced}")
    }
}

fn escape_mermaid_label(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "#quot;")
        .replace(']', "#93;")
        .replace('[', "#91;")
}

fn escape_dot_label(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn complexity_fill(complexity: &str) -> &'static str {
    match complexity {
        "LOW" => "#e3f2fd",
        "MED" => "#fff3e0",
        "HIGH" => "#fce4ec",
        _ => "#eeeeee",
    }
}

fn status_stroke(status: TaskStatus) -> (&'static str, &'static str) {
    match status {
        TaskStatus::Pending => ("#9e9e9e", "solid"),
        TaskStatus::Running => ("#1976d2", "dashed"),
        TaskStatus::Done => ("#2e7d32", "solid"),
        TaskStatus::Failed => ("#c62828", "solid"),
        TaskStatus::Skipped => ("#757575", "dashed"),
    }
}

fn render_mermaid(graph: &VizGraph) -> String {
    let mut out = String::from("flowchart TB\n");
    let mut by_rank: BTreeMap<u32, Vec<&VizNode>> = BTreeMap::new();
    for n in &graph.nodes {
        by_rank.entry(n.rank).or_default().push(n);
    }
    for (rank, nodes) in &by_rank {
        out.push_str(&format!("  subgraph rank_{rank}[\"Rank {rank}\"]\n"));
        for n in nodes {
            let label = format!(
                "{}<br/>{}",
                escape_mermaid_label(&n.id),
                escape_mermaid_label(&n.title)
            );
            out.push_str(&format!("    {}[\"{}\"]\n", n.alias, label));
        }
        out.push_str("  end\n");
    }
    let alias_of: BTreeMap<&str, &str> = graph
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n.alias.as_str()))
        .collect();
    for (from, to) in &graph.edges {
        let fa = alias_of
            .get(from.as_str())
            .copied()
            .unwrap_or(from.as_str());
        let ta = alias_of.get(to.as_str()).copied().unwrap_or(to.as_str());
        out.push_str(&format!("  {fa} --> {ta}\n"));
    }
    out
}

fn render_dot(graph: &VizGraph) -> String {
    let mut out = String::from("digraph dare_dag {\n  rankdir=TB;\n");
    for n in &graph.nodes {
        let label = format!(
            "{}\\n{}",
            escape_dot_label(&n.id),
            escape_dot_label(&n.title)
        );
        out.push_str(&format!("  {} [label=\"{}\"];\n", n.alias, label));
    }
    let alias_of: BTreeMap<&str, &str> = graph
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n.alias.as_str()))
        .collect();
    for (from, to) in &graph.edges {
        let fa = alias_of
            .get(from.as_str())
            .copied()
            .unwrap_or(from.as_str());
        let ta = alias_of.get(to.as_str()).copied().unwrap_or(to.as_str());
        out.push_str(&format!("  {fa} -> {ta};\n"));
    }
    out.push('}');
    out.push('\n');
    out
}

fn render_excalidraw(graph: &VizGraph) -> String {
    let mut elements: Vec<Value> = Vec::new();
    let mut pos: BTreeMap<&str, (f64, f64)> = BTreeMap::new();

    let mut by_rank: BTreeMap<u32, Vec<&VizNode>> = BTreeMap::new();
    for n in &graph.nodes {
        by_rank.entry(n.rank).or_default().push(n);
    }

    for (rank, nodes) in &by_rank {
        for (idx, n) in nodes.iter().enumerate() {
            let x = 40.0 + f64::from(*rank) * EXCAL_DX;
            let y = 40.0 + (idx as f64) * EXCAL_DY;
            pos.insert(n.id.as_str(), (x, y));
            let (stroke, stroke_style) = status_stroke(n.status);
            let fill = complexity_fill(&n.complexity);
            let label = format!("{}\n{}", n.id, n.title);
            elements.push(json!({
                "type": "rectangle",
                "id": format!("task-{}", n.id),
                "x": x,
                "y": y,
                "width": EXCAL_W,
                "height": EXCAL_H,
                "backgroundColor": fill,
                "strokeColor": stroke,
                "strokeStyle": stroke_style,
                "label": label,
            }));
            elements.push(json!({
                "type": "text",
                "id": format!("text-{}", n.id),
                "x": x + 8.0,
                "y": y + 16.0,
                "text": label,
                "fontSize": 14,
            }));
        }
    }

    for (from, to) in &graph.edges {
        let Some(&(fx, fy)) = pos.get(from.as_str()) else {
            continue;
        };
        let Some(&(tx, ty)) = pos.get(to.as_str()) else {
            continue;
        };
        let x1 = fx + EXCAL_W;
        let y1 = fy + EXCAL_H / 2.0;
        let x2 = tx;
        let y2 = ty + EXCAL_H / 2.0;
        elements.push(json!({
            "type": "arrow",
            "id": format!("arrow-{from}-{to}"),
            "x": x1,
            "y": y1,
            "points": [[0.0, 0.0], [x2 - x1, y2 - y1]],
            "strokeColor": "#9e9e9e",
            "strokeStyle": "solid",
        }));
    }

    let doc = json!({
        "type": "excalidraw",
        "version": 2,
        "source": "dare-cli",
        "elements": elements,
        "appState": {},
    });
    // Compact deterministic JSON (no whitespace variance).
    serde_json::to_string(&doc).unwrap_or_else(|_| "{}".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_contracts::parse_dag_yaml;
    use std::fs;
    use std::path::PathBuf;

    fn fixture(name: &str) -> String {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("../../tests/fixtures/dag");
        p.push(name);
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
    }

    fn norm(s: &str) -> String {
        s.replace("\r\n", "\n")
    }

    #[test]
    fn render_mermaid_matches_golden() {
        let doc = parse_dag_yaml(&fixture("viz/sample.v21.yaml")).unwrap();
        let body = render(&doc, VizFormat::Mermaid, &VizOptions::default()).unwrap();
        let golden = fixture("viz/sample.mermaid.golden");
        assert_eq!(norm(&body), norm(&golden));
    }

    #[test]
    fn render_dot_matches_golden() {
        let doc = parse_dag_yaml(&fixture("viz/sample.v21.yaml")).unwrap();
        let body = render(&doc, VizFormat::Dot, &VizOptions::default()).unwrap();
        let golden = fixture("viz/sample.dot.golden");
        assert_eq!(norm(&body), norm(&golden));
    }

    #[test]
    fn render_excalidraw_matches_golden() {
        let doc = parse_dag_yaml(&fixture("viz/sample.v21.yaml")).unwrap();
        let body = render(&doc, VizFormat::Excalidraw, &VizOptions::default()).unwrap();
        let golden = fixture("viz/sample.excalidraw.golden");
        assert_eq!(norm(&body), norm(&golden));
        let v: Value = serde_json::from_str(&body).expect("parseable JSON");
        assert_eq!(v["type"], "excalidraw");
        assert_eq!(v["version"], 2);
        assert_eq!(v["source"], "dare-cli");
        assert!(v.get("updated").is_none());
        assert!(v.get("seed").is_none());
    }

    #[test]
    fn render_excalidraw_complexity_colors() {
        let doc = parse_dag_yaml(&fixture("viz/sample.v21.yaml")).unwrap();
        let body = render(&doc, VizFormat::Excalidraw, &VizOptions::default()).unwrap();
        let v: Value = serde_json::from_str(&body).unwrap();
        let els = v["elements"].as_array().unwrap();
        let rect_a = els
            .iter()
            .find(|e| e["id"] == "task-task-a")
            .expect("rect a");
        assert_eq!(rect_a["backgroundColor"], "#e3f2fd"); // LOW
        let rect_d = els
            .iter()
            .find(|e| e["id"] == "task-task-d")
            .expect("rect d");
        assert_eq!(rect_d["backgroundColor"], "#fce4ec"); // HIGH
    }

    #[test]
    fn render_status_soft_without_state() {
        let doc = parse_dag_yaml(&fixture("viz/sample.v21.yaml")).unwrap();
        let body = render(&doc, VizFormat::Excalidraw, &VizOptions::default()).unwrap();
        let v: Value = serde_json::from_str(&body).unwrap();
        for e in v["elements"].as_array().unwrap() {
            if e["type"] == "rectangle" {
                assert_eq!(e["strokeColor"], "#9e9e9e");
                assert_eq!(e["strokeStyle"], "solid");
            }
        }
    }

    #[test]
    fn render_output_cap_errors() {
        let err = apply_output_cap("x".repeat(10), 5).unwrap_err();
        match err {
            DagGraphError::InvalidDag { message } => {
                assert_eq!(message, "viz output too large");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn render_mermaid_cycle_errors() {
        let doc = parse_dag_yaml(&fixture("cycle.v21.yaml")).unwrap();
        let err = render(&doc, VizFormat::Mermaid, &VizOptions::default()).unwrap_err();
        match err {
            DagGraphError::Cycle { .. } => {}
            other => panic!("expected Cycle, got {other:?}"),
        }
    }

    #[test]
    fn render_title_truncates() {
        let long = "A".repeat(50);
        let yaml = format!(
            r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-a
    title: "{long}"
    depends_on: []
    complexity: LOW
    subtask_prompt: x
"#
        );
        let doc = parse_dag_yaml(&yaml).unwrap();
        let body = render(&doc, VizFormat::Mermaid, &VizOptions::default()).unwrap();
        assert!(body.contains('…'), "body={body}");
        assert!(!body.contains(&long));
        let title_part = body
            .lines()
            .find(|l| l.contains("task_a["))
            .expect("node line");
        let br = title_part.find("<br/>").unwrap();
        let after = &title_part[br + 5..];
        let end = after.find("\"]").unwrap();
        let title = &after[..end];
        assert_eq!(title.chars().count(), 40);
        assert!(title.ends_with('…'));
    }

    #[test]
    fn render_sanitize_kebab_id() {
        let yaml = r#"
title: "T"
version: "1.0.0"
tasks:
  - id: task-a
    title: Alpha
    depends_on: []
    complexity: LOW
    subtask_prompt: x
  - id: task-b
    title: Beta
    depends_on: [task-a]
    complexity: MED
    subtask_prompt: y
"#;
        let doc = parse_dag_yaml(yaml).unwrap();
        let body = render(&doc, VizFormat::Mermaid, &VizOptions::default()).unwrap();
        assert!(body.contains("task_a[\"task-a<br/>Alpha\"]"));
        assert!(body.contains("task_b[\"task-b<br/>Beta\"]"));
        assert!(body.contains("task_a --> task_b"));
        assert!(!body.contains("subtask_prompt"));
    }

    #[test]
    fn viz_format_parse_exact() {
        assert_eq!(VizFormat::parse("mermaid"), Some(VizFormat::Mermaid));
        assert_eq!(VizFormat::parse("Mermaid"), None);
        assert_eq!(VizFormat::parse("dot"), Some(VizFormat::Dot));
        assert_eq!(VizFormat::as_str(&VizFormat::Excalidraw), "excalidraw");
    }
}
