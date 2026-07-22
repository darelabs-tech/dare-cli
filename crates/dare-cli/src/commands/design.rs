//! `dare design` — render determinístico de DARE/DESIGN.md (microplano 023).

use std::collections::HashMap;
use std::io::{self, IsTerminal, Write};

use dare_ai::{
    inject_enrichable, parse_and_validate_sections, resolve_provider, EnrichRequest, ProviderId,
};
use dare_core::fs::{atomic_write, read_to_string};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use dare_project::find_project_root;
use serde::Serialize;
use serde_json::{json, Value};

pub const DESIGN_REL: &str = "DARE/DESIGN.md";
pub const DESIGN_SCHEMA_VERSION: u32 = 2;
pub const DESC_MAX: usize = 32_768;
pub const DESIGN_READ_CAP: usize = 262_144;
pub const MARKER_BEGIN: &str = "<!-- AGENT:BEGIN section=\"";
pub const MARKER_END_PREFIX: &str = "<!-- AGENT:END section=\"";
pub const ENRICHABLE: &[&str] = &[
    "description",
    "objectives",
    "functional-requirements",
    "stack",
];

#[derive(Debug, Clone)]
pub struct DesignInput {
    pub title: String,
    pub description: String,
    pub interactive: bool,
    /// Override header date (tests use `Some("1970-01-01")`).
    pub fixed_date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignReport {
    pub schema_version: u32,
    pub mode: String,
    pub ok: bool,
    pub path: String,
    pub action: String,
    pub title: String,
    pub marker_count: u32,
    pub preserved_regions: u32,
    pub interactive: bool,
    pub warnings: Vec<String>,
    pub ai: bool,
    pub provider: Option<String>,
    pub enriched: bool,
}

fn marker_begin(section: &str) -> String {
    format!("{MARKER_BEGIN}{section}\" -->")
}

fn marker_end(section: &str) -> String {
    format!("{MARKER_END_PREFIX}{section}\" -->")
}

fn resolve_date(input: &DesignInput) -> String {
    if let Some(ref d) = input.fixed_date {
        return d.clone();
    }
    utc_today_yyyy_mm_dd()
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

pub fn validate_description(desc: &str) -> CoreResult<()> {
    if desc.trim().is_empty() {
        return Err(CoreError::invalid_input("description must not be empty"));
    }
    if desc.len() > DESC_MAX {
        return Err(CoreError::invalid_input(format!(
            "description exceeds maximum length of {DESC_MAX} bytes"
        )));
    }
    Ok(())
}

pub fn derive_title(desc: &str) -> String {
    let trimmed = desc.trim();
    if trimmed.is_empty() {
        return "Untitled".into();
    }
    trimmed.chars().take(60).collect()
}

pub fn render_canonical(input: &DesignInput) -> String {
    let date = resolve_date(input);
    let mut out = String::new();

    out.push_str(&format!("# DESIGN: {}\n\n", input.title));
    out.push_str(&format!(
        "> **Versão:** v1.0 | **Data:** {date} | **Status:** DRAFT\n\n"
    ));
    out.push_str("---\n\n");

    out.push_str("## 1. DESCRIÇÃO\n\n");
    out.push_str(&marker_begin("description"));
    out.push('\n');
    out.push_str(&input.description);
    out.push('\n');
    out.push_str(&marker_end("description"));
    out.push_str("\n\n---\n\n");

    out.push_str("## 2. OBJETIVOS E MÉTRICAS DE SUCESSO\n\n");
    out.push_str(&marker_begin("objectives"));
    out.push_str(
        "\n| # | Objetivo | Métrica verificável | Meta |\n\
         |---|----------|---------------------|------|\n\
         | O-01 | [A definir] | | |\n",
    );
    out.push_str(&marker_end("objectives"));
    out.push_str("\n\n---\n\n");

    out.push_str("## 3. STAKEHOLDERS\n\n");
    out.push_str(
        "| Papel | Nome / Time | Interesse principal |\n\
         |-------|-------------|---------------------|\n\
         | Product Owner | [A definir] | Aprovação de scope e prioridades |\n\
         | Tech Lead | [A definir] | Decisões arquiteturais |\n\
         | Usuário Final | [A definir] | [Persona] — [necessidade] |\n\
         | Operações / SRE | [A definir] | SLA, alertas, deploys |\n\n\
         ---\n\n",
    );

    out.push_str("## 4. REQUISITOS FUNCIONAIS\n\n");
    out.push_str(&marker_begin("functional-requirements"));
    out.push_str(
        "\n| ID | Requisito | Prioridade | Critério de aceite |\n\
         |----|-----------|------------|--------------------|\n\
         | RF-01 | [A definir] | MUST | |\n\n\
         > Prioridades: **MUST** (bloqueia v1) · **SHOULD** (importante, mas não bloqueia) · **COULD** (nice to have)\n",
    );
    out.push_str(&marker_end("functional-requirements"));
    out.push_str("\n\n---\n\n");

    out.push_str("## 5. REQUISITOS NÃO-FUNCIONAIS\n\n");
    out.push_str(
        "| ID | Categoria | Requisito | Meta |\n\
         |----|-----------|-----------|------|\n\
         | RNF-01 | Performance | [A definir] | |\n\
         | RNF-02 | Disponibilidade | [A definir] | |\n\
         | RNF-03 | Segurança | [A definir] | |\n\n\
         ---\n\n",
    );

    out.push_str("## 6. REQUISITOS DE SEGURANÇA\n\n");
    out.push_str(
        "| ID | Requisito | Referência |\n\
         |----|-----------|------------|\n\
         | RS-01 | [A definir] | OWASP A03 |\n\
         | RS-02 | [A definir] | OWASP A02 |\n\
         | RS-03 | [A definir] | OWASP A01 |\n\
         | RS-04 | [A definir] | OWASP A06 |\n\
         | RS-05 | [A definir] | Supply chain |\n\n\
         ---\n\n",
    );

    out.push_str("## 7. STACK TÉCNICA\n\n");
    out.push_str(&marker_begin("stack"));
    out.push_str(
        "\n| Camada | Tecnologia | Versão |\n\
         |--------|-----------|--------|\n\
         | Linguagem / Runtime | [A definir] | |\n\
         | Framework principal | [A definir] | |\n\
         | Banco de dados | [A definir] | |\n\
         | Cache | [A definir] | |\n\
         | Frontend | [A definir] | |\n\
         | Infra / deploy | [A definir] | |\n\
         | Observabilidade | [A definir] | |\n",
    );
    out.push_str(&marker_end("stack"));
    out.push_str("\n\n---\n\n");

    out.push_str("## 8. INTEGRAÇÕES EXTERNAS\n\n");
    out.push_str(
        "| Sistema | Tipo | Protocolo | Direção | Dados trocados | Responsável |\n\
         |---------|------|-----------|---------|----------------|-------------|\n\
         | [A definir] | | | | | |\n\n\
         ---\n\n",
    );

    out.push_str("## 9. RESTRIÇÕES\n\n");
    out.push_str(
        "- **Prazo:** [A definir]\n\
         - **Orçamento de infra:** [A definir]\n\
         - **Limitações técnicas:** [A definir]\n\
         - **Regulatórias / Compliance:** [A definir]\n\n\
         ---\n\n",
    );

    out.push_str("## 10. FORA DO ESCOPO (v1)\n\n");
    out.push_str("- [A definir]\n\n---\n\n");

    out.push_str("## 11. RISCOS E MITIGAÇÕES\n\n");
    out.push_str(
        "| # | Risco | Probabilidade | Impacto | Mitigação |\n\
         |---|-------|---------------|---------|-----------|\n\
         | R-01 | [A definir] | | | |\n\n\
         ---\n\n",
    );

    out.push_str("## 12. CHECKLIST DE APROVAÇÃO\n\n");
    out.push_str(
        "- [ ] Requisitos funcionais revisados e priorizados\n\
         - [ ] Requisitos de segurança validados pelo Tech Lead\n\
         - [ ] Stack técnica aprovada\n\
         - [ ] Integrações externas confirmadas com responsáveis\n\
         - [ ] Fora do escopo alinhado com Product Owner\n\
         - [ ] Riscos críticos com mitigação definida\n",
    );

    out
}

pub fn format_human(r: &DesignReport) -> String {
    let status = if r.ok { "ok" } else { "failed" };
    let mut out = format!(
        "design: {status}\n\
         path: {}\n\
         action: {}\n\
         title: {}\n\
         markerCount: {}\n\
         preservedRegions: {}\n\
         ai: {}\n",
        r.path, r.action, r.title, r.marker_count, r.preserved_regions, r.ai
    );
    if let Some(ref provider) = r.provider {
        out.push_str(&format!("provider: {provider}\n"));
    }
    out.push_str(&format!("enriched: {}\nmode: {}", r.enriched, r.mode));
    out
}

pub fn report_to_json(r: &DesignReport) -> Value {
    json!(r)
}

const PRESERVED_MARKER: &str = "<!-- dare:preserved -->";
const APPENDIX_HEADING: &str = "## APPENDIX — Preserved previous content";

fn section_heading(section_id: &str) -> Option<&'static str> {
    match section_id {
        "description" => Some("## 1. DESCRIÇÃO"),
        "objectives" => Some("## 2. OBJETIVOS E MÉTRICAS DE SUCESSO"),
        "functional-requirements" => Some("## 4. REQUISITOS FUNCIONAIS"),
        "stack" => Some("## 7. STACK TÉCNICA"),
        _ => None,
    }
}

fn parse_section_id(line: &str, kind: &str) -> Option<String> {
    let needle = format!("AGENT:{kind} section=\"");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn validate_agent_markers(content: &str) -> CoreResult<()> {
    let mut open: Option<String> = None;
    for line in content.lines() {
        if line.contains("<!-- AGENT:BEGIN") {
            if open.is_some() {
                return Err(CoreError::invalid_input(
                    "malformed AGENT markers in DARE/DESIGN.md",
                ));
            }
            open = parse_section_id(line, "BEGIN");
        } else if line.contains("<!-- AGENT:END") {
            match open.take() {
                Some(_) => {}
                None => {
                    return Err(CoreError::invalid_input(
                        "malformed AGENT markers in DARE/DESIGN.md",
                    ));
                }
            }
        }
    }
    if open.is_some() {
        return Err(CoreError::invalid_input(
            "malformed AGENT markers in DARE/DESIGN.md",
        ));
    }
    Ok(())
}

fn parse_enrichable_blocks(content: &str) -> CoreResult<HashMap<String, String>> {
    let mut blocks = HashMap::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.contains("<!-- AGENT:BEGIN") {
            if let Some(id) = parse_section_id(line, "BEGIN") {
                if ENRICHABLE.contains(&id.as_str()) {
                    let begin = i;
                    i += 1;
                    while i < lines.len() && !lines[i].contains("<!-- AGENT:END") {
                        i += 1;
                    }
                    if i >= lines.len() {
                        return Err(CoreError::invalid_input(
                            "malformed AGENT markers in DARE/DESIGN.md",
                        ));
                    }
                    let end_line = lines[i];
                    if parse_section_id(end_line, "END").as_deref() != Some(id.as_str()) {
                        return Err(CoreError::invalid_input(
                            "malformed AGENT markers in DARE/DESIGN.md",
                        ));
                    }
                    let block = lines[begin..=i].join("\n");
                    blocks.insert(id, block);
                }
            }
        }
        i += 1;
    }
    Ok(blocks)
}

fn find_block_range(content: &str, section_id: &str) -> Option<(usize, usize)> {
    let begin = marker_begin(section_id);
    let end = marker_end(section_id);
    let start = content.find(&begin)?;
    let tail = &content[start..];
    let end_rel = tail.find(&end)?;
    Some((start, start + end_rel + end.len()))
}

fn insert_block_after_heading(content: &str, section_id: &str, block: &str) -> String {
    let heading = match section_heading(section_id) {
        Some(h) => h,
        None => return format!("{content}\n\n{block}\n"),
    };
    if let Some(pos) = content.find(heading) {
        let after_heading = pos + heading.len();
        let mut out = String::with_capacity(content.len() + block.len() + 4);
        out.push_str(&content[..after_heading]);
        out.push_str("\n\n");
        out.push_str(block);
        out.push_str(&content[after_heading..]);
        return out;
    }
    format!("{content}\n\n{block}\n")
}

fn replace_or_insert_block(content: &str, section_id: &str, block: &str) -> String {
    if let Some((start, end)) = find_block_range(content, section_id) {
        let mut out = String::with_capacity(content.len() + block.len());
        out.push_str(&content[..start]);
        out.push_str(block);
        out.push_str(&content[end..]);
        out
    } else {
        insert_block_after_heading(content, section_id, block)
    }
}

fn count_preserved_regions(content: &str) -> u32 {
    content.matches(PRESERVED_MARKER).count() as u32
}

fn existing_has_agent_markers(existing: &str) -> bool {
    existing.contains("AGENT:BEGIN")
}

pub fn merge_preserve(existing: &str, fresh: &str) -> CoreResult<String> {
    let fresh_blocks = parse_enrichable_blocks(fresh)?;

    if !existing_has_agent_markers(existing) {
        let mut out = fresh.to_string();
        if !existing.trim().is_empty() {
            out.push_str("\n\n");
            out.push_str(APPENDIX_HEADING);
            out.push_str("\n\n");
            out.push_str(PRESERVED_MARKER);
            out.push('\n');
            out.push_str(existing);
        }
        return Ok(out);
    }

    validate_agent_markers(existing)?;

    let mut merged = existing.to_string();
    for id in ENRICHABLE {
        if let Some(block) = fresh_blocks.get(*id) {
            merged = replace_or_insert_block(&merged, id, block);
        }
    }
    Ok(merged)
}

fn ensure_dare_dir(root: &ProjectRoot) -> CoreResult<()> {
    let rel = SafeRelativePath::new("DARE")?;
    let abs = root.resolve(&rel)?;
    std::fs::create_dir_all(abs.as_path().as_std_path()).map_err(|e| CoreError::io(e.to_string()))
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
    read_to_string(root, rel)
}

fn design_file_exists(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<bool> {
    let abs = root.resolve(rel)?;
    Ok(abs.as_path().exists())
}

/// Returns whether stdin is attached to a TTY (interactive gate).
pub fn is_stdin_tty() -> bool {
    io::stdin().is_terminal()
}

fn prompt_line(label: &str) -> CoreResult<String> {
    print!("{label}");
    io::stdout()
        .flush()
        .map_err(|e| CoreError::io(e.to_string()))?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| CoreError::io(e.to_string()))?;
    Ok(line.trim_end_matches(['\r', '\n']).to_string())
}

fn read_interactive_input() -> CoreResult<(String, String)> {
    let title_raw = prompt_line("Title (empty = derive from description): ")?;
    let description = prompt_line("Description: ")?;
    validate_description(&description)?;
    let title = if title_raw.trim().is_empty() {
        derive_title(&description)
    } else {
        title_raw
    };
    Ok((title, description))
}

/// CLI entry: resolve project root, optional interactive prompts, apply design.
pub fn run_design(
    description: Option<String>,
    interactive: bool,
    ai: bool,
    provider: Option<String>,
) -> CoreResult<(String, Value)> {
    if provider.is_some() && !ai {
        return Err(CoreError::usage("--provider requires --ai"));
    }

    if interactive {
        if description.is_some() {
            return Err(CoreError::usage(
                "cannot combine --interactive with description",
            ));
        }
        if !is_stdin_tty() {
            return Err(CoreError::usage("design --interactive requires a TTY"));
        }
    } else if description.is_none() {
        return Err(CoreError::usage("description required (or --interactive)"));
    }

    let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
    let Some(root_path) = find_project_root(&cwd) else {
        return Err(CoreError::invalid_input("project root not found"));
    };
    let root = ProjectRoot::new(&root_path)?;

    let (title, desc) = if interactive {
        read_interactive_input()?
    } else {
        let desc = description.unwrap_or_default();
        validate_description(&desc)?;
        (derive_title(&desc), desc)
    };

    let input = DesignInput {
        title,
        description: desc,
        interactive,
        fixed_date: None,
    };
    let base = apply_design(&root, &input)?;
    let report = if ai {
        enrich_design(&root, &input, base, provider)?
    } else {
        report_v2(base, false, None, false)
    };
    let human = format_human(&report);
    let data = report_to_json(&report);
    Ok((human, data))
}

fn report_v2(
    mut base: DesignReport,
    ai: bool,
    provider: Option<String>,
    enriched: bool,
) -> DesignReport {
    base.schema_version = DESIGN_SCHEMA_VERSION;
    base.ai = ai;
    base.provider = provider;
    base.enriched = enriched;
    base
}

fn enrich_design(
    root: &ProjectRoot,
    input: &DesignInput,
    base: DesignReport,
    provider: Option<String>,
) -> CoreResult<DesignReport> {
    let pid = match provider.as_deref() {
        None => ProviderId::Codex,
        Some(s) => ProviderId::parse(s)?,
    };
    let prov = resolve_provider(pid)?;
    let rel = SafeRelativePath::new(DESIGN_REL)?;
    let md = read_design_capped(root, &rel)?;
    let cwd_rel = SafeRelativePath::new("DARE")?;
    let req = EnrichRequest {
        command: "design".into(),
        title: base.title.clone(),
        description: input.description.clone(),
        current_markdown: md.clone(),
        cwd: Some((root.clone(), cwd_rel)),
    };
    let raw = prov.enrich(&req)?;
    let sections = parse_and_validate_sections(&raw.stdout)?;
    let injected = inject_enrichable(&md, &sections)?;
    atomic_write(root, &rel, injected.as_bytes())?;
    Ok(report_v2(base, true, Some(pid.as_str().to_string()), true))
}

pub fn apply_design(root: &ProjectRoot, input: &DesignInput) -> CoreResult<DesignReport> {
    validate_description(&input.description)?;
    ensure_dare_dir(root)?;

    let rel = SafeRelativePath::new(DESIGN_REL)?;
    let fresh = render_canonical(input);
    let title = input.title.clone();

    let (action, content, preserved_regions) = if design_file_exists(root, &rel)? {
        match read_design_capped(root, &rel) {
            Ok(existing) if existing.trim().is_empty() => ("created".to_string(), fresh, 0),
            Ok(existing) => {
                let merged = merge_preserve(&existing, &fresh)?;
                let preserved = count_preserved_regions(&merged);
                ("updated".to_string(), merged, preserved)
            }
            Err(e) if e.kind() == dare_core::ErrorKind::NotFound => {
                ("created".to_string(), fresh, 0)
            }
            Err(e) => return Err(e),
        }
    } else {
        ("created".to_string(), fresh, 0)
    };

    atomic_write(root, &rel, content.as_bytes())?;

    Ok(DesignReport {
        schema_version: DESIGN_SCHEMA_VERSION,
        mode: "design".into(),
        ok: true,
        path: DESIGN_REL.into(),
        action,
        title,
        marker_count: 4,
        preserved_regions,
        interactive: input.interactive,
        warnings: vec![],
        ai: false,
        provider: None,
        enriched: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::ErrorKind;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/design")
            .join(name)
    }

    fn load_fixture(name: &str) -> String {
        std::fs::read_to_string(fixture_path(name))
            .unwrap_or_else(|e| panic!("fixture {name}: {e}"))
    }

    fn sample_input(desc: &str) -> DesignInput {
        DesignInput {
            title: derive_title(desc),
            description: desc.into(),
            interactive: false,
            fixed_date: Some("1970-01-01".into()),
        }
    }

    fn sample_input_from_fixture() -> DesignInput {
        let desc = load_fixture("input-basic.txt");
        sample_input(desc.trim())
    }

    #[test]
    fn validate_description_rejects_empty() {
        assert!(validate_description("").is_err());
        assert!(validate_description("   \n\t").is_err());
        let err = validate_description("").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn validate_description_rejects_oversize() {
        let big = "x".repeat(DESC_MAX + 1);
        let err = validate_description(&big).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn derive_title_truncates_60() {
        let long = "a".repeat(100);
        let title = derive_title(&long);
        assert!(title.len() <= 60);
        assert_eq!(title.chars().count(), 60);
    }

    #[test]
    fn derive_title_empty_is_untitled() {
        assert_eq!(derive_title(""), "Untitled");
        assert_eq!(derive_title("   "), "Untitled");
    }

    #[test]
    fn render_contains_four_enrichable_markers() {
        let md = render_canonical(&sample_input("API de pagamentos"));
        for id in ENRICHABLE {
            assert!(
                md.contains(&marker_begin(id)),
                "missing BEGIN marker for {id}"
            );
            assert!(md.contains(&marker_end(id)), "missing END marker for {id}");
        }
        assert_eq!(md.matches(MARKER_BEGIN).count(), 4);
        assert_eq!(md.matches(MARKER_END_PREFIX).count(), 4);
        assert!(md.contains("## 12. CHECKLIST DE APROVAÇÃO"));
        assert!(!md.is_empty());
    }

    #[test]
    fn render_stable_with_fixed_date() {
        let input = sample_input("Stable output check");
        let a = render_canonical(&input);
        let b = render_canonical(&input);
        assert_eq!(a, b);
        assert!(a.contains("**Data:** 1970-01-01"));
    }

    #[test]
    fn report_schema_version_2() {
        let report = DesignReport {
            schema_version: DESIGN_SCHEMA_VERSION,
            mode: "design".into(),
            ok: true,
            path: DESIGN_REL.into(),
            action: "created".into(),
            title: "My API".into(),
            marker_count: 4,
            preserved_regions: 0,
            interactive: false,
            warnings: vec![],
            ai: false,
            provider: None,
            enriched: false,
        };
        let v = report_to_json(&report);
        assert_eq!(v["schemaVersion"], 2);
        assert_eq!(v["mode"], "design");
        assert_eq!(v["ok"], true);
        assert_eq!(v["markerCount"], 4);
        assert_eq!(v["ai"], false);
        assert_eq!(v["enriched"], false);
        assert!(v["provider"].is_null());
        let human = format_human(&report);
        assert!(human.contains("design: ok"));
        assert!(human.contains("ai: false"));
        assert!(human.contains("enriched: false"));
        assert!(human.contains("mode: design"));
    }

    #[test]
    fn design_ai_schema_fail_keeps_file() {
        std::env::set_var("DARE_AI_MOCK_MODE", "invalid-json");
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("dare.config.json"), "{}").unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let input = sample_input("Schema fail preserve write1");
        let base = apply_design(&root, &input).unwrap();
        let rel = SafeRelativePath::new(DESIGN_REL).unwrap();
        let write1 = read_to_string(&root, &rel).unwrap();
        assert!(write1.contains("[A definir]"));

        let err = enrich_design(&root, &input, base, Some("mock".into())).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);

        let after = read_to_string(&root, &rel).unwrap();
        assert_eq!(
            write1, after,
            "write1 must remain intact on enrich schema fail"
        );
        std::env::remove_var("DARE_AI_MOCK_MODE");
    }

    #[test]
    fn merge_preserve_keeps_unmanaged_paragraph() {
        let existing = load_fixture("existing-with-notes.md");
        let fresh = render_canonical(&sample_input_from_fixture());
        let merged = merge_preserve(&existing, &fresh).unwrap();
        assert!(
            merged.contains("User note outside any AGENT marker — must survive merge."),
            "unmanaged paragraph missing after merge"
        );
        assert!(merged.contains(&marker_begin("description")));
        assert!(merged.contains("API de pagamentos recorrentes"));
    }

    #[test]
    fn merge_first_existing_without_markers_appends_appendix() {
        let existing = "# Old design\n\nCustom stakeholder table kept by user.\n";
        let fresh = render_canonical(&sample_input("New feature"));
        let merged = merge_preserve(existing, &fresh).unwrap();
        assert!(merged.contains(APPENDIX_HEADING));
        assert!(merged.contains(PRESERVED_MARKER));
        assert!(merged.contains("Custom stakeholder table kept by user."));
        assert!(merged.starts_with("# DESIGN:"));
    }

    #[test]
    fn apply_design_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let input = sample_input_from_fixture();
        let report = apply_design(&root, &input).unwrap();
        assert!(report.ok);
        assert_eq!(report.action, "created");
        assert_eq!(report.path, DESIGN_REL);
        assert_eq!(report.marker_count, 4);
        assert_eq!(report.preserved_regions, 0);
        let rel = SafeRelativePath::new(DESIGN_REL).unwrap();
        let written = read_to_string(&root, &rel).unwrap();
        assert!(written.contains("<!-- AGENT:BEGIN"));
        assert_eq!(written.matches(MARKER_BEGIN).count(), 4);
    }

    #[test]
    fn golden_basic_structure() {
        let input = sample_input_from_fixture();
        let rendered = render_canonical(&input);
        let golden = load_fixture("golden-basic.md");
        assert_eq!(rendered, golden);
    }
}
