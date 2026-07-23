//! Merge AST and regex extractions with deterministic dedupe.

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::model::{Entity, HttpEndpoint};

#[cfg(test)]
use crate::model::SourceKind;

/// Merge extractions preferring [`SourceKind::Ast`] over regex duplicates.
///
/// Endpoint key: `METHOD\0path`. Entity key: `kind\0name`.
/// Output is sorted for stable serialization.
pub fn merge_extractions(
    ast_endpoints: Vec<HttpEndpoint>,
    ast_entities: Vec<Entity>,
    regex_endpoints: Vec<HttpEndpoint>,
    regex_entities: Vec<Entity>,
) -> (Vec<HttpEndpoint>, Vec<Entity>) {
    let endpoints = merge_list(
        ast_endpoints,
        regex_endpoints,
        |e| format!("{}\0{}", e.method, e.path),
        |a, b| {
            a.method
                .cmp(&b.method)
                .then(a.path.cmp(&b.path))
                .then(a.line.cmp(&b.line))
        },
    );
    let entities = merge_list(
        ast_entities,
        regex_entities,
        |e| format!("{}\0{}", e.kind, e.name),
        |a, b| {
            a.kind
                .cmp(&b.kind)
                .then(a.name.cmp(&b.name))
                .then(a.line.cmp(&b.line))
        },
    );
    (endpoints, entities)
}

fn merge_list<T, K, FKey, FCmp>(ast: Vec<T>, regex: Vec<T>, key_of: FKey, cmp: FCmp) -> Vec<T>
where
    K: Ord,
    FKey: Fn(&T) -> K,
    FCmp: Fn(&T, &T) -> Ordering,
{
    let mut map: BTreeMap<K, T> = BTreeMap::new();
    for item in regex {
        map.insert(key_of(&item), item);
    }
    for item in ast {
        // AST overwrites regex for the same key.
        map.insert(key_of(&item), item);
    }
    let mut out: Vec<T> = map.into_values().collect();
    out.sort_by(cmp);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_ast_over_regex() {
        let ast = vec![HttpEndpoint {
            method: "GET".into(),
            path: "/a".into(),
            line: 1,
            source: SourceKind::Ast,
        }];
        let reg = vec![HttpEndpoint {
            method: "GET".into(),
            path: "/a".into(),
            line: 9,
            source: SourceKind::Regex,
        }];
        let (eps, _) = merge_extractions(ast, vec![], reg, vec![]);
        assert_eq!(eps.len(), 1);
        assert_eq!(eps[0].source, SourceKind::Ast);
        assert_eq!(eps[0].line, 1);
    }

    #[test]
    fn sorts_deterministically() {
        let a = vec![
            HttpEndpoint {
                method: "POST".into(),
                path: "/b".into(),
                line: 2,
                source: SourceKind::Regex,
            },
            HttpEndpoint {
                method: "GET".into(),
                path: "/a".into(),
                line: 1,
                source: SourceKind::Regex,
            },
        ];
        let (eps, _) = merge_extractions(vec![], vec![], a, vec![]);
        assert_eq!(eps[0].method, "GET");
        assert_eq!(eps[1].method, "POST");
    }
}
