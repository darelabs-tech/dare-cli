//! `dare blueprint` — bundle determinístico Design → BLUEPRINT/TASKS/DAG/EXECUTION (025).

use std::collections::BTreeMap;
use std::path::Path;

use dare_ai::{
    inject_sections, parse_and_validate_sections_with, resolve_provider, EnrichRequest, ProviderId,
};
use dare_contracts::parse_dag_yaml;
use dare_core::fs::{atomic_write, read_to_string};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use dare_dag::{validate_dag, ValidateFsContext, ValidateOptions, ValidationReport};
use dare_project::find_project_root;
use serde::Serialize;
use serde_json::{json, Value};

pub const DEFAULT_DESIGN_REL: &str = "DARE/DESIGN.md";
pub const OUT_BLUEPRINT: &str = "DARE/BLUEPRINT.md";
pub const OUT_TASKS: &str = "DARE/TASKS.md";
pub const OUT_DAG: &str = "DARE/dare-dag.yaml";
pub const OUT_EXEC_DIR: &str = "DARE/EXECUTION";
pub const BLUEPRINT_SCHEMA_VERSION: u32 = 1;
pub const DESIGN_READ_CAP: usize = 262_144;
pub const ARTIFACT_WRITE_CAP: usize = 1_048_576;
pub const MANAGED_MD: &str = "<!-- dare:managed -->";
pub const MANAGED_YAML: &str = "# dare:managed";

pub const MARKER_BEGIN: &str = "<!-- AGENT:BEGIN section=\"";
pub const MARKER_END_PREFIX: &str = "<!-- AGENT:END section=\"";

/// Blueprint enrichable section ids (stable).
pub const BP_ENRICHABLE: &[&str] = &[
    "architecture-overview",
    "execution-phases",
    "api-contracts",
    "data-model",
];

#[derive(Debug, Clone)]
pub struct BlueprintInput {
    pub design_rel_or_abs: Option<std::path::PathBuf>,
    pub force: bool,
    pub ai: bool,
    pub provider: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GeneratedBundle {
    pub blueprint_md: String,
    pub tasks_md: String,
    pub dag_yaml: String,
    pub specs: BTreeMap<String, String>,
    pub task_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlueprintReport {
    pub schema_version: u32,
    pub mode: String,
    pub ok: bool,
    pub design_path: String,
    pub force: bool,
    pub ai: bool,
    pub provider: Option<String>,
    pub enriched: bool,
    pub written: Vec<String>,
    pub kept: Vec<String>,
    pub task_count: u32,
    pub validate_ok: bool,
    pub warnings: Vec<String>,
    pub validation: Option<Value>,
}

#[derive(Debug, Clone)]
struct TaskDef {
    id: String,
    title: String,
    depends_on: Vec<String>,
    complexity: String,
}

fn marker_begin(section: &str) -> String {
    format!("{MARKER_BEGIN}{section}\" -->")
}

fn marker_end(section: &str) -> String {
    format!("{MARKER_END_PREFIX}{section}\" -->")
}

fn utc_today_yyyy_mm_dd() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (y, m, d) = days_since_epoch_to_ymd(secs / 86_400);
    format!("{y:04}-{m:02}-{d:02}")
}

fn days_since_epoch_to_ymd(days: u64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut y = (yoe + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 {
        (mp + 3) as u32
    } else {
        (mp - 9) as u32
    };
    if m <= 2 {
        y += 1;
    }
    (y, m, d)
}

fn resolve_date_from_design(design: &str) -> String {
    for line in design.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("> **Data:**") {
            let date = rest.trim().trim_start_matches('|').trim();
            if !date.is_empty() {
                return date.to_string();
            }
        }
    }
    utc_today_yyyy_mm_dd()
}

fn first_non_empty_lines(content: &str, max: usize) -> Vec<&str> {
    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .take(max)
        .collect()
}

/// True when the first non-empty line contains `MANAGED_MD`.
pub fn is_managed_markdown(content: &str) -> bool {
    first_non_empty_lines(content.trim_start(), 1)
        .first()
        .is_some_and(|line| line.contains(MANAGED_MD))
}

/// True when the first non-empty line contains `MANAGED_YAML`.
pub fn is_managed_yaml(content: &str) -> bool {
    first_non_empty_lines(content.trim_start(), 1)
        .first()
        .is_some_and(|line| line.contains(MANAGED_YAML))
}

/// First `# DESIGN: …` line, else first H1, else `"Untitled"`.
pub fn parse_design_title(design: &str) -> String {
    for line in design.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# DESIGN:") {
            let title = rest.trim();
            if !title.is_empty() {
                return title.to_string();
            }
        }
    }
    for line in design.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let title = rest.trim();
            if !title.is_empty() {
                return title.to_string();
            }
        }
    }
    "Untitled".into()
}

fn is_table_separator(line: &str) -> bool {
    let t = line.trim();
    t.starts_with('|') && t.contains("---")
}

fn parse_table_row(line: &str) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return None;
    }
    let cells: Vec<String> = trimmed
        .trim_start_matches('|')
        .trim_end_matches('|')
        .split('|')
        .map(|c| c.trim().trim_matches('`').to_string())
        .collect();
    if cells.len() < 3 {
        return None;
    }
    Some(cells)
}

/// Rows in the RF table where the priority cell is MUST — max 8, stable file order.
pub fn extract_must_requirements(design: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut in_rf = false;
    let mut past_header = false;

    for line in design.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ")
            && trimmed
                .to_ascii_lowercase()
                .contains("requisitos funcionais")
        {
            in_rf = true;
            past_header = false;
            continue;
        }
        if in_rf && trimmed.starts_with("## ") {
            break;
        }
        if !in_rf {
            continue;
        }
        if is_table_separator(trimmed) {
            past_header = true;
            continue;
        }
        if !past_header {
            continue;
        }
        let Some(cells) = parse_table_row(trimmed) else {
            continue;
        };
        let id = cells[0].clone();
        if !id.starts_with("RF-") {
            continue;
        }
        let requisito = cells[1].clone();
        let priority = cells[2].to_ascii_uppercase();
        if priority.contains("MUST") {
            out.push((id, requisito));
            if out.len() >= 8 {
                break;
            }
        }
    }
    out
}

fn truncate_title(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    text.chars().take(max).collect()
}

fn subtask_prompt(title: &str) -> String {
    format!(
        "Implement task \"{title}\". Follow the architecture and acceptance criteria in \
         DARE/BLUEPRINT.md. Run tests scoped to this task. Follow DARE/BLUEPRINT.md; no git commit."
    )
}

fn build_task_defs(must_rfs: &[(String, String)]) -> Vec<TaskDef> {
    let mut tasks = vec![
        TaskDef {
            id: "task-001".into(),
            title: "Verify docker-compose / container baseline".into(),
            depends_on: vec![],
            complexity: "LOW".into(),
        },
        TaskDef {
            id: "task-002".into(),
            title: "Implement core from design".into(),
            depends_on: vec![],
            complexity: "MED".into(),
        },
    ];

    for (i, (rf_id, requisito)) in must_rfs.iter().enumerate() {
        let num = 3 + i;
        let title = format!("{rf_id}: {}", truncate_title(requisito, 60));
        tasks.push(TaskDef {
            id: format!("task-{num:03}"),
            title,
            depends_on: vec!["task-002".into()],
            complexity: "MED".into(),
        });
    }

    let audit_deps: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
    tasks.push(TaskDef {
        id: "task-audit".into(),
        title: "Ralph audit fmt/clippy/test".into(),
        depends_on: audit_deps,
        complexity: "MED".into(),
    });
    tasks.push(TaskDef {
        id: "task-close".into(),
        title: "Closeout checklist".into(),
        depends_on: vec!["task-audit".into()],
        complexity: "LOW".into(),
    });

    tasks
}

fn yaml_escape(s: &str) -> String {
    if s.contains(':') || s.contains('"') || s.contains('\n') {
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

fn render_dag_yaml(title: &str, tasks: &[TaskDef]) -> String {
    let mut out = String::new();
    out.push_str(MANAGED_YAML);
    out.push('\n');
    out.push_str(&format!("title: \"{} - Development Tasks\"\n", title));
    out.push_str("version: \"1.0.0\"\n\n");
    out.push_str("limits:\n");
    out.push_str("  parent_context_chars: 2000\n");
    out.push_str("  task_output_chars: 4000\n");
    out.push_str("  timeout_seconds: 600\n\n");
    out.push_str("models:\n");
    out.push_str(
        "  cursor:      { HIGH: gpt-5.3-codex,     MED: composer-2,       LOW: auto-low }\n",
    );
    out.push_str(
        "  claude:      { HIGH: claude-sonnet-4-5, MED: claude-haiku-4,   LOW: claude-haiku-4 }\n",
    );
    out.push_str("  antigravity: { HIGH: gemini-2.5-pro,    MED: gemini-2.5-flash, LOW: gemini-2.5-flash }\n\n");
    out.push_str("tasks:\n");

    for task in tasks {
        out.push_str(&format!("  - id: {}\n", task.id));
        out.push_str(&format!("    title: {}\n", yaml_escape(&task.title)));
        if task.depends_on.is_empty() {
            out.push_str("    depends_on: []\n");
        } else {
            out.push_str("    depends_on: [");
            for (i, dep) in task.depends_on.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(dep);
            }
            out.push_str("]\n");
        }
        out.push_str(&format!("    complexity: {}\n", task.complexity));
        out.push_str(&format!("    spec_file: EXECUTION/{}.md\n", task.id));
        out.push_str("    subtask_prompt: |\n");
        for line in subtask_prompt(&task.title).lines() {
            out.push_str("      ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn render_execution_spec(id: &str, title: &str) -> String {
    format!(
        "{MANAGED_MD}\n\
         # Task {id}: {title}\n\n\
         ## Objetivo\n\
         {title}\n\n\
         ## Validation Gates\n\
         - [ ] Behavior matches BLUEPRINT\n\
         - [ ] Tests pass for this task scope\n\
         - [ ] No git commit\n\n\
         ## Definition of Done (ANTI-STUB)\n\
         - [ ] No todo!/unimplemented in public paths\n\
         - [ ] No git commit\n"
    )
}

fn extract_section_by_heading(design: &str, heading_needle: &str) -> Option<String> {
    let mut lines = Vec::new();
    let mut capturing = false;
    for line in design.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            if capturing {
                break;
            }
            if trimmed.to_ascii_lowercase().contains(heading_needle) {
                capturing = true;
                lines.push(line.to_string());
                continue;
            }
        } else if capturing {
            if trimmed.starts_with("---") {
                break;
            }
            lines.push(line.to_string());
        }
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}

fn extract_design_description(design: &str) -> String {
    extract_section_by_heading(design, "descri")
        .map(|s| {
            s.lines()
                .skip(1)
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "[Derived from DESIGN — no description section found]".into())
}

fn rf_ids_list(must_rfs: &[(String, String)]) -> String {
    if must_rfs.is_empty() {
        "[A definir — derived in later refinement]".into()
    } else {
        must_rfs
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn execution_phases_list(tasks: &[TaskDef]) -> String {
    let mut out = String::new();
    out.push_str("### Generated execution phases\n\n");
    for (i, task) in tasks.iter().enumerate() {
        if task.id == "task-audit" || task.id == "task-close" {
            continue;
        }
        out.push_str(&format!(
            "#### Fase {}: {}\n**Task:** `{}` · **Complexity:** {}\n\n",
            i + 1,
            task.title,
            task.id,
            task.complexity
        ));
    }
    out.push_str("#### Fase N-1: Ralph audit\n**Task:** `task-audit`\n\n");
    out.push_str("#### Fase N: Closeout\n**Task:** `task-close`\n");
    out
}

fn render_blueprint_md(
    design: &str,
    title: &str,
    must_rfs: &[(String, String)],
    tasks: &[TaskDef],
) -> String {
    let date = resolve_date_from_design(design);
    let description = extract_design_description(design);
    let rf_annex = extract_section_by_heading(design, "requisitos funcionais");
    let rnf_annex = extract_section_by_heading(design, "requisitos n");
    let stack_annex = extract_section_by_heading(design, "stack t");

    let mut out = String::new();
    out.push_str(MANAGED_MD);
    out.push_str("\n\n");
    out.push_str(&format!("# BLUEPRINT: {title}\n\n"));
    out.push_str(&format!(
        "> **Gerado a partir de:** `DARE/DESIGN.md` v1.0  \n\
         > **Data:** {date} | **Status:** DRAFT\n\n"
    ));
    out.push_str("---\n\n");

    out.push_str("## 1. VISÃO GERAL DA ARQUITETURA\n\n");
    out.push_str(&marker_begin("architecture-overview"));
    out.push('\n');
    out.push_str(&description);
    out.push('\n');
    out.push_str(&marker_end("architecture-overview"));
    out.push_str("\n\n---\n\n");

    out.push_str("## 2. STACK TÉCNICA DEFINIDA\n\n");
    if let Some(ref stack) = stack_annex {
        out.push_str(stack);
        out.push_str("\n\n");
    } else {
        out.push_str("| Camada | Tecnologia | Versão | Papel |\n");
        out.push_str("|--------|-----------|--------|-------|\n");
        out.push_str("| [A definir] | | | |\n\n");
    }
    out.push_str("---\n\n");

    out.push_str("## 3. ESTRUTURA DE PASTAS E ARQUIVOS\n\n");
    out.push_str("```text\n");
    out.push_str("[project]/\n");
    out.push_str("├── src/\n");
    out.push_str("├── DARE/\n");
    out.push_str("│   ├── DESIGN.md\n");
    out.push_str("│   ├── BLUEPRINT.md\n");
    out.push_str("│   ├── TASKS.md\n");
    out.push_str("│   ├── dare-dag.yaml\n");
    out.push_str("│   └── EXECUTION/\n");
    out.push_str("└── docker-compose.yml\n");
    out.push_str("```\n\n---\n\n");

    out.push_str("## 4. MODELO DE DADOS\n\n");
    out.push_str(&marker_begin("data-model"));
    out.push('\n');
    if let Some(ref rf) = rf_annex {
        out.push_str(rf);
        out.push('\n');
    } else {
        out.push_str("[Derived from DESIGN RF table]\n");
    }
    out.push_str(&marker_end("data-model"));
    out.push_str("\n\n---\n\n");

    out.push_str("## 5. CONTRATOS DE API\n\n");
    out.push_str(&marker_begin("api-contracts"));
    out.push('\n');
    out.push_str("[A definir — derived in later refinement]\n\n");
    out.push_str("Functional requirement IDs: ");
    out.push_str(&rf_ids_list(must_rfs));
    out.push('\n');
    out.push_str(&marker_end("api-contracts"));
    out.push_str("\n\n---\n\n");

    out.push_str("## 6. PLANO DE EXECUÇÃO (FASES)\n\n");
    out.push_str(&marker_begin("execution-phases"));
    out.push('\n');
    out.push_str(&execution_phases_list(tasks));
    out.push_str(&marker_end("execution-phases"));
    out.push_str("\n\n---\n\n");

    if let Some(ref rnf) = rnf_annex {
        out.push_str("## ANEXO — REQUISITOS NÃO-FUNCIONAIS (do Design)\n\n");
        out.push_str(rnf);
        out.push_str("\n\n---\n\n");
    }

    out.push_str("## 7. VALIDATION GATES POR STACK\n\n");
    out.push_str("| Stack | Build | Test | Lint / Audit |\n");
    out.push_str("|-------|-------|------|--------------|\n");
    out.push_str(
        "| Rust | `cargo build` | `cargo test --workspace` | `cargo clippy && cargo audit` |\n\n",
    );

    out.push_str("## 8. CHECKLIST DE APROVAÇÃO DO BLUEPRINT\n\n");
    out.push_str("- [ ] Arquitetura revisada e aprovada\n");
    out.push_str("- [ ] Modelo de dados validado\n");
    out.push_str("- [ ] Contratos de API definidos\n");
    out.push_str("- [ ] Fases com critérios de DONE claros\n");
    out.push_str("- [ ] DAG de tasks gerado (`dare-dag.yaml`)\n");

    out
}

fn render_tasks_md(tasks: &[TaskDef]) -> String {
    let mut out = String::new();
    out.push_str(MANAGED_MD);
    out.push_str("\n\n# TASKS\n\n");
    out.push_str("| ID | Title | Depends on | Complexity | Status |\n");
    out.push_str("|----|-------|------------|------------|--------|\n");
    for task in tasks {
        let deps = if task.depends_on.is_empty() {
            "—".to_string()
        } else {
            task.depends_on.join(", ")
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | PENDING |\n",
            task.id, task.title, deps, task.complexity
        ));
    }
    out.push_str("\n## Fases\n\n");
    for task in tasks {
        out.push_str(&format!(
            "- **{}** — {} ({})\n",
            task.id, task.title, task.complexity
        ));
    }
    out
}

fn check_artifact_cap(label: &str, content: &str) -> CoreResult<()> {
    if content.len() > ARTIFACT_WRITE_CAP {
        return Err(CoreError::invalid_input(format!(
            "{label} exceeds artifact write cap ({ARTIFACT_WRITE_CAP} bytes)"
        )));
    }
    Ok(())
}

/// Deterministic bundle from Design text and resolved title (§5.3–5.4).
pub fn generate_bundle(design: &str, title: &str) -> CoreResult<GeneratedBundle> {
    let must_rfs = extract_must_requirements(design);
    let tasks = build_task_defs(&must_rfs);
    let task_ids: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();

    let blueprint_md = render_blueprint_md(design, title, &must_rfs, &tasks);
    let tasks_md = render_tasks_md(&tasks);
    let dag_yaml = render_dag_yaml(title, &tasks);

    check_artifact_cap("BLUEPRINT.md", &blueprint_md)?;
    check_artifact_cap("TASKS.md", &tasks_md)?;
    check_artifact_cap("dare-dag.yaml", &dag_yaml)?;

    let mut specs = BTreeMap::new();
    for task in &tasks {
        let body = render_execution_spec(&task.id, &task.title);
        check_artifact_cap(&format!("EXECUTION/{}.md", task.id), &body)?;
        specs.insert(format!("EXECUTION/{}.md", task.id), body);
    }

    Ok(GeneratedBundle {
        blueprint_md,
        tasks_md,
        dag_yaml,
        specs,
        task_ids,
    })
}

fn blueprint_stage_pid() -> u32 {
    std::process::id()
}

fn blueprint_stage_rel_prefix() -> String {
    format!(".dare/blueprint-stage-{}/", blueprint_stage_pid())
}

fn ensure_dare_dir(root: &ProjectRoot) -> CoreResult<()> {
    let rel = SafeRelativePath::new("DARE")?;
    let abs = root.resolve(&rel)?;
    std::fs::create_dir_all(abs.as_path().as_std_path()).map_err(|e| CoreError::io(e.to_string()))
}

fn ensure_dot_dare_dir(root: &ProjectRoot) -> CoreResult<()> {
    let rel = SafeRelativePath::new(".dare")?;
    let abs = root.resolve(&rel)?;
    std::fs::create_dir_all(abs.as_path().as_std_path()).map_err(|e| CoreError::io(e.to_string()))
}

fn write_stage_file(root: &ProjectRoot, rel_str: &str, data: &[u8]) -> CoreResult<()> {
    let rel = SafeRelativePath::new(rel_str)?;
    atomic_write(root, &rel, data)
}

fn write_bundle_to_stage(root: &ProjectRoot, bundle: &GeneratedBundle) -> CoreResult<()> {
    ensure_dot_dare_dir(root)?;
    let prefix = blueprint_stage_rel_prefix();
    write_stage_file(
        root,
        &format!("{prefix}DARE/BLUEPRINT.md"),
        bundle.blueprint_md.as_bytes(),
    )?;
    write_stage_file(
        root,
        &format!("{prefix}DARE/TASKS.md"),
        bundle.tasks_md.as_bytes(),
    )?;
    write_stage_file(
        root,
        &format!("{prefix}DARE/dare-dag.yaml"),
        bundle.dag_yaml.as_bytes(),
    )?;
    for (spec_rel, content) in &bundle.specs {
        write_stage_file(
            root,
            &format!("{prefix}DARE/{spec_rel}"),
            content.as_bytes(),
        )?;
    }
    Ok(())
}

/// Best-effort removal of the current process staging directory.
pub fn purge_blueprint_stage(root: &ProjectRoot) {
    let rel_str = format!(".dare/blueprint-stage-{}", blueprint_stage_pid());
    if let Ok(rel) = SafeRelativePath::new(&rel_str) {
        if let Ok(abs) = root.resolve(&rel) {
            let _ = std::fs::remove_dir_all(abs.as_path().as_std_path());
        }
    }
}

fn promote_one(
    root: &ProjectRoot,
    dest_rel: &str,
    content: &str,
    force: bool,
    is_managed: fn(&str) -> bool,
    written: &mut Vec<String>,
    kept: &mut Vec<String>,
) -> CoreResult<()> {
    let rel = SafeRelativePath::new(dest_rel)?;
    let abs = root.resolve(&rel)?;
    let path = abs.as_path().as_std_path();

    if path.is_file() && !force {
        if let Ok(existing) = read_to_string(root, &rel) {
            if !is_managed(&existing) {
                kept.push(dest_rel.to_string());
                return Ok(());
            }
        }
    }

    atomic_write(root, &rel, content.as_bytes())?;
    written.push(dest_rel.to_string());
    Ok(())
}

/// Write bundle under `.dare/blueprint-stage-{pid}/`, validate DAG; purge stage on failure.
pub fn stage_and_validate(
    root: &ProjectRoot,
    bundle: &GeneratedBundle,
) -> CoreResult<ValidationReport> {
    write_bundle_to_stage(root, bundle)?;

    let doc = parse_dag_yaml(&bundle.dag_yaml)?;
    let ctx = ValidateFsContext {
        root,
        dag_path_display: OUT_DAG.to_string(),
    };
    let report = validate_dag(&doc, &ValidateOptions { strict: false }, &ctx);

    if !report.ok {
        purge_blueprint_stage(root);
        return Err(CoreError::internal(format!(
            "DAG validation failed: {} error(s)",
            report.error_count
        )));
    }

    Ok(report)
}

fn ensure_exec_dir(root: &ProjectRoot) -> CoreResult<()> {
    let rel = SafeRelativePath::new(OUT_EXEC_DIR)?;
    let abs = root.resolve(&rel)?;
    std::fs::create_dir_all(abs.as_path().as_std_path()).map_err(|e| CoreError::io(e.to_string()))
}

/// Promote bundle artifacts into `DARE/` with managed keep / `--force` policy.
pub fn promote(
    root: &ProjectRoot,
    bundle: &GeneratedBundle,
    force: bool,
) -> CoreResult<(Vec<String>, Vec<String>)> {
    ensure_dare_dir(root)?;
    ensure_exec_dir(root)?;
    let mut written = Vec::new();
    let mut kept = Vec::new();

    promote_one(
        root,
        OUT_BLUEPRINT,
        &bundle.blueprint_md,
        force,
        is_managed_markdown,
        &mut written,
        &mut kept,
    )?;
    promote_one(
        root,
        OUT_TASKS,
        &bundle.tasks_md,
        force,
        is_managed_markdown,
        &mut written,
        &mut kept,
    )?;
    promote_one(
        root,
        OUT_DAG,
        &bundle.dag_yaml,
        force,
        is_managed_yaml,
        &mut written,
        &mut kept,
    )?;

    for (spec_rel, content) in &bundle.specs {
        let dest = format!("DARE/{spec_rel}");
        promote_one(
            root,
            &dest,
            content,
            force,
            is_managed_markdown,
            &mut written,
            &mut kept,
        )?;
    }

    purge_blueprint_stage(root);
    Ok((written, kept))
}

fn resolve_design_rel(
    root: &ProjectRoot,
    design: Option<&Path>,
) -> CoreResult<(SafeRelativePath, String)> {
    let Some(design) = design else {
        let rel = SafeRelativePath::new(DEFAULT_DESIGN_REL)?;
        return Ok((rel, DEFAULT_DESIGN_REL.to_string()));
    };

    if design.is_absolute() {
        let design_canon = if design.exists() {
            std::fs::canonicalize(design).map_err(|e| CoreError::io(e.to_string()))?
        } else {
            design.to_path_buf()
        };
        let root_std = root.as_path().as_std_path();
        let rel = design_canon
            .strip_prefix(root_std)
            .map_err(|_| CoreError::invalid_input("design path is outside project root"))?;
        let s = rel.to_string_lossy().replace('\\', "/");
        if s.is_empty() {
            return Err(CoreError::invalid_input("invalid design path"));
        }
        if !design.exists() {
            return Err(CoreError::not_found(format!("file not found: {s}")));
        }
        return Ok((SafeRelativePath::new(&s)?, s));
    }

    let joined = root.as_path().as_std_path().join(design);
    let s = design.to_string_lossy().replace('\\', "/");
    if !joined.exists() {
        return Err(CoreError::not_found(format!("file not found: {s}")));
    }
    Ok((SafeRelativePath::new(&s)?, s))
}

fn read_design_capped(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<String> {
    let abs = root.resolve(rel)?;
    let meta = std::fs::metadata(abs.as_path().as_std_path()).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CoreError::not_found(format!("file not found: {}", rel.as_str()))
        } else {
            CoreError::io(e.to_string())
        }
    })?;
    if meta.len() > DESIGN_READ_CAP as u64 {
        return Err(CoreError::invalid_input(format!(
            "file exceeds design read cap ({} bytes): {}",
            DESIGN_READ_CAP,
            rel.as_str()
        )));
    }
    let content = read_to_string(root, rel)?;
    if content.trim().is_empty() {
        return Err(CoreError::invalid_input("design file must not be empty"));
    }
    Ok(content)
}

fn validation_to_json(report: &ValidationReport) -> Value {
    serde_json::to_value(report).unwrap_or(Value::Null)
}

fn kept_warnings(kept: &[String]) -> Vec<String> {
    kept.iter()
        .map(|path| format!("kept unmanaged artifact: {path}"))
        .collect()
}

/// Optional AI enrichment for `BLUEPRINT.md` sections (soft-fail).
pub fn maybe_enrich_blueprint(
    bundle: &mut GeneratedBundle,
    design: &str,
    title: &str,
    root: &ProjectRoot,
    provider: ProviderId,
) -> (bool, Vec<String>) {
    let mut warnings = Vec::new();

    let prov = match resolve_provider(provider) {
        Ok(p) => p,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return (false, warnings);
        }
    };

    let description = extract_design_description(design);
    let cwd_rel = match SafeRelativePath::new("DARE") {
        Ok(r) => r,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return (false, warnings);
        }
    };
    let req = EnrichRequest {
        command: "blueprint".into(),
        title: title.to_string(),
        description,
        current_markdown: bundle.blueprint_md.clone(),
        cwd: Some((root.clone(), cwd_rel)),
    };

    let raw = match prov.enrich(&req) {
        Ok(r) => r,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return (false, warnings);
        }
    };

    let sections = match parse_and_validate_sections_with(&raw.stdout, BP_ENRICHABLE) {
        Ok(s) => s,
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            return (false, warnings);
        }
    };

    match inject_sections(&bundle.blueprint_md, &sections, BP_ENRICHABLE) {
        Ok(injected) => {
            bundle.blueprint_md = injected;
            (true, warnings)
        }
        Err(e) => {
            warnings.push(format!("AI enrichment skipped: {}", e.message()));
            (false, warnings)
        }
    }
}

/// CLI entry: resolve project root, read design, generate bundle, optional AI, stage/validate/promote.
pub fn run_blueprint(input: BlueprintInput) -> CoreResult<(String, Value)> {
    if input.provider.is_some() && !input.ai {
        return Err(CoreError::usage("--provider requires --ai"));
    }

    let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
    let Some(root_path) = find_project_root(&cwd) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&root_path)?;

    let (design_rel, design_path) = resolve_design_rel(&root, input.design_rel_or_abs.as_deref())?;
    let design = read_design_capped(&root, &design_rel)?;
    let title = parse_design_title(&design);

    let mut bundle = generate_bundle(&design, &title)?;
    let task_count = bundle.task_ids.len() as u32;

    let (provider_id, provider_str) = if input.ai {
        let pid = match input.provider.as_deref() {
            None => ProviderId::Codex,
            Some(s) => ProviderId::parse(s)?,
        };
        (Some(pid), Some(pid.as_str().to_string()))
    } else {
        (None, None)
    };

    let (enriched, mut warnings) = if let Some(pid) = provider_id {
        maybe_enrich_blueprint(&mut bundle, &design, &title, &root, pid)
    } else {
        (false, Vec::new())
    };

    let validation_report = stage_and_validate(&root, &bundle)?;

    let (written, kept) = promote(&root, &bundle, input.force)?;
    warnings.extend(kept_warnings(&kept));

    let report = BlueprintReport {
        schema_version: BLUEPRINT_SCHEMA_VERSION,
        mode: "blueprint".into(),
        ok: true,
        design_path,
        force: input.force,
        ai: input.ai,
        provider: provider_str,
        enriched,
        written,
        kept,
        task_count,
        validate_ok: validation_report.ok,
        warnings,
        validation: Some(validation_to_json(&validation_report)),
    };

    let human = format_human(&report);
    let data = report_to_json(&report);
    Ok((human, data))
}

pub fn format_human(r: &BlueprintReport) -> String {
    let status = if r.ok { "ok" } else { "failed" };
    let mut out = format!(
        "blueprint: {status}\n\
         designPath: {}\n\
         taskCount: {}\n\
         written: {}\n\
         kept: {}\n\
         validateOk: {}\n\
         force: {}\n\
         ai: {}\n",
        r.design_path,
        r.task_count,
        r.written.len(),
        r.kept.len(),
        r.validate_ok,
        r.force,
        r.ai
    );
    if let Some(ref provider) = r.provider {
        out.push_str(&format!("provider: {provider}\n"));
    }
    out.push_str(&format!("enriched: {}\nmode: {}", r.enriched, r.mode));
    out
}

pub fn report_to_json(r: &BlueprintReport) -> Value {
    json!(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::fs::read_to_string;
    use std::path::PathBuf;

    fn test_root() -> (tempfile::TempDir, ProjectRoot) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("dare.config.json"), "{}").unwrap();
        std::fs::create_dir_all(dir.path().join("DARE")).unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        (dir, root)
    }

    fn sample_bundle() -> GeneratedBundle {
        let design = load_fixture("sample-design.md");
        let title = super::parse_design_title(&design);
        generate_bundle(&design, &title).unwrap()
    }

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/blueprint")
            .join(name)
    }

    fn load_fixture(name: &str) -> String {
        std::fs::read_to_string(fixture_path(name))
            .unwrap_or_else(|e| panic!("fixture {name}: {e}"))
    }

    #[test]
    fn parse_design_title() {
        let design = load_fixture("sample-design.md");
        assert_eq!(super::parse_design_title(&design), "Sample API Service");

        let custom = "# DESIGN: My Feature\n\nbody";
        assert_eq!(super::parse_design_title(custom), "My Feature");

        let h1 = "# Plain H1 Title\n";
        assert_eq!(super::parse_design_title(h1), "Plain H1 Title");

        assert_eq!(super::parse_design_title(""), "Untitled");
    }

    #[test]
    fn extract_must_requirements_stable() {
        let design = load_fixture("sample-design.md");
        let a = extract_must_requirements(&design);
        let b = extract_must_requirements(&design);
        assert_eq!(a, b);
        assert_eq!(a.len(), 2);
        assert_eq!(a[0].0, "RF-01");
        assert_eq!(a[1].0, "RF-02");
        assert!(!a[0].1.is_empty());
    }

    #[test]
    fn generate_bundle_has_managed_markers() {
        let design = load_fixture("sample-design.md");
        let title = super::parse_design_title(&design);
        let bundle = generate_bundle(&design, &title).unwrap();

        assert!(bundle.blueprint_md.starts_with(MANAGED_MD));
        assert!(bundle.tasks_md.starts_with(MANAGED_MD));
        assert!(bundle.dag_yaml.starts_with(MANAGED_YAML));

        for id in BP_ENRICHABLE {
            assert!(
                bundle.blueprint_md.contains(&marker_begin(id)),
                "missing BEGIN for {id}"
            );
            assert!(
                bundle.blueprint_md.contains(&marker_end(id)),
                "missing END for {id}"
            );
        }

        for spec in bundle.specs.values() {
            assert!(spec.starts_with(MANAGED_MD));
        }
    }

    #[test]
    fn generate_bundle_rank0_at_least_2() {
        let design = load_fixture("sample-design.md");
        let title = super::parse_design_title(&design);
        let bundle = generate_bundle(&design, &title).unwrap();

        assert!(bundle.task_ids.contains(&"task-001".to_string()));
        assert!(bundle.task_ids.contains(&"task-002".to_string()));

        let yaml = &bundle.dag_yaml;
        assert!(yaml.contains("id: task-001"));
        assert!(yaml.contains("id: task-002"));
        assert!(yaml.contains("depends_on: []"));

        let rank0_count = yaml.matches("depends_on: []").count();
        assert!(
            rank0_count >= 2,
            "expected at least 2 rank-0 tasks, got {rank0_count}"
        );
    }

    #[test]
    fn is_managed_detects_marker() {
        let md = format!("{MANAGED_MD}\n\n# Title\n");
        assert!(is_managed_markdown(&md));

        let md_late = format!("# Title\n\n{MANAGED_MD}\n");
        assert!(!is_managed_markdown(&md_late));

        let yaml = format!("{MANAGED_YAML}\ntitle: test\n");
        assert!(is_managed_yaml(&yaml));

        let yaml_late = "title: test\n# dare:managed\n";
        assert!(!is_managed_yaml(yaml_late));
    }

    #[test]
    fn promote_keeps_unmanaged_without_force() {
        let (_dir, root) = test_root();
        let bundle = sample_bundle();
        let custom = "# Custom blueprint kept by user\n\nStakeholder notes.\n";
        atomic_write(
            &root,
            &SafeRelativePath::new(OUT_BLUEPRINT).unwrap(),
            custom.as_bytes(),
        )
        .unwrap();

        let (written, kept) = promote(&root, &bundle, false).unwrap();

        assert!(kept.contains(&OUT_BLUEPRINT.to_string()));
        assert!(!written.contains(&OUT_BLUEPRINT.to_string()));
        let rel = SafeRelativePath::new(OUT_BLUEPRINT).unwrap();
        let after = read_to_string(&root, &rel).unwrap();
        assert_eq!(after, custom);
    }

    #[test]
    fn promote_overwrites_managed_without_force() {
        let (_dir, root) = test_root();
        let bundle = sample_bundle();
        let old = format!("{MANAGED_MD}\n\n# Old managed blueprint\n");
        atomic_write(
            &root,
            &SafeRelativePath::new(OUT_BLUEPRINT).unwrap(),
            old.as_bytes(),
        )
        .unwrap();

        let (written, kept) = promote(&root, &bundle, false).unwrap();

        assert!(written.contains(&OUT_BLUEPRINT.to_string()));
        assert!(!kept.contains(&OUT_BLUEPRINT.to_string()));
        let rel = SafeRelativePath::new(OUT_BLUEPRINT).unwrap();
        let after = read_to_string(&root, &rel).unwrap();
        assert_eq!(after, bundle.blueprint_md);
    }

    #[test]
    fn promote_force_overwrites_unmanaged() {
        let (_dir, root) = test_root();
        let bundle = sample_bundle();
        let custom = "# Custom TASKS table\n| x | y |\n";
        atomic_write(
            &root,
            &SafeRelativePath::new(OUT_TASKS).unwrap(),
            custom.as_bytes(),
        )
        .unwrap();

        let (written, kept) = promote(&root, &bundle, true).unwrap();

        assert!(written.contains(&OUT_TASKS.to_string()));
        assert!(!kept.contains(&OUT_TASKS.to_string()));
        let rel = SafeRelativePath::new(OUT_TASKS).unwrap();
        let after = read_to_string(&root, &rel).unwrap();
        assert_eq!(after, bundle.tasks_md);
    }

    #[test]
    fn validate_rejects_bad_bundle() {
        use dare_core::ErrorKind;

        let (_dir, root) = test_root();
        let mut bundle = sample_bundle();
        bundle.dag_yaml = bundle.dag_yaml.replace("id: task-audit", "id: task-001");

        let err = stage_and_validate(&root, &bundle).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::Internal);
        assert!(err.message().contains("DAG validation failed"));

        let stage_rel = format!(".dare/blueprint-stage-{}", std::process::id());
        let stage_abs = root
            .resolve(&SafeRelativePath::new(&stage_rel).unwrap())
            .unwrap();
        assert!(!stage_abs.as_path().as_std_path().exists());

        let dag_rel = SafeRelativePath::new(OUT_DAG).unwrap();
        assert!(!root
            .resolve(&dag_rel)
            .unwrap()
            .as_path()
            .as_std_path()
            .exists());
    }

    #[test]
    fn report_schema_version_1() {
        let report = BlueprintReport {
            schema_version: BLUEPRINT_SCHEMA_VERSION,
            mode: "blueprint".into(),
            ok: true,
            design_path: DEFAULT_DESIGN_REL.into(),
            force: false,
            ai: false,
            provider: None,
            enriched: false,
            written: vec![OUT_BLUEPRINT.into()],
            kept: vec![],
            task_count: 5,
            validate_ok: true,
            warnings: vec![],
            validation: None,
        };
        let v = report_to_json(&report);
        assert_eq!(v["schemaVersion"], 1);
        assert_eq!(v["mode"], "blueprint");
        assert_eq!(v["ok"], true);
        assert_eq!(v["taskCount"], 5);
        assert_eq!(v["validateOk"], true);
        assert!(v["provider"].is_null());
        assert!(v["validation"].is_null());

        let human = format_human(&report);
        assert!(human.contains("blueprint: ok"));
        assert!(human.contains("taskCount: 5"));
        assert!(human.contains("mode: blueprint"));
    }
}
