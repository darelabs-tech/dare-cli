//! Skills-pacote registry: model, registries, and dependency resolution (microplano 044).

mod model;
mod registry;

pub use model::{
    classify_skill, validate_skill_id, validate_version_segment, RegistrySkill, SkillKind,
    SkillManifest, SkillSource, GENERIC_SKILL_IDS,
};
pub use registry::{
    is_generic_skill, load_project_skills, resolve_dependencies, CompositeRegistry, FailingHttpGet,
    HttpGet, LocalRegistry, MockRegistry, RemoteRegistry, UreqHttpGet, ENV_LOCAL_REGISTRY,
    ENV_REMOTE_REGISTRY, PROJECT_SKILLS_REL, REMOTE_BASE_URL_DEFAULT, REMOTE_TIMEOUT_SECS,
};
