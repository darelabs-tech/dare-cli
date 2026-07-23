//! AST walk extractors for HTTP endpoints and type-like entities.

use tree_sitter::{Node, Tree};

use crate::model::{Entity, HttpEndpoint, Language, SourceKind};

const HTTP_METHODS: &[&str] = &["get", "post", "put", "patch", "delete", "options", "head"];

/// Extract endpoints and entities from a parsed tree.
pub fn extract_from_tree(
    lang: Language,
    source: &str,
    tree: &Tree,
) -> (Vec<HttpEndpoint>, Vec<Entity>) {
    let root = tree.root_node();
    let mut endpoints = Vec::new();
    let mut entities = Vec::new();
    walk(lang, source, root, &mut endpoints, &mut entities);
    (endpoints, entities)
}

fn walk(
    lang: Language,
    source: &str,
    node: Node<'_>,
    endpoints: &mut Vec<HttpEndpoint>,
    entities: &mut Vec<Entity>,
) {
    maybe_endpoint(lang, source, node, endpoints);
    maybe_entity(lang, source, node, entities);

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(lang, source, child, endpoints, entities);
    }
}

fn maybe_endpoint(lang: Language, source: &str, node: Node<'_>, out: &mut Vec<HttpEndpoint>) {
    let kind = node.kind();
    match lang {
        Language::TypeScript | Language::Tsx | Language::JavaScript => {
            // member call: app.get('/x') / router.post("/y")
            if kind == "call_expression" {
                if let Some(ep) = js_call_endpoint(source, node) {
                    out.push(ep);
                }
            }
            // decorator: @Get('path')
            if kind == "decorator" || kind == "call_expression" {
                if let Some(ep) = js_decorator_endpoint(source, node) {
                    out.push(ep);
                }
            }
        }
        Language::Python => {
            if kind == "decorator" {
                if let Some(ep) = python_decorator_endpoint(source, node) {
                    out.push(ep);
                }
            }
        }
        Language::Php => {
            if kind == "scoped_call_expression" || kind == "member_call_expression" {
                if let Some(ep) = php_route_endpoint(source, node) {
                    out.push(ep);
                }
            }
        }
        Language::Go => {
            if kind == "call_expression" {
                if let Some(ep) = go_call_endpoint(source, node) {
                    out.push(ep);
                }
            }
        }
        Language::Ruby => {
            if kind == "call" {
                if let Some(ep) = ruby_call_endpoint(source, node) {
                    out.push(ep);
                }
            }
        }
        Language::Rust => {
            if kind == "call_expression" {
                if let Some(ep) = rust_route_endpoint(source, node) {
                    out.push(ep);
                }
            }
            if kind == "attribute_item" {
                if let Some(ep) = rust_attr_endpoint(source, node) {
                    out.push(ep);
                }
            }
        }
    }
}

fn maybe_entity(lang: Language, source: &str, node: Node<'_>, out: &mut Vec<Entity>) {
    let kind = node.kind();
    let line = node.start_position().row as u32 + 1;

    let push = |name: String, entity_kind: &str, out: &mut Vec<Entity>| {
        if !name.is_empty() {
            out.push(Entity {
                name,
                kind: entity_kind.to_string(),
                line,
                source: SourceKind::Ast,
            });
        }
    };

    match lang {
        Language::TypeScript | Language::Tsx | Language::JavaScript => match kind {
            "class_declaration" => {
                if let Some(n) = field_text(source, node, "name") {
                    push(n, "class", out);
                }
            }
            "interface_declaration" => {
                if let Some(n) = field_text(source, node, "name") {
                    push(n, "interface", out);
                }
            }
            "enum_declaration" => {
                if let Some(n) = field_text(source, node, "name") {
                    push(n, "enum", out);
                }
            }
            _ => {}
        },
        Language::Python => {
            if kind == "class_definition" {
                if let Some(n) = field_text(source, node, "name") {
                    push(n, "class", out);
                }
            }
        }
        Language::Php => {
            if kind == "class_declaration" {
                if let Some(n) = field_text(source, node, "name") {
                    push(n, "class", out);
                }
            }
        }
        Language::Go => {
            if kind == "type_declaration" {
                // type Foo struct { ... }
                if let Some(spec) = node.child_by_field_name("name") {
                    // go grammar: type_spec under type_declaration
                    let _ = spec;
                }
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "type_spec" {
                        if let Some(name) = field_text(source, child, "name") {
                            let type_node = child.child_by_field_name("type");
                            let ek = match type_node.map(|t| t.kind()) {
                                Some("struct_type") => "struct",
                                Some("interface_type") => "interface",
                                _ => "type",
                            };
                            push(name, ek, out);
                        }
                    }
                }
            }
        }
        Language::Ruby => {
            if kind == "class" {
                if let Some(n) = field_text(source, node, "name") {
                    push(n, "class", out);
                }
            }
        }
        Language::Rust => match kind {
            "struct_item" => {
                if let Some(n) = field_text(source, node, "name") {
                    push(n, "struct", out);
                }
            }
            "enum_item" => {
                if let Some(n) = field_text(source, node, "name") {
                    push(n, "enum", out);
                }
            }
            "trait_item" => {
                if let Some(n) = field_text(source, node, "name") {
                    push(n, "interface", out);
                }
            }
            _ => {}
        },
    }
}

fn js_call_endpoint(source: &str, node: Node<'_>) -> Option<HttpEndpoint> {
    let func = node.child_by_field_name("function")?;
    let method = match func.kind() {
        "member_expression" => field_text(source, func, "property")?,
        "identifier" => node_text(source, func),
        _ => return None,
    };
    let method_l = method.to_ascii_lowercase();
    if !HTTP_METHODS.contains(&method_l.as_str()) {
        return None;
    }
    let args = node.child_by_field_name("arguments")?;
    let path = first_string_arg(source, args)?;
    Some(HttpEndpoint {
        method: method_l.to_ascii_uppercase(),
        path: normalize_path(&path),
        line: node.start_position().row as u32 + 1,
        source: SourceKind::Ast,
    })
}

fn js_decorator_endpoint(source: &str, node: Node<'_>) -> Option<HttpEndpoint> {
    // @Get('x') → call_expression inside decorator, or bare call
    let call = if node.kind() == "decorator" {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        *children.iter().find(|c| c.kind() == "call_expression")?
    } else {
        node
    };
    let func = call.child_by_field_name("function")?;
    if func.kind() != "identifier" {
        return None;
    }
    let name = node_text(source, func);
    let method_l = name.to_ascii_lowercase();
    if !HTTP_METHODS.contains(&method_l.as_str()) {
        return None;
    }
    // Nest-style decorators are capitalized Get/Post — accept if identifier matches method set
    let args = call.child_by_field_name("arguments")?;
    let path = first_string_arg(source, args)?;
    Some(HttpEndpoint {
        method: method_l.to_ascii_uppercase(),
        path: normalize_path(&path),
        line: node.start_position().row as u32 + 1,
        source: SourceKind::Ast,
    })
}

fn python_decorator_endpoint(source: &str, node: Node<'_>) -> Option<HttpEndpoint> {
    // @app.get("/x") or @router.post('/y')
    let mut cursor = node.walk();
    let call = node.children(&mut cursor).find(|c| c.kind() == "call")?;
    let func = call.child_by_field_name("function")?;
    let method = if func.kind() == "attribute" {
        field_text(source, func, "attribute")?
    } else {
        return None;
    };
    let method_l = method.to_ascii_lowercase();
    if !HTTP_METHODS.contains(&method_l.as_str()) {
        return None;
    }
    let args = call.child_by_field_name("arguments")?;
    let path = first_string_arg(source, args)?;
    Some(HttpEndpoint {
        method: method_l.to_ascii_uppercase(),
        path: normalize_path(&path),
        line: node.start_position().row as u32 + 1,
        source: SourceKind::Ast,
    })
}

fn php_route_endpoint(source: &str, node: Node<'_>) -> Option<HttpEndpoint> {
    // Route::get('/x')
    let name = match node.kind() {
        "scoped_call_expression" => field_text(source, node, "name")?,
        "member_call_expression" => field_text(source, node, "name")?,
        _ => return None,
    };
    let method_l = name.to_ascii_lowercase();
    if !HTTP_METHODS.contains(&method_l.as_str()) {
        return None;
    }
    let args = node.child_by_field_name("arguments")?;
    let path = first_string_arg(source, args)?;
    Some(HttpEndpoint {
        method: method_l.to_ascii_uppercase(),
        path: normalize_path(&path),
        line: node.start_position().row as u32 + 1,
        source: SourceKind::Ast,
    })
}

fn go_call_endpoint(source: &str, node: Node<'_>) -> Option<HttpEndpoint> {
    let func = node.child_by_field_name("function")?;
    let method = match func.kind() {
        "selector_expression" => field_text(source, func, "field")?,
        "identifier" => node_text(source, func),
        _ => return None,
    };
    let method_l = method.to_ascii_lowercase();
    if !HTTP_METHODS.contains(&method_l.as_str()) && method_l != "handlefunc" {
        return None;
    }
    let args = node.child_by_field_name("arguments")?;
    let path = first_string_arg(source, args)?;
    let method_out = if method_l == "handlefunc" {
        "GET".to_string()
    } else {
        method_l.to_ascii_uppercase()
    };
    Some(HttpEndpoint {
        method: method_out,
        path: normalize_path(&path),
        line: node.start_position().row as u32 + 1,
        source: SourceKind::Ast,
    })
}

fn ruby_call_endpoint(source: &str, node: Node<'_>) -> Option<HttpEndpoint> {
    let method = field_text(source, node, "method").or_else(|| {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        children
            .iter()
            .find(|c| c.kind() == "identifier")
            .map(|c| node_text(source, *c))
    })?;
    let method_l = method.to_ascii_lowercase();
    if !HTTP_METHODS.contains(&method_l.as_str()) {
        return None;
    }
    let path = first_string_in_node(source, node)?;
    Some(HttpEndpoint {
        method: method_l.to_ascii_uppercase(),
        path: normalize_path(&path),
        line: node.start_position().row as u32 + 1,
        source: SourceKind::Ast,
    })
}

fn rust_route_endpoint(source: &str, node: Node<'_>) -> Option<HttpEndpoint> {
    let func = node.child_by_field_name("function")?;
    let name = match func.kind() {
        "field_expression" => field_text(source, func, "field")?,
        "identifier" => node_text(source, func),
        "scoped_identifier" => node_text(source, func),
        _ => return None,
    };
    if name != "route"
        && name != "get"
        && name != "post"
        && name != "put"
        && name != "patch"
        && name != "delete"
    {
        return None;
    }
    let args = node.child_by_field_name("arguments")?;
    let path = first_string_arg(source, args)?;
    let method = if name == "route" {
        "GET".to_string()
    } else {
        name.to_ascii_uppercase()
    };
    Some(HttpEndpoint {
        method,
        path: normalize_path(&path),
        line: node.start_position().row as u32 + 1,
        source: SourceKind::Ast,
    })
}

fn rust_attr_endpoint(source: &str, node: Node<'_>) -> Option<HttpEndpoint> {
    // #[get("/x")]
    let text = node_text(source, node);
    let lower = text.to_ascii_lowercase();
    for m in HTTP_METHODS {
        let needle = format!("#[{m}");
        if let Some(idx) = lower.find(&needle) {
            let after = &text[idx + needle.len()..];
            if let Some(path) = extract_quoted(after) {
                return Some(HttpEndpoint {
                    method: m.to_ascii_uppercase(),
                    path: normalize_path(&path),
                    line: node.start_position().row as u32 + 1,
                    source: SourceKind::Ast,
                });
            }
        }
    }
    None
}

fn field_text(source: &str, node: Node<'_>, field: &str) -> Option<String> {
    let child = node.child_by_field_name(field)?;
    Some(node_text(source, child))
}

fn node_text(source: &str, node: Node<'_>) -> String {
    source
        .get(node.byte_range())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn first_string_arg(source: &str, args: Node<'_>) -> Option<String> {
    let mut cursor = args.walk();
    for child in args.children(&mut cursor) {
        if let Some(s) = string_literal(source, child) {
            return Some(s);
        }
    }
    None
}

fn first_string_in_node(source: &str, node: Node<'_>) -> Option<String> {
    if let Some(s) = string_literal(source, node) {
        return Some(s);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(s) = first_string_in_node(source, child) {
            return Some(s);
        }
    }
    None
}

fn string_literal(source: &str, node: Node<'_>) -> Option<String> {
    match node.kind() {
        "string"
        | "string_literal"
        | "raw_string_literal"
        | "interpreted_string_literal"
        | "string_content"
        | "encapsed_string" => {
            let raw = node_text(source, node);
            Some(strip_quotes(&raw))
        }
        _ => None,
    }
}

fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 {
        let bytes = s.as_bytes();
        let q = bytes[0];
        if (q == b'\'' || q == b'"' || q == b'`') && bytes[bytes.len() - 1] == q {
            return s[1..s.len() - 1].to_string();
        }
    }
    s.to_string()
}

fn extract_quoted(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\'' || bytes[i] == b'"' {
            let q = bytes[i];
            i += 1;
            let start = i;
            while i < bytes.len() && bytes[i] != q {
                i += 1;
            }
            if i < bytes.len() {
                return Some(s[start..i].to_string());
            }
            return None;
        }
        i += 1;
    }
    None
}

fn normalize_path(path: &str) -> String {
    path.trim()
        .trim_matches(|c| c == '\'' || c == '"')
        .to_string()
}
