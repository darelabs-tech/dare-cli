//! Deterministic pattern mining - frequency & co-occurrence (microplano 038).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use dare_core::fs::atomic_write;
use dare_core::redact;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::report::MANIFEST_READ_CAP;
use crate::root::find_project_root;

pub const PATTERNS_SCHEMA_VERSION: u32 = 1;
pub const PATTERNS_MD_REL: &str = "DARE/PATTERNS.md";
pub const PATTERNS_FACTS_REL: &str = "DARE/patterns-facts.json";
pub const AST_FILE_CAP: usize = 32;
pub const AST_BYTES_CAP: usize = 524_288;
pub const WALK_MAX_ENTRIES: usize = 2_000;
pub const MIN_FREQUENCY: u64 = 1;

/// Closed set of pattern kinds (Mestre 5.6).
pub const PATTERN_KINDS: &[&str] = &[
    "inferred-layer",
    "naming-idiom",
    "structural-idiom",
    "call-idiom",
    "implicit-decision",
];

const SKIP_DIRS: &[&str] = &[
    "target",
    "node_modules",
    ".git",
    ".dare",
    "dist",
    "build",
    "vendor",
    ".next",
    "coverage",
    "__pycache__",
    "DARE",
];

const LAYER_DIRS: &[&str] = &[
    "handlers",
    "services",
    "repositories",
    "controllers",
    "models",
    "components",
    "middleware",
    "routes",
    "domain",
    "application",
    "infrastructure",
];

const STRUCTURAL_NAMES: &[&str] = &[
    "mod.rs",
    "lib.rs",
    "main.rs",
    "index.ts",
    "index.js",
    "__init__.py",
];

#[derive(Debug, Clone)]
pub struct PatternsOptions {
    pub dir: PathBuf,
    pub check: bool,
    pub inject: bool,
    pub ast: bool,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredPattern {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub frequency: u64,
    pub score: u64,
    pub evidence: Vec<String>,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cooccurrence {
    pub left: String,
    pub right: String,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternsReport {
    pub schema_version: u32,
    pub mode: String,
    pub project_root: String,
    pub patterns: Vec<DiscoveredPattern>,
    pub cooccurrences: Vec<Cooccurrence>,
    pub written: Vec<String>,
    pub modules_scanned: Vec<String>,
    pub ast_enabled: bool,
    pub inject: bool,
    pub graph_indexed: bool,
    pub warnings: Vec<String>,
}

fn display_path(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn to_posix_rel(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.to_string_lossy().replace('\\', "/")
}

fn skip_dir_name(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

fn read_capped(path: &Path, cap: usize) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    if data.len() > cap || data.contains(&0) {
        return None;
    }
    String::from_utf8(data).ok()
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if matches!(c, '-' | '_' | '/' | '.') && !out.ends_with('-') {
            out.push('-');
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "unknown".into()
    } else {
        out
    }
}

fn pattern_id(kind: &str, slug: &str) -> String {
    format!("{kind}:{}", slugify(slug))
}

/// Inventory modules: `crates/*`, `src`, and top-level source-ish dirs.
fn inventory_modules(root: &Path) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    let crates = root.join("crates");
    if crates.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&crates) {
            for ent in rd.flatten() {
                if !ent.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let name = ent.file_name().to_string_lossy().to_string();
                if skip_dir_name(&name) || name.starts_with('.') {
                    continue;
                }
                out.push((name, ent.path()));
            }
        }
    }
    let src = root.join("src");
    if src.is_dir() {
        out.push(("src".into(), src));
    }
    for name in ["app", "lib", "packages", "services"] {
        let p = root.join(name);
        if p.is_dir() {
            out.push((name.into(), p));
        }
    }
    // Fallback: project root as single module when nothing else found
    if out.is_empty() {
        out.push(("root".into(), root.to_path_buf()));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out.dedup_by(|a, b| a.0 == b.0);
    out
}

fn filter_modules(
    all: Vec<(String, PathBuf)>,
    filter: &[String],
) -> CoreResult<Vec<(String, PathBuf)>> {
    if filter.is_empty() {
        return Ok(all);
    }
    for id in filter {
        if id.trim().is_empty() {
            return Err(CoreError::invalid_input("empty module id in --modules"));
        }
    }
    let set: BTreeSet<String> = filter.iter().map(|s| s.trim().to_string()).collect();
    let filtered: Vec<_> = all.into_iter().filter(|(id, _)| set.contains(id)).collect();
    if filtered.is_empty() {
        return Err(CoreError::invalid_input(
            "no modules matched --modules filter",
        ));
    }
    Ok(filtered)
}

type PatternAcc = (String, String, u64, BTreeSet<String>, BTreeSet<String>);

struct Acc {
    /// id -> (kind, title, frequency, evidence, modules)
    patterns: BTreeMap<String, PatternAcc>,
    /// module_id -> pattern ids seen in that module
    per_module: BTreeMap<String, BTreeSet<String>>,
}

impl Acc {
    fn new() -> Self {
        Self {
            patterns: BTreeMap::new(),
            per_module: BTreeMap::new(),
        }
    }

    fn bump(
        &mut self,
        kind: &str,
        slug: &str,
        title: &str,
        module: &str,
        evidence: impl AsRef<str>,
        n: u64,
    ) {
        let id = pattern_id(kind, slug);
        let ev = redact(evidence.as_ref());
        let e = self.patterns.entry(id.clone()).or_insert_with(|| {
            (
                kind.to_string(),
                title.to_string(),
                0,
                BTreeSet::new(),
                BTreeSet::new(),
            )
        });
        e.2 = e.2.saturating_add(n);
        if e.3.len() < 24 && !ev.is_empty() {
            e.3.insert(ev);
        }
        e.4.insert(module.to_string());
        self.per_module
            .entry(module.to_string())
            .or_default()
            .insert(id);
    }
}

fn walk_module_files(mod_root: &Path, project_root: &Path, mut visit: impl FnMut(&str, &Path)) {
    let mut stack = vec![mod_root.to_path_buf()];
    let mut n = 0usize;
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut ents: Vec<_> = rd.flatten().collect();
        ents.sort_by_key(|e| e.file_name());
        for ent in ents {
            if n >= WALK_MAX_ENTRIES {
                return;
            }
            let path = ent.path();
            let name = ent.file_name().to_string_lossy().to_string();
            if ent.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if skip_dir_name(&name) {
                    continue;
                }
                stack.push(path);
                continue;
            }
            n += 1;
            let rel = to_posix_rel(project_root, &path);
            visit(&rel, &path);
        }
    }
}

fn classify_name(stem: &str) -> &'static str {
    if stem.contains('-') {
        "kebab-case"
    } else if stem.contains('_') {
        "snake_case"
    } else if stem.chars().next().is_some_and(|c| c.is_uppercase()) {
        "PascalCase"
    } else if stem.chars().any(|c| c.is_uppercase()) {
        "camelCase"
    } else {
        "other"
    }
}

fn mine_module(acc: &mut Acc, module_id: &str, mod_root: &Path, project_root: &Path) {
    // inferred-layer: known layer directory names under module
    for layer in LAYER_DIRS {
        let p = mod_root.join(layer);
        if p.is_dir() {
            acc.bump(
                "inferred-layer",
                layer,
                &format!("Layer directory `{layer}/`"),
                module_id,
                format!("{}/", to_posix_rel(project_root, &p)),
                1,
            );
        }
    }
    // also detect layer as path segment deeper
    walk_module_files(mod_root, project_root, |rel, path| {
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            return;
        };
        // structural
        if STRUCTURAL_NAMES.contains(&name) {
            acc.bump(
                "structural-idiom",
                name,
                &format!("Structural entry `{name}`"),
                module_id,
                rel,
                1,
            );
        }
        // naming style
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if !stem.starts_with('.') {
                let style = classify_name(stem);
                if style != "other" {
                    acc.bump(
                        "naming-idiom",
                        style,
                        &format!("File naming style `{style}`"),
                        module_id,
                        rel,
                        1,
                    );
                }
                // suffix idioms
                for (suf, label) in [
                    ("_service", "service-suffix"),
                    ("_controller", "controller-suffix"),
                    ("_handler", "handler-suffix"),
                    ("_repo", "repo-suffix"),
                    ("_repository", "repository-suffix"),
                    (".service", "service-dot-suffix"),
                    (".controller", "controller-dot-suffix"),
                ] {
                    let lower = stem.to_ascii_lowercase();
                    if lower.ends_with(suf.trim_start_matches('.'))
                        || name.to_ascii_lowercase().contains(suf)
                    {
                        acc.bump(
                            "naming-idiom",
                            label,
                            &format!("Naming idiom `{label}`"),
                            module_id,
                            rel,
                            1,
                        );
                    }
                }
            }
        }
        // path segment layers
        for seg in rel.split('/') {
            if LAYER_DIRS.contains(&seg) {
                acc.bump(
                    "inferred-layer",
                    seg,
                    &format!("Layer path segment `{seg}`"),
                    module_id,
                    rel,
                    1,
                );
            }
        }
    });
}

fn mine_implicit(acc: &mut Acc, project_root: &Path, modules: &[(String, PathBuf)]) {
    let root_mod = modules.first().map(|(id, _)| id.as_str()).unwrap_or("root");
    if project_root.join("Cargo.toml").is_file() && project_root.join("crates").is_dir() {
        acc.bump(
            "implicit-decision",
            "rust-workspace",
            "Implicit decision: Rust Cargo workspace layout",
            root_mod,
            "Cargo.toml",
            1,
        );
        acc.bump(
            "implicit-decision",
            "rust-workspace",
            "Implicit decision: Rust Cargo workspace layout",
            root_mod,
            "crates/",
            1,
        );
    }
    if project_root.join("package.json").is_file() {
        acc.bump(
            "implicit-decision",
            "node-package",
            "Implicit decision: Node package root",
            root_mod,
            "package.json",
            1,
        );
    }
    if project_root.join("pnpm-workspace.yaml").is_file()
        || project_root.join("pnpm-workspace.yml").is_file()
    {
        acc.bump(
            "implicit-decision",
            "pnpm-workspace",
            "Implicit decision: pnpm workspace",
            root_mod,
            "pnpm-workspace.yaml",
            1,
        );
    }
    if project_root.join("pyproject.toml").is_file() {
        acc.bump(
            "implicit-decision",
            "python-project",
            "Implicit decision: Python project root",
            root_mod,
            "pyproject.toml",
            1,
        );
    }
    if project_root.join("DARE").is_dir() {
        acc.bump(
            "implicit-decision",
            "dare-methodology",
            "Implicit decision: DARE methodology present",
            root_mod,
            "DARE/",
            1,
        );
    }
}

fn mine_ast(
    acc: &mut Acc,
    modules: &[(String, PathBuf)],
    project_root: &Path,
    warnings: &mut Vec<String>,
) {
    let mut scanned = 0usize;
    for (mod_id, mod_root) in modules {
        if scanned >= AST_FILE_CAP {
            break;
        }
        walk_module_files(mod_root, project_root, |rel, path| {
            if scanned >= AST_FILE_CAP {
                return;
            }
            if dare_ast::detect_language(rel).is_none() {
                return;
            }
            let Some(src) = read_capped(path, AST_BYTES_CAP) else {
                return;
            };
            scanned += 1;
            match dare_ast::analyze_source(rel, &src) {
                Ok(model) => {
                    let mut methods: BTreeMap<String, u64> = BTreeMap::new();
                    for ep in &model.endpoints {
                        *methods.entry(ep.method.to_ascii_uppercase()).or_default() += 1;
                    }
                    for (m, n) in methods {
                        if n >= MIN_FREQUENCY {
                            acc.bump(
                                "call-idiom",
                                &format!("http-{m}"),
                                &format!("HTTP call idiom `{m}`"),
                                mod_id,
                                rel,
                                n,
                            );
                        }
                    }
                    let mut kinds: BTreeMap<String, u64> = BTreeMap::new();
                    for ent in &model.entities {
                        *kinds.entry(ent.kind.to_ascii_lowercase()).or_default() += 1;
                    }
                    for (k, n) in kinds {
                        if n >= MIN_FREQUENCY {
                            acc.bump(
                                "call-idiom",
                                &format!("entity-{k}"),
                                &format!("Entity kind idiom `{k}`"),
                                mod_id,
                                rel,
                                n,
                            );
                        }
                    }
                }
                Err(e) => {
                    if warnings.len() < 16 {
                        warnings.push(format!("ast skip {rel}: {}", redact(e.message())));
                    }
                }
            }
        });
    }
}

fn finish_patterns(acc: Acc) -> (Vec<DiscoveredPattern>, Vec<Cooccurrence>) {
    let mut patterns: Vec<DiscoveredPattern> = acc
        .patterns
        .into_iter()
        .filter(|(_, (_, _, freq, _, _))| *freq >= MIN_FREQUENCY)
        .map(|(id, (kind, title, frequency, evidence, modules))| {
            let mut evidence: Vec<_> = evidence.into_iter().collect();
            evidence.sort();
            let mut modules: Vec<_> = modules.into_iter().collect();
            modules.sort();
            DiscoveredPattern {
                id,
                kind,
                title: redact(&title),
                frequency,
                score: frequency,
                evidence,
                modules,
            }
        })
        .collect();
    patterns.sort_by(|a, b| (&a.kind, &a.id).cmp(&(&b.kind, &b.id)));

    let mut pair_counts: BTreeMap<(String, String), u64> = BTreeMap::new();
    for ids in acc.per_module.values() {
        let list: Vec<_> = ids.iter().cloned().collect();
        for i in 0..list.len() {
            for j in (i + 1)..list.len() {
                let (a, b) = if list[i] <= list[j] {
                    (list[i].clone(), list[j].clone())
                } else {
                    (list[j].clone(), list[i].clone())
                };
                *pair_counts.entry((a, b)).or_default() += 1;
            }
        }
    }
    let mut cooccurrences: Vec<Cooccurrence> = pair_counts
        .into_iter()
        .map(|((left, right), count)| Cooccurrence { left, right, count })
        .collect();
    cooccurrences.sort_by(|a, b| (&a.left, &a.right).cmp(&(&b.left, &b.right)));

    (patterns, cooccurrences)
}

fn extract_agent_bodies(existing: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut rest = existing;
    while let Some(start) = rest.find("<!-- AGENT:BEGIN section=\"") {
        let after = &rest[start + "<!-- AGENT:BEGIN section=\"".len()..];
        let Some(end_q) = after.find('"') else {
            break;
        };
        let section = after[..end_q].to_string();
        let Some(body_start_rel) = after.find("-->") else {
            break;
        };
        let body_start = body_start_rel + 3;
        let body_slice = &after[body_start..];
        let end_marker = format!("<!-- AGENT:END section=\"{section}\" -->");
        let Some(body_end) = body_slice.find(&end_marker) else {
            break;
        };
        let body = body_slice[..body_end].to_string();
        map.insert(section, body);
        rest = &body_slice[body_end + end_marker.len()..];
    }
    map
}

fn render_patterns_md(
    patterns: &[DiscoveredPattern],
    cooccurrences: &[Cooccurrence],
    preserved: &BTreeMap<String, String>,
) -> String {
    let mut out = String::new();
    out.push_str("# Project Patterns\n\n");
    out.push_str(
        "_Deterministic mining (frequency + co-occurrence). Enrich AGENT sections as needed._\n\n",
    );

    out.push_str("## Discovered Patterns\n\n");
    if patterns.is_empty() {
        out.push_str("_No patterns detected._\n\n");
    } else {
        out.push_str("| ID | Kind | Title | Frequency | Score | Evidence |\n");
        out.push_str("|----|------|-------|-----------|-------|----------|\n");
        for p in patterns {
            let ev = p.evidence.join("; ");
            out.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} |\n",
                p.id, p.kind, p.title, p.frequency, p.score, ev
            ));
        }
        out.push('\n');
    }

    out.push_str("<!-- AGENT:BEGIN section=\"patterns-notes\" -->\n");
    if let Some(body) = preserved.get("patterns-notes") {
        out.push_str(body);
    } else {
        out.push_str("_Human notes on pattern confidence and exceptions._\n");
    }
    out.push_str("<!-- AGENT:END section=\"patterns-notes\" -->\n\n");

    out.push_str("## Co-occurrence\n\n");
    if cooccurrences.is_empty() {
        out.push_str("_No co-occurrences detected._\n\n");
    } else {
        out.push_str("| Left | Right | Count |\n");
        out.push_str("|------|-------|-------|\n");
        for c in cooccurrences.iter().take(100) {
            out.push_str(&format!("| `{}` | `{}` | {} |\n", c.left, c.right, c.count));
        }
        out.push('\n');
    }

    out.push_str("<!-- AGENT:BEGIN section=\"guidance\" -->\n");
    if let Some(body) = preserved.get("guidance") {
        out.push_str(body);
    } else {
        out.push_str("_ALWAYS / NEVER guidance derived from patterns._\n");
    }
    out.push_str("<!-- AGENT:END section=\"guidance\" -->\n");
    out
}

fn facts_document(
    patterns: &[DiscoveredPattern],
    cooccurrences: &[Cooccurrence],
) -> CoreResult<String> {
    let doc = json!({
        "schemaVersion": PATTERNS_SCHEMA_VERSION,
        "patterns": patterns,
        "cooccurrences": cooccurrences,
    });
    serde_json::to_string_pretty(&doc).map_err(|e| {
        CoreError::internal(format!(
            "patterns facts serialize: {}",
            redact(&e.to_string())
        ))
    })
}

fn try_index_graph(root: &ProjectRoot, patterns: &[DiscoveredPattern]) -> CoreResult<bool> {
    use dare_graph::{canonical_pattern_node_id, KnowledgeGraph};
    let cfg = dare_graph::load_graph_config(root, None)?;
    let abs = root.resolve(&SafeRelativePath::new(&cfg.path)?)?;
    if !abs.as_path().as_std_path().exists() {
        return Ok(false);
    }
    let mut g = dare_graph::open_graph(root, &cfg)?;
    g.migrate()?;
    for p in patterns.iter().take(100) {
        let id = canonical_pattern_node_id(&p.id);
        let mut node =
            dare_graph::GraphNode::new(id, dare_graph::NodeType::Pattern, p.title.clone());
        node.description = Some(format!("{} (freq={})", p.kind, p.frequency));
        let mut meta = serde_json::Map::new();
        meta.insert("kind".into(), json!(p.kind));
        meta.insert("patternId".into(), json!(p.id));
        meta.insert("frequency".into(), json!(p.frequency));
        meta.insert("score".into(), json!(p.score));
        meta.insert("evidence".into(), json!(p.evidence));
        meta.insert("source".into(), json!("dare-patterns"));
        node.metadata = meta;
        g.add_node(node)?;
    }
    g.flush()?;
    Ok(true)
}

/// Run pattern mining. `--check` performs zero filesystem mutations.
pub fn run_patterns(opts: &PatternsOptions) -> CoreResult<PatternsReport> {
    let start = &opts.dir;
    if !start.exists() || !start.is_dir() {
        return Err(CoreError::not_found(format!(
            "directory not found: {}",
            start.display()
        )));
    }

    let Some(project_root) = find_project_root(start) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&project_root)?;
    let mut warnings = Vec::new();

    let all = inventory_modules(&project_root);
    let modules = filter_modules(all, &opts.modules)?;
    let modules_scanned: Vec<String> = modules.iter().map(|(id, _)| id.clone()).collect();

    let mut acc = Acc::new();
    for (id, path) in &modules {
        mine_module(&mut acc, id, path, &project_root);
    }
    mine_implicit(&mut acc, &project_root, &modules);
    if opts.ast {
        mine_ast(&mut acc, &modules, &project_root, &mut warnings);
    }

    let (patterns, cooccurrences) = finish_patterns(acc);
    let mode = if opts.check { "check" } else { "write" };
    let mut written = Vec::new();
    let mut graph_indexed = false;

    if !opts.check {
        let mut preserved = BTreeMap::new();
        if opts.inject {
            let md_rel = SafeRelativePath::new(PATTERNS_MD_REL)?;
            if let Ok(abs) = root.resolve(&md_rel) {
                let p = abs.as_path().as_std_path();
                if p.is_file() {
                    if let Some(existing) = read_capped(p, MANIFEST_READ_CAP.saturating_mul(16)) {
                        preserved = extract_agent_bodies(&existing);
                    }
                }
            }
        }
        let md = render_patterns_md(&patterns, &cooccurrences, &preserved);
        let facts = facts_document(&patterns, &cooccurrences)?;
        let md_rel = SafeRelativePath::new(PATTERNS_MD_REL)?;
        let json_rel = SafeRelativePath::new(PATTERNS_FACTS_REL)?;
        atomic_write(&root, &md_rel, md.as_bytes())?;
        atomic_write(&root, &json_rel, facts.as_bytes())?;
        written.push(PATTERNS_MD_REL.to_string());
        written.push(PATTERNS_FACTS_REL.to_string());

        match try_index_graph(&root, &patterns) {
            Ok(true) => graph_indexed = true,
            Ok(false) => {}
            Err(e) => warnings.push(format!("graph index skipped: {}", redact(e.message()))),
        }
    }

    Ok(PatternsReport {
        schema_version: PATTERNS_SCHEMA_VERSION,
        mode: mode.to_string(),
        project_root: display_path(&project_root),
        patterns,
        cooccurrences,
        written,
        modules_scanned,
        ast_enabled: opts.ast,
        inject: opts.inject,
        graph_indexed,
        warnings,
    })
}

/// Human-readable summary (en-US).
pub fn format_human(r: &PatternsReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("schemaVersion: {}\n", r.schema_version));
    out.push_str(&format!("mode: {}\n", r.mode));
    out.push_str(&format!("projectRoot: {}\n", r.project_root));
    out.push_str(&format!("patterns: {}\n", r.patterns.len()));
    out.push_str(&format!("cooccurrences: {}\n", r.cooccurrences.len()));
    out.push_str(&format!(
        "modulesScanned: {}\n",
        r.modules_scanned.join(",")
    ));
    out.push_str(&format!("astEnabled: {}\n", r.ast_enabled));
    out.push_str(&format!("inject: {}\n", r.inject));
    out.push_str(&format!("graphIndexed: {}\n", r.graph_indexed));
    if r.written.is_empty() {
        out.push_str("written: (none)\n");
    } else {
        out.push_str("written:\n");
        for w in &r.written {
            out.push_str(&format!("  - {w}\n"));
        }
    }
    if !r.warnings.is_empty() {
        out.push_str("warnings:\n");
        for w in &r.warnings {
            out.push_str(&format!("  - {w}\n"));
        }
    }
    if r.mode == "check" {
        out.push_str("mode: check (zero mutations)\n");
    }
    out
}

pub fn report_to_json(r: &PatternsReport) -> Value {
    serde_json::to_value(r).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture_rust() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers=[\"crates/api\"]\n",
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("crates/api/src/handlers")).unwrap();
        fs::write(
            dir.path().join("crates/api/src/lib.rs"),
            "pub mod handlers;\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("crates/api/src/handlers/mod.rs"),
            "pub fn handle() {}\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("crates/api/src/user_service.rs"),
            "pub struct UserService;\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn kinds_include_closed_set() {
        assert!(PATTERN_KINDS.contains(&"inferred-layer"));
        assert!(PATTERN_KINDS.contains(&"naming-idiom"));
        assert_eq!(PATTERN_KINDS.len(), 5);
    }

    #[test]
    fn mine_writes_patterns() {
        let dir = fixture_rust();
        let report = run_patterns(&PatternsOptions {
            dir: dir.path().to_path_buf(),
            check: false,
            inject: false,
            ast: false,
            modules: vec![],
        })
        .unwrap();
        assert_eq!(report.mode, "write");
        assert!(!report.patterns.is_empty());
        assert!(dir.path().join("DARE/PATTERNS.md").is_file());
        assert!(dir.path().join("DARE/patterns-facts.json").is_file());
        assert!(report.patterns.iter().any(|p| p.kind == "inferred-layer"));
        assert!(report
            .patterns
            .iter()
            .any(|p| p.kind == "implicit-decision"));
        // stable sort
        let mut sorted = report.patterns.clone();
        sorted.sort_by(|a, b| (&a.kind, &a.id).cmp(&(&b.kind, &b.id)));
        assert_eq!(report.patterns, sorted);
    }

    #[test]
    fn check_zero_write() {
        let dir = fixture_rust();
        let report = run_patterns(&PatternsOptions {
            dir: dir.path().to_path_buf(),
            check: true,
            inject: false,
            ast: false,
            modules: vec![],
        })
        .unwrap();
        assert_eq!(report.mode, "check");
        assert!(report.written.is_empty());
        assert!(!dir.path().join("DARE").exists());
    }

    #[test]
    fn inject_preserves_agent_body() {
        let dir = fixture_rust();
        // first write
        run_patterns(&PatternsOptions {
            dir: dir.path().to_path_buf(),
            check: false,
            inject: false,
            ast: false,
            modules: vec![],
        })
        .unwrap();
        let path = dir.path().join("DARE/PATTERNS.md");
        let mut md = fs::read_to_string(&path).unwrap();
        md = md.replace(
            "_Human notes on pattern confidence and exceptions._\n",
            "KEEP_ME_CUSTOM_NOTE\n",
        );
        fs::write(&path, &md).unwrap();
        run_patterns(&PatternsOptions {
            dir: dir.path().to_path_buf(),
            check: false,
            inject: true,
            ast: false,
            modules: vec![],
        })
        .unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("KEEP_ME_CUSTOM_NOTE"));
    }

    #[test]
    fn modules_filter_invalid() {
        let dir = fixture_rust();
        let err = run_patterns(&PatternsOptions {
            dir: dir.path().to_path_buf(),
            check: true,
            inject: false,
            ast: false,
            modules: vec!["no-such".into()],
        })
        .unwrap_err();
        assert!(err.message().contains("no modules matched"));
    }

    #[test]
    fn missing_dir_not_found() {
        let err = run_patterns(&PatternsOptions {
            dir: PathBuf::from("__dare_missing_patterns_dir__"),
            check: true,
            inject: false,
            ast: false,
            modules: vec![],
        })
        .unwrap_err();
        assert_eq!(err.exit_code(), 3);
    }
}
