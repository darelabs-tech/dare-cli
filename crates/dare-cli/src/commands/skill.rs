//! `dare skill list|info|add|remove|update|publish` (microplanos 044–045).

use std::path::PathBuf;
use std::process::ExitCode;

use dare_core::{CoreError, CoreResult, ProjectRoot};
use dare_skills::{
    install_skill, load_project_skills, publish_skill, remove_skill, update_skill,
    CompositeRegistry, InstallOpts, RegistrySkill, SkillKind,
};
use serde_json::{json, Value};

use crate::output::OutputRenderer;

#[derive(Debug, Clone)]
pub enum SkillAction {
    List,
    Info {
        name: String,
    },
    Add {
        name: String,
        version: Option<String>,
        from: Option<PathBuf>,
    },
    Remove {
        name: String,
    },
    Update {
        name: String,
        from: Option<PathBuf>,
    },
    Publish {
        name: String,
        out: Option<PathBuf>,
    },
}

pub fn run_skill(action: SkillAction, renderer: &OutputRenderer<'_>) -> ExitCode {
    match run_skill_inner(action) {
        Ok((msg, data)) => {
            let _ = renderer.write_success(&msg, data);
            ExitCode::SUCCESS
        }
        Err(e) => ExitCode::from(renderer.write_error(&e) as u8),
    }
}

fn run_skill_inner(action: SkillAction) -> CoreResult<(String, Value)> {
    let cwd = std::env::current_dir().map_err(|e| CoreError::io(e.to_string()))?;
    let root = ProjectRoot::new(&cwd)?;
    let _project = load_project_skills(&root)?;
    let registry = CompositeRegistry::from_env();

    match action {
        SkillAction::List => {
            let skills = registry.list()?;
            let human = format_list_human(&skills);
            let data = json!({
                "action": "skill.list",
                "count": skills.len(),
                "skills": skills_to_json(&skills),
            });
            Ok((human, data))
        }
        SkillAction::Info { name } => {
            let skill = registry
                .get(&name)?
                .ok_or_else(|| CoreError::not_found(format!("skill not found: {name}")))?;
            let human = format_info_human(&skill);
            let data = json!({
                "action": "skill.info",
                "skill": skill_to_json(&skill),
            });
            Ok((human, data))
        }
        SkillAction::Add {
            name,
            version,
            from,
        } => {
            let opts = InstallOpts {
                version,
                from_archive: from,
            };
            let report = install_skill(&root, &name, &opts, &registry)?;
            let human = format!(
                "skill add: {}@{} → {}",
                report.name, report.version, report.path
            );
            let data = json!({
                "action": "skill.add",
                "name": report.name,
                "version": report.version,
                "path": report.path,
                "installed_deps": report.installed_deps,
            });
            Ok((human, data))
        }
        SkillAction::Remove { name } => {
            let report = remove_skill(&root, &name)?;
            let human = format!(
                "skill remove: {} (deleted {})",
                report.name, report.removed_path
            );
            let data = json!({
                "action": "skill.remove",
                "name": report.name,
                "removed_path": report.removed_path,
            });
            Ok((human, data))
        }
        SkillAction::Update { name, from } => {
            let opts = InstallOpts {
                version: None,
                from_archive: from,
            };
            let report = update_skill(&root, &name, &opts, &registry)?;
            let human = format!(
                "skill update: {}@{} → {}",
                report.name, report.version, report.path
            );
            let data = json!({
                "action": "skill.update",
                "name": report.name,
                "version": report.version,
                "path": report.path,
            });
            Ok((human, data))
        }
        SkillAction::Publish { name, out } => {
            let out_dir = out.unwrap_or_else(|| cwd.join("dist"));
            let report = publish_skill(&root, &name, &out_dir)?;
            let human = format!(
                "skill publish: {}@{} → {} (sha256={})",
                report.name, report.version, report.artifact, report.sha256
            );
            let data = json!({
                "action": "skill.publish",
                "name": report.name,
                "version": report.version,
                "artifact": report.artifact,
                "sha256": report.sha256,
                "signature": report.signature,
            });
            Ok((human, data))
        }
    }
}

fn skills_to_json(skills: &[RegistrySkill]) -> Value {
    Value::Array(skills.iter().map(skill_to_json).collect())
}

fn skill_to_json(s: &RegistrySkill) -> Value {
    json!({
        "name": s.name,
        "version": s.version,
        "description": s.description,
        "author": s.author,
        "license": s.license,
        "dare_version": s.dare_version,
        "depends_on": s.depends_on,
        "kind": kind_str(s.kind),
        "source": s.source.as_str(),
    })
}

fn kind_str(kind: SkillKind) -> &'static str {
    match kind {
        SkillKind::Generic => "generic",
        SkillKind::Stack => "stack",
    }
}

fn format_list_human(skills: &[RegistrySkill]) -> String {
    if skills.is_empty() {
        return "skill list: (empty)".to_string();
    }
    let mut lines = vec![format!("skill list: {} skill(s)", skills.len())];
    for s in skills {
        lines.push(format!(
            "  {}@{} [{}] ({})",
            s.name,
            s.version,
            kind_str(s.kind),
            s.source.as_str()
        ));
    }
    lines.join("\n")
}

fn format_info_human(s: &RegistrySkill) -> String {
    let deps = if s.depends_on.is_empty() {
        "(none)".to_string()
    } else {
        s.depends_on.join(", ")
    };
    format!(
        "skill info: {}@{}\n  kind: {}\n  source: {}\n  author: {}\n  license: {}\n  description: {}\n  depends_on: {}",
        s.name,
        s.version,
        kind_str(s.kind),
        s.source.as_str(),
        s.author,
        s.license,
        s.description,
        deps
    )
}
