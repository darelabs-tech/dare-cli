//! Skills-pacote registry + lifecycle (microplanos 044–045).

mod install;
mod model;
mod publish;
mod registry;

pub use install::{
    assert_safe_archive_entry, extract_archive_safe, install_skill, remove_skill, skill_rel,
    update_skill, write_tar_gz_from_dir, InstallOpts, InstallReport, RemoveReport,
    PACKAGES_SKILLS_REL,
};
pub use model::{
    classify_skill, validate_skill_id, validate_version_segment, RegistrySkill, SkillKind,
    SkillManifest, SkillSource, GENERIC_SKILL_IDS,
};
pub use publish::{
    load_installed_manifest, packages_skills_prefix, public_key_hex_from_private, publish_skill,
    sha256_file, sign_artifact, validate_for_publish, verify_artifact, PublishReport,
    ENV_SKILL_PRIVATE_KEY, REQUIRED_LICENSE, SIG_EXT,
};
pub use registry::{
    is_generic_skill, load_project_skills, resolve_dependencies, CompositeRegistry, FailingHttpGet,
    HttpGet, LocalRegistry, MockRegistry, RemoteRegistry, UreqHttpGet, ENV_LOCAL_REGISTRY,
    ENV_REMOTE_REGISTRY, PROJECT_SKILLS_REL, REMOTE_BASE_URL_DEFAULT, REMOTE_TIMEOUT_SECS,
};
