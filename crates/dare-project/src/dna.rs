//! Project DNA extraction — conventions with evidence (microplano 037).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

use dare_core::fs::atomic_write;
use dare_core::redact;
use dare_core::{
    CoreError, CoreResult, ProcessRunner, ProjectRoot, SafeCommand, SafeRelativePath,
    SystemProcessRunner,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::git::find_git_root;
use crate::report::MANIFEST_READ_CAP;
use crate::root::find_project_root;

pub const DNA_SCHEMA_VERSION: u32 = 1;
pub const PROJECT_DNA_REL: &str = "DARE/PROJECT-DNA.md";
pub const DNA_FACTS_REL: &str = "DARE/dna-facts.json";
pub const GIT_LOG_LIMIT: usize = 20;
pub const AST_FILE_CAP: usize = 32;
pub const AST_BYTES_CAP: usize = 524_288;
pub const TOP_LIBS: usize = 25;
pub const NAME_SAMPLE_CAP: usize = 200;
pub const WALK_MAX_ENTRIES: usize = 2_000;

#[derive(Debug, Clone)]
pub struct DnaOptions {
    pub dir: PathBuf,
    pub check: bool,
    pub ast: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnaFact {
    pub category: String,
    pub key: String,
    pub value: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnaReport {
    pub schema_version: u32,
    pub mode: String,
    pub project_root: String,
    pub git_root: Option<String>,
    pub facts: Vec<DnaFact>,
    pub written: Vec<String>,
    pub ast_enabled: bool,
    pub graph_indexed: bool,
    pub warnings: Vec<String>,
}

fn display_path(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn push_fact(
    facts: &mut Vec<DnaFact>,
    category: &str,
    key: &str,
    value: impl Into<String>,
    evidence: Vec<String>,
) {
    let mut evidence: Vec<String> = evidence
        .into_iter()
        .map(|e| redact(&e))
        .filter(|e| !e.is_empty())
        .collect();
    evidence.sort();
    evidence.dedup();
    facts.push(DnaFact {
        category: category.to_string(),
        key: key.to_string(),
        value: redact(&value.into()),
        evidence,
    });
}

fn sort_facts(facts: &mut [DnaFact]) {
    facts.sort_by(|a, b| (&a.category, &a.key).cmp(&(&b.category, &b.key)));
}

fn read_capped(path: &Path, cap: usize) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    if data.len() > cap {
        return None;
    }
    if data.contains(&0) {
        return None;
    }
    String::from_utf8(data).ok()
}

fn skip_dir_name(name: &str) -> bool {
    matches!(
        name,
        "target"
            | "node_modules"
            | ".git"
            | ".dare"
            | "dist"
            | "build"
            | "vendor"
            | ".next"
            | "coverage"
            | "__pycache__"
    )
}

/// Run DNA extraction. `--check` performs zero filesystem mutations.
pub fn run_dna(opts: &DnaOptions) -> CoreResult<DnaReport> {
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
    let git_root = find_git_root(start, Some(&project_root));
    let mut warnings = Vec::new();
    let mut facts = Vec::new();

    collect_tooling(&project_root, &mut facts);
    collect_naming(&project_root, &mut facts);
    collect_architecture(&project_root, &mut facts);
    collect_tests(&project_root, &mut facts);
    collect_libraries(&project_root, &mut facts);
    collect_commits(
        &project_root,
        git_root.as_deref(),
        &mut facts,
        &mut warnings,
    );

    if opts.ast {
        collect_ast_sample(&project_root, &mut facts, &mut warnings);
    }

    sort_facts(&mut facts);

    let mode = if opts.check { "check" } else { "write" };
    let mut written = Vec::new();
    let mut graph_indexed = false;

    if !opts.check {
        let md = render_project_dna(&facts);
        let facts_json = facts_document(&facts)?;
        let md_rel = SafeRelativePath::new(PROJECT_DNA_REL)?;
        let json_rel = SafeRelativePath::new(DNA_FACTS_REL)?;
        atomic_write(&root, &md_rel, md.as_bytes())?;
        atomic_write(&root, &json_rel, facts_json.as_bytes())?;
        written.push(PROJECT_DNA_REL.to_string());
        written.push(DNA_FACTS_REL.to_string());

        match try_index_graph(&root, &facts) {
            Ok(true) => graph_indexed = true,
            Ok(false) => {}
            Err(e) => warnings.push(format!("graph index skipped: {}", redact(e.message()))),
        }
    }

    Ok(DnaReport {
        schema_version: DNA_SCHEMA_VERSION,
        mode: mode.to_string(),
        project_root: display_path(&project_root),
        git_root: git_root.as_ref().map(|p| display_path(p)),
        facts,
        written,
        ast_enabled: opts.ast,
        graph_indexed,
        warnings,
    })
}

fn collect_tooling(root: &Path, facts: &mut Vec<DnaFact>) {
    let mut families = Vec::new();
    let mut family_ev = Vec::new();

    if root.join("package.json").is_file() {
        let mut ev = vec!["package.json".to_string()];
        if let Some(text) = read_capped(&root.join("package.json"), MANIFEST_READ_CAP) {
            if let Ok(v) = serde_json::from_str::<Value>(&text) {
                if let Some(pm) = v.get("packageManager").and_then(|x| x.as_str()) {
                    push_fact(facts, "tooling", "packageManager", pm, ev.clone());
                }
                if let Some(engines) = v.get("engines") {
                    push_fact(
                        facts,
                        "tooling",
                        "nodeEngines",
                        engines.to_string(),
                        ev.clone(),
                    );
                }
            }
        }
        for lock in [
            "pnpm-lock.yaml",
            "yarn.lock",
            "package-lock.json",
            "bun.lockb",
        ] {
            if root.join(lock).is_file() {
                ev.push(lock.to_string());
                push_fact(
                    facts,
                    "tooling",
                    "nodeLockfile",
                    lock,
                    vec![lock.to_string()],
                );
            }
        }
        families.push("node".to_string());
        family_ev.extend(ev);
    }

    if root.join("Cargo.toml").is_file() {
        let mut ev = vec!["Cargo.toml".to_string()];
        if let Some(text) = read_capped(&root.join("Cargo.toml"), MANIFEST_READ_CAP) {
            if let Some(ed) = extract_toml_string(&text, "edition") {
                push_fact(facts, "tooling", "rustEdition", ed, ev.clone());
            }
            if text.contains("[workspace]") {
                push_fact(facts, "tooling", "rustWorkspace", "true", ev.clone());
            }
        }
        if root.join("rust-toolchain.toml").is_file() {
            ev.push("rust-toolchain.toml".to_string());
            push_fact(
                facts,
                "tooling",
                "rustToolchainFile",
                "rust-toolchain.toml",
                vec!["rust-toolchain.toml".to_string()],
            );
        }
        if root.join("Cargo.lock").is_file() {
            push_fact(
                facts,
                "tooling",
                "rustLockfile",
                "Cargo.lock",
                vec!["Cargo.lock".to_string()],
            );
        }
        families.push("rust".to_string());
        family_ev.extend(ev);
    }

    let py_ev: Vec<String> = ["pyproject.toml", "requirements.txt", "setup.py"]
        .iter()
        .filter(|n| root.join(n).is_file())
        .map(|s| (*s).to_string())
        .collect();
    if !py_ev.is_empty() {
        families.push("python".to_string());
        family_ev.extend(py_ev);
    }

    if !families.is_empty() {
        families.sort();
        families.dedup();
        family_ev.sort();
        family_ev.dedup();
        push_fact(
            facts,
            "tooling",
            "stackFamily",
            families.join(","),
            family_ev,
        );
    }
}

fn extract_toml_string(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            continue;
        }
        if let Some(rest) = t.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                let v = rest.trim().trim_matches('"').trim_matches('\'').to_string();
                if !v.is_empty() {
                    return Some(v);
                }
            }
        }
    }
    None
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

fn collect_naming(root: &Path, facts: &mut Vec<DnaFact>) {
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut evidence = BTreeSet::new();
    let mut n = 0usize;
    walk_files(root, &mut |rel, path| {
        if n >= NAME_SAMPLE_CAP {
            return false;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            return true;
        };
        if stem.starts_with('.') {
            return true;
        }
        let style = classify_name(stem);
        *counts.entry(style).or_default() += 1;
        if evidence.len() < 12 {
            evidence.insert(rel);
        }
        n += 1;
        true
    });
    if let Some((style, _)) = counts.into_iter().max_by_key(|(_, c)| *c) {
        push_fact(
            facts,
            "naming",
            "fileNamingStyle",
            style,
            evidence.into_iter().collect(),
        );
    }
}

fn collect_architecture(root: &Path, facts: &mut Vec<DnaFact>) {
    let layers = [
        ("src", "src"),
        ("crates", "crates"),
        ("app", "app"),
        ("lib", "lib"),
        ("services", "services"),
        ("controllers", "controllers"),
        ("models", "models"),
        ("handlers", "handlers"),
        ("repositories", "repositories"),
        ("components", "components"),
    ];
    let mut found = Vec::new();
    let mut evidence = Vec::new();
    for (name, rel) in layers {
        if root.join(rel).is_dir() {
            found.push(name.to_string());
            evidence.push(format!("{rel}/"));
        }
    }
    if !found.is_empty() {
        found.sort();
        push_fact(
            facts,
            "architecture",
            "layersDetected",
            found.join(","),
            evidence,
        );
    }
}

fn collect_tests(root: &Path, facts: &mut Vec<DnaFact>) {
    let layouts = [
        "tests",
        "test",
        "__tests__",
        "spec",
        "crates/dare-cli/tests",
    ];
    let mut found = Vec::new();
    for rel in layouts {
        if root.join(rel).is_dir() {
            found.push(rel.to_string());
        }
    }
    // nested */tests dirs under crates (cap)
    let crates = root.join("crates");
    if crates.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&crates) {
            for ent in rd.flatten().take(40) {
                let t = ent.path().join("tests");
                if t.is_dir() {
                    let rel = format!("crates/{}/tests", ent.file_name().to_string_lossy());
                    found.push(rel);
                }
            }
        }
    }
    found.sort();
    found.dedup();
    if !found.is_empty() {
        let ev = found.iter().map(|s| format!("{s}/")).collect();
        push_fact(facts, "tests", "testLayout", found.join(","), ev);
    }

    if let Some(text) = read_capped(&root.join("package.json"), MANIFEST_READ_CAP) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            for section in ["devDependencies", "dependencies"] {
                if let Some(obj) = v.get(section).and_then(|x| x.as_object()) {
                    for name in [
                        "jest",
                        "vitest",
                        "mocha",
                        "ava",
                        "@playwright/test",
                        "pytest",
                    ] {
                        if obj.contains_key(name) {
                            push_fact(
                                facts,
                                "tests",
                                "testFramework",
                                name,
                                vec!["package.json".to_string()],
                            );
                        }
                    }
                }
            }
        }
    }
    if root.join("Cargo.toml").is_file() {
        push_fact(
            facts,
            "tests",
            "testFramework",
            "cargo-test",
            vec!["Cargo.toml".to_string()],
        );
    }
}

fn collect_libraries(root: &Path, facts: &mut Vec<DnaFact>) {
    if let Some(text) = read_capped(&root.join("package.json"), MANIFEST_READ_CAP) {
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            let mut names = Vec::new();
            for section in ["dependencies", "devDependencies"] {
                if let Some(obj) = v.get(section).and_then(|x| x.as_object()) {
                    for k in obj.keys() {
                        names.push(k.clone());
                    }
                }
            }
            names.sort();
            names.dedup();
            for name in names.into_iter().take(TOP_LIBS) {
                push_fact(
                    facts,
                    "libraries",
                    &format!("dep:{name}"),
                    "present",
                    vec!["package.json".to_string()],
                );
            }
        }
    }

    if let Some(text) = read_capped(&root.join("Cargo.toml"), MANIFEST_READ_CAP) {
        let mut in_deps = false;
        let mut names = Vec::new();
        for line in text.lines() {
            let t = line.trim();
            if t.starts_with('[') {
                in_deps = t == "[dependencies]"
                    || t == "[dev-dependencies]"
                    || t == "[workspace.dependencies]";
                continue;
            }
            if in_deps && !t.is_empty() && !t.starts_with('#') {
                if let Some(name) = t.split('=').next() {
                    let name = name.trim();
                    if !name.is_empty() && !name.contains('.') {
                        names.push(name.to_string());
                    }
                }
            }
        }
        names.sort();
        names.dedup();
        for name in names.into_iter().take(TOP_LIBS) {
            push_fact(
                facts,
                "libraries",
                &format!("dep:{name}"),
                "present",
                vec!["Cargo.toml".to_string()],
            );
        }
    }
}

fn collect_commits(
    project_root: &Path,
    git_root: Option<&Path>,
    facts: &mut Vec<DnaFact>,
    warnings: &mut Vec<String>,
) {
    let Some(_git) = git_root else {
        warnings.push("git not available; commit facts omitted".to_string());
        return;
    };
    let Ok(root) = ProjectRoot::new(project_root) else {
        warnings.push("git log skipped: invalid project root".to_string());
        return;
    };
    let Ok(rel) = SafeRelativePath::new(".") else {
        return;
    };
    let n = GIT_LOG_LIMIT.to_string();
    let cmd = SafeCommand::new("git")
        .args(["log", "-n", n.as_str(), "--pretty=format:%h%x09%s"])
        .cwd(root, rel)
        .timeout(Duration::from_secs(5))
        .stdout_limit(64 * 1024);
    let out = match SystemProcessRunner.run(&cmd) {
        Ok(o) => o,
        Err(e) => {
            warnings.push(format!("git log skipped: {}", redact(e.message())));
            return;
        }
    };
    if out.exit_code != 0 {
        warnings.push("git log failed; commit facts omitted".to_string());
        return;
    }
    for line in out.stdout.lines().take(GIT_LOG_LIMIT) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (hash, subject) = match line.split_once('\t') {
            Some((h, s)) => (h.trim(), s.trim()),
            None => continue,
        };
        if hash.is_empty() {
            continue;
        }
        push_fact(
            facts,
            "commits",
            &format!("recentCommit:{hash}"),
            subject,
            vec![format!("git:{hash}")],
        );
    }
}

fn collect_ast_sample(root: &Path, facts: &mut Vec<DnaFact>, warnings: &mut Vec<String>) {
    let mut files = Vec::new();
    walk_files(root, &mut |rel, path| {
        if files.len() >= AST_FILE_CAP {
            return false;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            return true;
        };
        if matches!(
            ext,
            "ts" | "tsx" | "js" | "jsx" | "py" | "php" | "go" | "rb" | "rs"
        ) {
            files.push((rel, path));
        }
        true
    });

    let mut entities = 0u64;
    let mut endpoints = 0u64;
    let mut scanned = 0u64;
    let mut ev = Vec::new();
    for (rel, path) in files {
        let Some(text) = read_capped(&path, AST_BYTES_CAP) else {
            continue;
        };
        match dare_ast::analyze_source(&rel, &text) {
            Ok(model) => {
                entities += model.entities.len() as u64;
                endpoints += model.endpoints.len() as u64;
                scanned += 1;
                if ev.len() < 8 {
                    ev.push(rel);
                }
            }
            Err(e) => warnings.push(format!("ast skip {rel}: {}", redact(e.message()))),
        }
    }
    if scanned > 0 {
        push_fact(
            facts,
            "architecture",
            "astFilesScanned",
            scanned.to_string(),
            ev.clone(),
        );
        push_fact(
            facts,
            "architecture",
            "astEntityCount",
            entities.to_string(),
            ev.clone(),
        );
        push_fact(
            facts,
            "architecture",
            "astEndpointCount",
            endpoints.to_string(),
            ev,
        );
    } else {
        warnings.push("ast enabled but no files analyzed".to_string());
    }
}

fn walk_files(root: &Path, visit: &mut dyn FnMut(String, PathBuf) -> bool) {
    let mut stack = vec![root.to_path_buf()];
    let mut visited = 0usize;
    while let Some(dir) = stack.pop() {
        if visited >= WALK_MAX_ENTRIES {
            break;
        }
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for ent in entries {
            visited += 1;
            if visited >= WALK_MAX_ENTRIES {
                break;
            }
            let path = ent.path();
            let name = ent.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if skip_dir_name(&name) {
                    continue;
                }
                stack.push(path);
            } else if path.is_file() {
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                if !visit(rel, path) {
                    return;
                }
            }
        }
    }
}

fn facts_document(facts: &[DnaFact]) -> CoreResult<String> {
    let doc = json!({
        "schemaVersion": DNA_SCHEMA_VERSION,
        "facts": facts,
    });
    serde_json::to_string_pretty(&doc).map_err(|e| CoreError::internal(e.to_string()))
}

/// Render deterministic PROJECT-DNA.md with AGENT markers for semantic sections.
pub fn render_project_dna(facts: &[DnaFact]) -> String {
    let mut out = String::new();
    out.push_str("# PROJECT DNA\n\n");
    out.push_str("<!-- dare:managed -->\n");
    out.push_str(&format!(
        "> Generated by `dare dna` (schemaVersion {DNA_SCHEMA_VERSION}). Deterministic facts below; AGENT sections are for semantic enrichment.\n\n"
    ));

    out.push_str("## Tooling\n\n");
    append_fact_table(&mut out, facts, "tooling");

    out.push_str("## Naming Conventions\n\n");
    append_fact_table(&mut out, facts, "naming");
    out.push_str("<!-- AGENT:BEGIN section=\"naming\" -->\n");
    out.push_str("_Confirm naming style and document exceptions._\n");
    out.push_str("<!-- AGENT:END section=\"naming\" -->\n\n");

    out.push_str("## Architecture & Layers\n\n");
    append_fact_table(&mut out, facts, "architecture");
    out.push_str("<!-- AGENT:BEGIN section=\"architecture\" -->\n");
    out.push_str("_Name the architecture pattern and layer rules._\n");
    out.push_str("<!-- AGENT:END section=\"architecture\" -->\n\n");

    out.push_str("## Tests\n\n");
    append_fact_table(&mut out, facts, "tests");
    out.push_str("<!-- AGENT:BEGIN section=\"tests\" -->\n");
    out.push_str("_Describe test layout, naming, and assertion style._\n");
    out.push_str("<!-- AGENT:END section=\"tests\" -->\n\n");

    out.push_str("## Libraries\n\n");
    append_fact_table(&mut out, facts, "libraries");

    out.push_str("## Recent Commits\n\n");
    append_fact_table(&mut out, facts, "commits");

    out.push_str("## Golden Rules\n\n");
    out.push_str("<!-- AGENT:BEGIN section=\"golden-rules\" -->\n");
    out.push_str("_ALWAYS / NEVER rules for this codebase._\n");
    out.push_str("<!-- AGENT:END section=\"golden-rules\" -->\n\n");

    out.push_str("## Uncertainties\n\n");
    out.push_str("<!-- AGENT:BEGIN section=\"uncertainties\" -->\n");
    out.push_str("_Ambiguous or mixed conventions needing human decision._\n");
    out.push_str("<!-- AGENT:END section=\"uncertainties\" -->\n");
    out
}

fn append_fact_table(out: &mut String, facts: &[DnaFact], category: &str) {
    let rows: Vec<&DnaFact> = facts.iter().filter(|f| f.category == category).collect();
    if rows.is_empty() {
        out.push_str("_No facts detected._\n\n");
        return;
    }
    out.push_str("| Key | Value | Evidence |\n");
    out.push_str("|-----|-------|----------|\n");
    for f in rows {
        let ev = f.evidence.join("; ");
        out.push_str(&format!("| `{}` | {} | {} |\n", f.key, f.value, ev));
    }
    out.push('\n');
}

fn try_index_graph(root: &ProjectRoot, facts: &[DnaFact]) -> CoreResult<bool> {
    use dare_graph::KnowledgeGraph;
    let cfg = dare_graph::load_graph_config(root, None)?;
    // Soft: only index if store already exists (avoid creating unexpected DB on every dna).
    let abs = root.resolve(&SafeRelativePath::new(&cfg.path)?)?;
    if !abs.as_path().as_std_path().exists() {
        return Ok(false);
    }
    let mut g = dare_graph::open_graph(root, &cfg)?;
    g.migrate()?;
    for fact in facts.iter().take(100) {
        let id = format!("concept:dna:{}:{}", fact.category, fact.key);
        let mut node = dare_graph::GraphNode::new(
            id,
            dare_graph::NodeType::Concept,
            format!("dna:{}:{}", fact.category, fact.key),
        );
        node.description = Some(fact.value.clone());
        let mut meta = serde_json::Map::new();
        meta.insert("category".into(), json!(fact.category));
        meta.insert("key".into(), json!(fact.key));
        meta.insert("evidence".into(), json!(fact.evidence));
        meta.insert("source".into(), json!("dare-dna"));
        node.metadata = meta;
        g.add_node(node)?;
    }
    g.flush()?;
    Ok(true)
}

/// Human-readable summary (en-US).
pub fn format_human(r: &DnaReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("schemaVersion: {}\n", r.schema_version));
    out.push_str(&format!("mode: {}\n", r.mode));
    out.push_str(&format!("projectRoot: {}\n", r.project_root));
    out.push_str(&format!(
        "gitRoot: {}\n",
        r.git_root.as_deref().unwrap_or("(none)")
    ));
    out.push_str(&format!("facts: {}\n", r.facts.len()));
    out.push_str(&format!("astEnabled: {}\n", r.ast_enabled));
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

pub fn report_to_json(r: &DnaReport) -> Value {
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
            "[package]\nname=\"demo\"\nedition=\"2021\"\n\n[dependencies]\nserde = \"1\"\n",
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::create_dir_all(dir.path().join("tests")).unwrap();
        fs::write(dir.path().join("tests/smoke.rs"), "#[test] fn t() {}\n").unwrap();
        dir
    }

    #[test]
    fn dna_check_no_write() {
        let dir = fixture_rust();
        let before: BTreeSet<_> = walk_snapshot(dir.path());
        let report = run_dna(&DnaOptions {
            dir: dir.path().to_path_buf(),
            check: true,
            ast: false,
        })
        .unwrap();
        assert_eq!(report.mode, "check");
        assert!(report.written.is_empty());
        assert!(!report.facts.is_empty());
        for f in &report.facts {
            assert!(
                !f.evidence.is_empty(),
                "fact {}.{} missing evidence",
                f.category,
                f.key
            );
        }
        let after: BTreeSet<_> = walk_snapshot(dir.path());
        assert_eq!(before, after, "check must not mutate filesystem");
    }

    #[test]
    fn dna_write_creates_artifacts() {
        let dir = fixture_rust();
        let report = run_dna(&DnaOptions {
            dir: dir.path().to_path_buf(),
            check: false,
            ast: false,
        })
        .unwrap();
        assert_eq!(report.mode, "write");
        assert!(dir.path().join(PROJECT_DNA_REL).is_file());
        assert!(dir.path().join(DNA_FACTS_REL).is_file());
        let md = fs::read_to_string(dir.path().join(PROJECT_DNA_REL)).unwrap();
        assert!(md.contains("PROJECT DNA"));
        assert!(md.contains("AGENT:BEGIN"));
    }

    #[test]
    fn dna_no_git_ok() {
        let dir = fixture_rust();
        let report = run_dna(&DnaOptions {
            dir: dir.path().to_path_buf(),
            check: true,
            ast: false,
        })
        .unwrap();
        assert!(report.git_root.is_none());
        assert!(report
            .warnings
            .iter()
            .any(|w| w.contains("git not available")));
        assert!(!report.facts.iter().any(|f| f.category == "commits"));
    }

    #[test]
    fn dna_tooling_rust_edition() {
        let dir = fixture_rust();
        let report = run_dna(&DnaOptions {
            dir: dir.path().to_path_buf(),
            check: true,
            ast: false,
        })
        .unwrap();
        let ed = report
            .facts
            .iter()
            .find(|f| f.category == "tooling" && f.key == "rustEdition")
            .expect("edition");
        assert_eq!(ed.value, "2021");
    }

    fn walk_snapshot(root: &Path) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        walk_files(root, &mut |rel, _| {
            out.insert(rel);
            true
        });
        out
    }
}
