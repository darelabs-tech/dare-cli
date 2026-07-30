//! Brownfield reverse engineering (microplano 036) — deterministic Fase 0 artifacts.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use dare_ast::{analyze_source, detect_language, Entity, HttpEndpoint, SourceKind};
use dare_core::fs::{atomic_write, read_to_string};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::detect;
use crate::root::find_project_root;

/// Frozen JSON schema version for reverse reports/facts.
pub const REVERSE_SCHEMA_VERSION: u32 = 1;

pub const IDEIA_REL: &str = "DARE/IDEIA.md";
pub const REVERSE_DIR: &str = "DARE/REVERSE";
pub const FACTS_REL: &str = "DARE/REVERSE/reverse-facts.json";
pub const EXCALIDRAW_REL: &str = "DARE/REVERSE/modules.excalidraw";
pub const CONFIDENCE_REL: &str = "DARE/REVERSE/confidence-report.md";

pub const MAX_MODULES: usize = 64;
pub const MAX_AST_FILES: usize = 200;
pub const MAX_FILE_BYTES: usize = 1_048_576;
pub const MAX_WALK_ENTRIES: usize = 4_096;

const MSG_CHECK: &str = "mode: check (zero mutations)";

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "vendor",
    ".dare",
    "DARE",
    "dist",
    "build",
    ".next",
    "coverage",
];

const SOURCE_EXTS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "py", "go", "php", "rb", "java", "kt", "cs", "vue", "svelte",
];

/// Options for `dare reverse`.
#[derive(Debug, Clone, Default)]
pub struct ReverseOptions {
    pub check: bool,
    pub deep: bool,
    pub modules: Vec<String>,
    pub ast: bool,
    pub excalidraw: bool,
    pub report: bool,
}

/// One discovered module (deterministic id + path).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleFact {
    pub id: String,
    pub path: String,
    pub languages: Vec<String>,
    pub loc: u64,
    pub file_count: u64,
    pub depends_on: Vec<String>,
}

/// Optional AST aggregate when `--ast` is set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AstSummary {
    pub files_scanned: u64,
    pub endpoints: Vec<AstEndpointFact>,
    pub entities: Vec<AstEntityFact>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AstEndpointFact {
    pub method: String,
    pub path: String,
    pub line: u32,
    pub file: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AstEntityFact {
    pub name: String,
    pub kind: String,
    pub line: u32,
    pub file: String,
    pub source: String,
}

/// Deterministic facts blob written to `reverse-facts.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseFacts {
    pub schema_version: u32,
    pub project_root: String,
    pub stacks: Vec<String>,
    pub modules: Vec<ModuleFact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ast: Option<AstSummary>,
    pub deep: bool,
}

/// CLI/JSON report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseReport {
    pub schema_version: u32,
    pub mode: String,
    pub ok: bool,
    pub project_root: String,
    pub module_count: u64,
    pub written: Vec<String>,
    pub warnings: Vec<String>,
    pub enriched: bool,
    pub check: bool,
    pub deep: bool,
    pub ast: bool,
    pub excalidraw: bool,
    pub report: bool,
}

fn display_path(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn to_posix_rel(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.to_string_lossy().replace('\\', "/")
}

fn is_skip_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

fn is_source_file(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| SOURCE_EXTS.contains(&ext))
}

fn sanitize_module_id(raw: &str) -> String {
    let mut out = String::new();
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
            out.push(c);
        } else if c == '/' || c == '\\' {
            out.push('-');
        }
    }
    if out.is_empty() {
        "module".into()
    } else {
        out
    }
}

fn count_loc_and_langs(dir: &Path) -> (u64, u64, Vec<String>) {
    let mut loc = 0u64;
    let mut files = 0u64;
    let mut langs: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec![dir.to_path_buf()];
    let mut visited = 0usize;

    while let Some(cur) = stack.pop() {
        if visited >= MAX_WALK_ENTRIES {
            break;
        }
        visited += 1;
        let Ok(entries) = fs::read_dir(&cur) else {
            continue;
        };
        for ent in entries.flatten() {
            let path = ent.path();
            let name = ent.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if !is_skip_dir(&name) {
                    stack.push(path);
                }
                continue;
            }
            if !path.is_file() || !is_source_file(&name) {
                continue;
            }
            files += 1;
            if let Some(lang) = detect_language(&display_path(&path)) {
                langs.insert(lang.as_str().to_string());
            }
            if let Ok(bytes) = fs::read(&path) {
                let slice = if bytes.len() > MAX_FILE_BYTES {
                    &bytes[..MAX_FILE_BYTES]
                } else {
                    &bytes
                };
                let text = String::from_utf8_lossy(slice);
                loc += text.lines().filter(|l| !l.trim().is_empty()).count() as u64;
            }
        }
    }

    (loc, files, langs.into_iter().collect())
}

fn rust_path_deps(crate_dir: &Path) -> Vec<String> {
    let cargo = crate_dir.join("Cargo.toml");
    let Ok(bytes) = fs::read(&cargo) else {
        return Vec::new();
    };
    let slice = if bytes.len() > 262_144 {
        &bytes[..262_144]
    } else {
        &bytes
    };
    let text = String::from_utf8_lossy(slice);
    let mut deps = BTreeSet::new();
    let mut in_deps = false;
    for line in text.lines() {
        let trimmed = line.split('#').next().unwrap_or("").trim();
        if trimmed.starts_with('[') {
            in_deps = trimmed == "[dependencies]" || trimmed.starts_with("[dependencies.");
            continue;
        }
        if !in_deps {
            continue;
        }
        if let Some(eq) = trimmed.find('=') {
            let key = trimmed[..eq].trim();
            let val = trimmed[eq + 1..].trim();
            if val.contains("path") {
                // path = "../foo" or { path = "..." }
                if let Some(start) = val.find('"') {
                    let rest = &val[start + 1..];
                    if let Some(end) = rest.find('"') {
                        let p = &rest[..end];
                        let name = Path::new(p)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| key.to_string());
                        deps.insert(sanitize_module_id(&name));
                    }
                }
            } else if !key.is_empty() && key.starts_with("dare-") {
                // workspace path deps often just `dare-core = { workspace = true }`
                deps.insert(sanitize_module_id(key));
            }
        }
    }
    deps.into_iter().collect()
}

/// Discover modules under `root` (absolute project path).
pub fn analyze_modules(root: &Path, filter: &[String]) -> CoreResult<Vec<ModuleFact>> {
    let mut modules: Vec<ModuleFact> = Vec::new();
    let crates = root.join("crates");
    if crates.is_dir() {
        let Ok(entries) = fs::read_dir(&crates) else {
            return Err(CoreError::io(format!(
                "cannot read crates dir: {}",
                crates.display()
            )));
        };
        let mut names: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        names.sort();
        for path in names {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "crate".into());
            if is_skip_dir(&name) {
                continue;
            }
            let id = sanitize_module_id(&name);
            let (loc, file_count, languages) = count_loc_and_langs(&path);
            if file_count == 0 {
                continue;
            }
            let depends_on = rust_path_deps(&path);
            modules.push(ModuleFact {
                id,
                path: to_posix_rel(root, &path),
                languages,
                loc,
                file_count,
                depends_on,
            });
            if modules.len() >= MAX_MODULES {
                break;
            }
        }
    }

    if modules.is_empty() {
        let src = root.join("src");
        if src.is_dir() {
            let (loc, file_count, languages) = count_loc_and_langs(&src);
            if file_count > 0 {
                modules.push(ModuleFact {
                    id: "src".into(),
                    path: "src".into(),
                    languages,
                    loc,
                    file_count,
                    depends_on: Vec::new(),
                });
            }
        }
    }

    if modules.is_empty() {
        // Top-level source-bearing directories
        if let Ok(entries) = fs::read_dir(root) {
            let mut dirs: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            dirs.sort();
            for path in dirs {
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                if is_skip_dir(&name) {
                    continue;
                }
                let (loc, file_count, languages) = count_loc_and_langs(&path);
                if file_count == 0 {
                    continue;
                }
                modules.push(ModuleFact {
                    id: sanitize_module_id(&name),
                    path: to_posix_rel(root, &path),
                    languages,
                    loc,
                    file_count,
                    depends_on: Vec::new(),
                });
                if modules.len() >= MAX_MODULES {
                    break;
                }
            }
        }
    }

    modules.sort_by(|a, b| a.id.cmp(&b.id));

    if !filter.is_empty() {
        let wanted: BTreeSet<String> = filter.iter().map(|s| s.trim().to_string()).collect();
        for w in &wanted {
            if w.is_empty() {
                return Err(CoreError::invalid_input("empty module id in --modules"));
            }
        }
        modules.retain(|m| wanted.contains(&m.id));
        if modules.is_empty() {
            return Err(CoreError::invalid_input(
                "no modules matched --modules filter",
            ));
        }
    }

    Ok(modules)
}

fn collect_source_files(root: &Path, modules: &[ModuleFact]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for m in modules {
        let dir = root.join(&m.path);
        let mut stack = vec![dir];
        let mut visited = 0usize;
        while let Some(cur) = stack.pop() {
            if visited >= MAX_WALK_ENTRIES || files.len() >= MAX_AST_FILES {
                break;
            }
            visited += 1;
            let Ok(entries) = fs::read_dir(&cur) else {
                continue;
            };
            for ent in entries.flatten() {
                let path = ent.path();
                let name = ent.file_name().to_string_lossy().to_string();
                if path.is_dir() {
                    if !is_skip_dir(&name) {
                        stack.push(path);
                    }
                } else if path.is_file() && is_source_file(&name) {
                    files.push(path);
                    if files.len() >= MAX_AST_FILES {
                        break;
                    }
                }
            }
        }
    }
    files.sort();
    files
}

/// Run optional AST pass; merge is stable (dedupe by method+path / kind+name, prefer first = AST order from engine).
pub fn analyze_ast(root: &Path, modules: &[ModuleFact]) -> AstSummary {
    let mut summary = AstSummary::default();
    let files = collect_source_files(root, modules);
    let mut endpoints: BTreeMap<(String, String), AstEndpointFact> = BTreeMap::new();
    let mut entities: BTreeMap<(String, String), AstEntityFact> = BTreeMap::new();

    for path in files {
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        if meta.len() as usize > MAX_FILE_BYTES {
            summary
                .warnings
                .push(format!("file_too_large: {}", to_posix_rel(root, &path)));
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        if bytes.contains(&0) {
            summary
                .warnings
                .push(format!("nul_byte: {}", to_posix_rel(root, &path)));
            continue;
        }
        let Ok(source) = String::from_utf8(bytes) else {
            continue;
        };
        let rel = to_posix_rel(root, &path);
        match analyze_source(&rel, &source) {
            Ok(model) => {
                summary.files_scanned += 1;
                for ep in model.endpoints {
                    insert_endpoint(&mut endpoints, &rel, ep);
                }
                for ent in model.entities {
                    insert_entity(&mut entities, &rel, ent);
                }
                for w in model.warnings {
                    summary.warnings.push(format!("{rel}: {w}"));
                }
            }
            Err(e) => {
                summary
                    .warnings
                    .push(format!("ast_skip {rel}: {}", e.message()));
            }
        }
    }

    summary.endpoints = endpoints.into_values().collect();
    summary.entities = entities.into_values().collect();
    summary.endpoints.sort_by(|a, b| {
        (&a.method, &a.path, a.line, &a.file).cmp(&(&b.method, &b.path, b.line, &b.file))
    });
    summary.entities.sort_by(|a, b| {
        (&a.kind, &a.name, a.line, &a.file).cmp(&(&b.kind, &b.name, b.line, &b.file))
    });
    summary.warnings.sort();
    summary
}

fn source_kind_str(s: SourceKind) -> &'static str {
    match s {
        SourceKind::Ast => "ast",
        SourceKind::Regex => "regex",
    }
}

fn insert_endpoint(
    map: &mut BTreeMap<(String, String), AstEndpointFact>,
    file: &str,
    ep: HttpEndpoint,
) {
    let key = (ep.method.clone(), ep.path.clone());
    map.entry(key).or_insert(AstEndpointFact {
        method: ep.method,
        path: ep.path,
        line: ep.line,
        file: file.to_string(),
        source: source_kind_str(ep.source).to_string(),
    });
}

fn insert_entity(map: &mut BTreeMap<(String, String), AstEntityFact>, file: &str, ent: Entity) {
    let key = (ent.kind.clone(), ent.name.clone());
    map.entry(key).or_insert(AstEntityFact {
        name: ent.name,
        kind: ent.kind,
        line: ent.line,
        file: file.to_string(),
        source: source_kind_str(ent.source).to_string(),
    });
}

fn render_ideia(facts: &ReverseFacts) -> String {
    let mut table = String::from("| Módulo | Path | LOC | Arquivos | Linguagens | Depende de |\n");
    table.push_str("|--------|------|-----|----------|------------|------------|\n");
    for m in &facts.modules {
        table.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} | {} |\n",
            m.id,
            m.path,
            m.loc,
            m.file_count,
            if m.languages.is_empty() {
                "—".into()
            } else {
                m.languages.join(", ")
            },
            if m.depends_on.is_empty() {
                "—".into()
            } else {
                m.depends_on.join(", ")
            }
        ));
    }

    let stacks = if facts.stacks.is_empty() {
        "(none)".to_string()
    } else {
        facts.stacks.join(", ")
    };

    format!(
        r#"# IDEIA — Pré-arquitetura (engenharia reversa)

> Gerado por `dare reverse` (schemaVersion {schema}). Camada determinística.
> Preencha os marcadores AGENT com `/dare-reverse` ou `dare reverse --ai`.

## Stacks detectadas

🟢 {stacks}

## Mapa de Módulos

{table}

## Propósito Inferido

<!-- AGENT:BEGIN section="purpose" -->
<!-- AGENT: preencher propósito (2-4 frases) -->
<!-- AGENT:END section="purpose" -->

## Domínio & Conceitos

<!-- AGENT:BEGIN section="domain" -->
<!-- AGENT: entidades e glossário -->
<!-- AGENT:END section="domain" -->

## Modelo de Dados (reconstruído)

<!-- AGENT:BEGIN section="data-model" -->
<!-- AGENT: entidades, campos, relacionamentos -->
<!-- AGENT:END section="data-model" -->

## Superfície de API

<!-- AGENT:BEGIN section="api-surface" -->
<!-- AGENT: endpoints método/rota/propósito -->
<!-- AGENT:END section="api-surface" -->

## Fluxo do Sistema

<!-- AGENT:BEGIN section="system-flow" -->
<!-- AGENT: flowchart Mermaid -->
<!-- AGENT:END section="system-flow" -->

## ⚠️ Incertezas / Gaps

<!-- AGENT:BEGIN section="gaps" -->
<!-- AGENT: gaps e perguntas ao humano -->
<!-- AGENT:END section="gaps" -->
"#,
        schema = facts.schema_version,
        stacks = stacks,
        table = table,
    )
}

fn render_module_spec(m: &ModuleFact) -> String {
    format!(
        r#"# Módulo: {id}

> Gerado por `dare reverse`. Fatos estruturais 🟢 — não editar a tabela.

## Fatos

| Campo | Valor |
|-------|-------|
| id | `{id}` |
| path | `{path}` |
| loc | {loc} |
| files | {files} |
| languages | {langs} |
| depends_on | {deps} |

## Responsabilidade

<!-- AGENT:BEGIN section="responsibility" -->
<!-- AGENT: 1-3 frases -->
<!-- AGENT:END section="responsibility" -->

## Superfície Pública

<!-- AGENT:BEGIN section="public-surface" -->
<!-- AGENT: exports / endpoints -->
<!-- AGENT:END section="public-surface" -->

## Como Funciona (fluxo)

<!-- AGENT:BEGIN section="flow" -->
<!-- AGENT: sequenceDiagram Mermaid -->
<!-- AGENT:END section="flow" -->

## Dependências & Acoplamento

<!-- AGENT:BEGIN section="coupling" -->
<!-- AGENT: riscos de acoplamento -->
<!-- AGENT:END section="coupling" -->
"#,
        id = m.id,
        path = m.path,
        loc = m.loc,
        files = m.file_count,
        langs = if m.languages.is_empty() {
            "—".into()
        } else {
            m.languages.join(", ")
        },
        deps = if m.depends_on.is_empty() {
            "—".into()
        } else {
            m.depends_on.join(", ")
        },
    )
}

fn render_excalidraw(modules: &[ModuleFact]) -> String {
    let mut elements = Vec::new();
    for (i, m) in modules.iter().enumerate() {
        let x = 40.0 + (i % 4) as f64 * 220.0;
        let y = 40.0 + (i / 4) as f64 * 120.0;
        let id = format!("mod-{i}");
        elements.push(serde_json::json!({
            "id": id,
            "type": "rectangle",
            "x": x,
            "y": y,
            "width": 180,
            "height": 60,
            "angle": 0,
            "strokeColor": "#1e1e1e",
            "backgroundColor": "#ffffff",
            "fillStyle": "solid",
            "strokeWidth": 1,
            "roughness": 0,
            "opacity": 100,
            "groupIds": [],
            "roundness": null,
            "seed": i as u64 + 1,
            "version": 1,
            "versionNonce": i as u64 + 1000,
            "isDeleted": false,
            "boundElements": [{"id": format!("txt-{i}"), "type": "text"}],
            "updated": 1,
            "link": null,
            "locked": false
        }));
        elements.push(serde_json::json!({
            "id": format!("txt-{i}"),
            "type": "text",
            "x": x + 10.0,
            "y": y + 18.0,
            "width": 160,
            "height": 24,
            "angle": 0,
            "strokeColor": "#1e1e1e",
            "backgroundColor": "transparent",
            "fillStyle": "solid",
            "strokeWidth": 1,
            "roughness": 0,
            "opacity": 100,
            "groupIds": [],
            "roundness": null,
            "seed": i as u64 + 500,
            "version": 1,
            "versionNonce": i as u64 + 1500,
            "isDeleted": false,
            "boundElements": null,
            "updated": 1,
            "link": null,
            "locked": false,
            "text": m.id,
            "fontSize": 16,
            "fontFamily": 1,
            "textAlign": "center",
            "verticalAlign": "middle",
            "containerId": id,
            "originalText": m.id,
            "lineHeight": 1.25
        }));
    }
    serde_json::json!({
        "type": "excalidraw",
        "version": 2,
        "source": "dare reverse",
        "elements": elements,
        "appState": { "viewBackgroundColor": "#ffffff" },
        "files": {}
    })
    .to_string()
}

fn render_confidence(facts: &ReverseFacts) -> String {
    let mut open_markers = 0u64;
    // Deterministic index: module markers expected = 4 sections × N + 6 in IDEIA
    open_markers += 6;
    open_markers += facts.modules.len() as u64 * 4;
    format!(
        r#"# Confidence report

> Gerado por `dare reverse --report` (determinístico).

| Métrica | Valor |
|---------|-------|
| modules | {mods} |
| open AGENT sections (expected) | {open} |
| ast | {ast} |
| deep | {deep} |

Preencha os marcadores com `/dare-reverse` e rode `--report` de novo após edição manual se necessário.
"#,
        mods = facts.modules.len(),
        open = open_markers,
        ast = facts.ast.is_some(),
        deep = facts.deep,
    )
}

fn write_rel(
    root: &ProjectRoot,
    rel: &str,
    data: &str,
    written: &mut Vec<String>,
) -> CoreResult<()> {
    let safe = SafeRelativePath::new(rel)?;
    atomic_write(root, &safe, data.as_bytes())?;
    written.push(rel.replace('\\', "/"));
    Ok(())
}

/// Orchestrate reverse analysis and optional writes.
pub fn reverse(start: &Path, opts: &ReverseOptions) -> CoreResult<ReverseReport> {
    if !start.exists() || !start.is_dir() {
        return Err(CoreError::not_found(format!(
            "directory not found: {}",
            start.display()
        )));
    }

    let Some(pr) = find_project_root(start) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&pr)?;
    let detection = detect(&pr)?;
    let stacks: Vec<String> = detection.stacks.iter().map(|s| s.id.clone()).collect();

    let modules = analyze_modules(&pr, &opts.modules)?;
    let ast = if opts.ast {
        Some(analyze_ast(&pr, &modules))
    } else {
        None
    };

    let facts = ReverseFacts {
        schema_version: REVERSE_SCHEMA_VERSION,
        project_root: display_path(&pr),
        stacks,
        modules: modules.clone(),
        ast,
        deep: opts.deep,
    };

    let mut warnings = Vec::new();
    if let Some(ref a) = facts.ast {
        warnings.extend(a.warnings.iter().cloned());
    }

    let mut written = Vec::new();
    if !opts.check {
        // Ensure DARE/ exists
        let dare_rel = SafeRelativePath::new("DARE")?;
        let dare_abs = root.resolve(&dare_rel)?;
        fs::create_dir_all(dare_abs.as_path().as_std_path())
            .map_err(|e| CoreError::io(e.to_string()))?;

        write_rel(&root, IDEIA_REL, &render_ideia(&facts), &mut written)?;

        let facts_json = serde_json::to_string_pretty(&facts)
            .map_err(|e| CoreError::io(format!("serialize facts: {e}")))?;
        write_rel(&root, FACTS_REL, &facts_json, &mut written)?;

        for m in &facts.modules {
            let rel = format!("{REVERSE_DIR}/module-{}.md", m.id);
            write_rel(&root, &rel, &render_module_spec(m), &mut written)?;
        }

        if opts.excalidraw {
            write_rel(
                &root,
                EXCALIDRAW_REL,
                &render_excalidraw(&facts.modules),
                &mut written,
            )?;
        }

        if opts.deep {
            write_rel(
                &root,
                &format!("{REVERSE_DIR}/erd.md"),
                "# ERD\n\n<!-- AGENT: complete entities/relations -->\n",
                &mut written,
            )?;
            write_rel(
                &root,
                &format!("{REVERSE_DIR}/domain-rules.md"),
                "# Domain rules\n\n<!-- AGENT: business rules -->\n",
                &mut written,
            )?;
            write_rel(
                &root,
                &format!("{REVERSE_DIR}/state-machines.md"),
                "# State machines\n\n<!-- AGENT: stateDiagram-v2 -->\n",
                &mut written,
            )?;
            write_rel(
                &root,
                &format!("{REVERSE_DIR}/permissions.md"),
                "# Permissions\n\n<!-- AGENT: roles/resources -->\n",
                &mut written,
            )?;
            write_rel(
                &root,
                &format!("{REVERSE_DIR}/c4/c4-component.md"),
                &format!(
                    "# C4 Component (module map)\n\nDeterministic module list:\n\n{}\n",
                    facts
                        .modules
                        .iter()
                        .map(|m| format!("- `{}` (`{}`)", m.id, m.path))
                        .collect::<Vec<_>>()
                        .join("\n")
                ),
                &mut written,
            )?;
            write_rel(
                &root,
                &format!("{REVERSE_DIR}/c4/c4-context.md"),
                "# C4 Context\n\n<!-- AGENT: actors/external systems -->\n",
                &mut written,
            )?;
            write_rel(
                &root,
                &format!("{REVERSE_DIR}/c4/c4-container.md"),
                "# C4 Container\n\n<!-- AGENT: deploy containers -->\n",
                &mut written,
            )?;
        }

        if opts.report {
            write_rel(
                &root,
                CONFIDENCE_REL,
                &render_confidence(&facts),
                &mut written,
            )?;
        }

        written.sort();
    }

    Ok(ReverseReport {
        schema_version: REVERSE_SCHEMA_VERSION,
        mode: if opts.check {
            "check".into()
        } else {
            "reverse".into()
        },
        ok: true,
        project_root: display_path(&pr),
        module_count: facts.modules.len() as u64,
        written,
        warnings,
        enriched: false,
        check: opts.check,
        deep: opts.deep,
        ast: opts.ast,
        excalidraw: opts.excalidraw,
        report: opts.report,
    })
}

/// Human-readable report (en-US).
pub fn format_reverse_human(r: &ReverseReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("schemaVersion: {}\n", r.schema_version));
    out.push_str(&format!("mode: {}\n", r.mode));
    out.push_str(&format!("projectRoot: {}\n", r.project_root));
    out.push_str(&format!("moduleCount: {}\n", r.module_count));
    out.push_str(&format!("ok: {}\n", r.ok));
    out.push_str(&format!("enriched: {}\n", r.enriched));
    out.push_str(&format!(
        "flags: check={} deep={} ast={} excalidraw={} report={}\n",
        r.check, r.deep, r.ast, r.excalidraw, r.report
    ));
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
    if r.check {
        out.push_str(MSG_CHECK);
        out.push('\n');
    } else {
        out.push_str("mode: reverse (artifacts written)\n");
    }
    out
}

pub fn reverse_report_to_json(r: &ReverseReport) -> Value {
    serde_json::to_value(r).unwrap_or(Value::Null)
}

/// Read IDEIA.md for enrichment inject (CLI).
pub fn read_ideia(root: &ProjectRoot) -> CoreResult<String> {
    let rel = SafeRelativePath::new(IDEIA_REL)?;
    read_to_string(root, &rel)
}

/// Overwrite IDEIA.md after enrichment.
pub fn write_ideia(root: &ProjectRoot, content: &str) -> CoreResult<()> {
    let rel = SafeRelativePath::new(IDEIA_REL)?;
    atomic_write(root, &rel, content.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_crate(root: &Path, name: &str, src: &str) {
        let dir = root.join("crates").join(name).join("src");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            root.join("crates").join(name).join("Cargo.toml"),
            format!("[package]\nname=\"{name}\"\nversion=\"0.0.0\"\nedition=\"2021\"\n"),
        )
        .unwrap();
        fs::write(dir.join("lib.rs"), src).unwrap();
    }

    #[test]
    fn analyze_modules_crates_sorted() {
        let dir = tempdir().unwrap();
        write_crate(dir.path(), "zeta", "pub fn z() {}\n");
        write_crate(dir.path(), "alpha", "pub fn a() {}\n");
        fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers=[\"crates/*\"]\n",
        )
        .unwrap();
        let mods = analyze_modules(dir.path(), &[]).unwrap();
        assert_eq!(mods.len(), 2);
        assert_eq!(mods[0].id, "alpha");
        assert_eq!(mods[1].id, "zeta");
    }

    #[test]
    fn modules_filter_and_check_no_write() {
        let dir = tempdir().unwrap();
        write_crate(dir.path(), "alpha", "pub fn a() {}\n");
        write_crate(dir.path(), "beta", "pub fn b() {}\n");
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let before: BTreeSet<_> = fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect();
        let report = reverse(
            dir.path(),
            &ReverseOptions {
                check: true,
                modules: vec!["alpha".into()],
                excalidraw: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(report.check);
        assert_eq!(report.module_count, 1);
        assert!(report.written.is_empty());
        let after: BTreeSet<_> = fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect();
        assert_eq!(before, after);
    }

    #[test]
    fn reverse_writes_ideia() {
        let dir = tempdir().unwrap();
        write_crate(dir.path(), "svc", "pub struct S;\n");
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        let report = reverse(
            dir.path(),
            &ReverseOptions {
                check: false,
                deep: true,
                excalidraw: true,
                report: true,
                ast: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!report.written.is_empty());
        assert!(dir.path().join("DARE").join("IDEIA.md").is_file());
        assert!(dir
            .path()
            .join("DARE")
            .join("REVERSE")
            .join("reverse-facts.json")
            .is_file());
        assert!(dir
            .path()
            .join("DARE")
            .join("REVERSE")
            .join("module-svc.md")
            .is_file());
        let ideia = fs::read_to_string(dir.path().join("DARE").join("IDEIA.md")).unwrap();
        assert!(ideia.contains("AGENT:BEGIN section=\"purpose\""));
        assert!(ideia.contains("`svc`"));
    }

    #[test]
    fn missing_dir_not_found() {
        let err = reverse(
            Path::new("__no_such_reverse_dir__"),
            &ReverseOptions::default(),
        )
        .unwrap_err();
        assert!(matches!(err, CoreError::NotFound(_)));
    }

    #[test]
    fn ast_merge_stable_keys() {
        let dir = tempdir().unwrap();
        write_crate(
            dir.path(),
            "api",
            r#"
pub struct User {}
"#,
        );
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        let mods = analyze_modules(dir.path(), &[]).unwrap();
        let a1 = analyze_ast(dir.path(), &mods);
        let a2 = analyze_ast(dir.path(), &mods);
        assert_eq!(a1.entities, a2.entities);
        assert_eq!(a1.endpoints, a2.endpoints);
    }
}
