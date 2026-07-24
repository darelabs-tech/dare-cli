//! Atomic skill install / remove / update + safe archive extraction (microplano 045).

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

use dare_contracts::{load_skills_manifest, save_skills_manifest, SkillEntry, SkillsManifest};
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::Map;
use tar::Archive;

use crate::model::{validate_skill_id, RegistrySkill, SkillManifest, SkillSource};
use crate::registry::{resolve_dependencies, CompositeRegistry, LocalRegistry, PROJECT_SKILLS_REL};

pub const PACKAGES_SKILLS_REL: &str = "packages/skills";

#[derive(Debug, Clone, Default)]
pub struct InstallOpts {
    /// Optional version pin (registry lookup / materialize).
    pub version: Option<String>,
    /// Optional local archive (`.tar`, `.tar.gz`, `.tgz`, `.zip`).
    pub from_archive: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallReport {
    pub name: String,
    pub version: String,
    pub installed_deps: Vec<String>,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveReport {
    pub name: String,
    pub removed_path: String,
}

/// Install `name` (and missing deps) under `packages/skills/`.
pub fn install_skill(
    root: &ProjectRoot,
    name: &str,
    opts: &InstallOpts,
    registry: &CompositeRegistry,
) -> CoreResult<InstallReport> {
    validate_skill_id(name)?;
    if let Some(archive) = &opts.from_archive {
        return install_from_archive(root, name, archive, opts.version.as_deref());
    }

    let catalog = registry.list()?;
    let order = resolve_dependencies(&catalog, &[name.to_string()])?;
    let mut installed_deps = Vec::new();

    for dep in &order {
        if dep != name && skill_dir_exists(root, dep)? {
            continue;
        }
        if dep != name {
            materialize_from_registry(root, dep, None, registry)?;
            installed_deps.push(dep.clone());
        }
    }
    let version = materialize_from_registry(root, name, opts.version.as_deref(), registry)?;
    upsert_manifest_entry(root, name, &version)?;

    Ok(InstallReport {
        name: name.to_string(),
        version,
        installed_deps,
        path: format!("{PACKAGES_SKILLS_REL}/{name}"),
    })
}

/// Re-copy content from registry/archive and refresh manifest version.
pub fn update_skill(
    root: &ProjectRoot,
    name: &str,
    opts: &InstallOpts,
    registry: &CompositeRegistry,
) -> CoreResult<InstallReport> {
    validate_skill_id(name)?;
    if !skill_dir_exists(root, name)? && opts.from_archive.is_none() {
        return Err(CoreError::not_found(format!("skill not installed: {name}")));
    }
    // Force re-materialize (overwrite via staging).
    install_skill(root, name, opts, registry)
}

/// Remove installed skill directory + manifest entry; block if reverse deps.
pub fn remove_skill(root: &ProjectRoot, name: &str) -> CoreResult<RemoveReport> {
    validate_skill_id(name)?;
    if !skill_dir_exists(root, name)? {
        return Err(CoreError::not_found(format!("skill not installed: {name}")));
    }
    let dependents = find_reverse_dependents(root, name)?;
    if !dependents.is_empty() {
        return Err(CoreError::invalid_input(format!(
            "cannot remove {name}: required by {}",
            dependents.join(", ")
        )));
    }

    let rel = skill_rel(name)?;
    let abs = root.resolve(&rel)?;
    if abs.as_path().is_dir() {
        fs::remove_dir_all(abs.as_path()).map_err(|e| CoreError::io(e.to_string()))?;
    }
    remove_manifest_entry(root, name)?;

    Ok(RemoveReport {
        name: name.to_string(),
        removed_path: rel.as_str().to_string(),
    })
}

/// Extract tar/tar.gz/zip into `dest`, rejecting path traversal.
pub fn extract_archive_safe(archive: &Path, dest: &Path) -> CoreResult<()> {
    let name = archive
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    fs::create_dir_all(dest).map_err(|e| CoreError::io(e.to_string()))?;
    if name.ends_with(".zip") {
        extract_zip_safe(archive, dest)
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        let file = File::open(archive).map_err(|e| CoreError::io(e.to_string()))?;
        let dec = GzDecoder::new(file);
        extract_tar_safe(dec, dest)
    } else if name.ends_with(".tar") {
        let file = File::open(archive).map_err(|e| CoreError::io(e.to_string()))?;
        extract_tar_safe(file, dest)
    } else {
        Err(CoreError::invalid_input(
            "unsupported archive format (use .tar, .tar.gz, .tgz, or .zip)",
        ))
    }
}

/// Validate a single archive entry path (shared by tar/zip).
pub fn assert_safe_archive_entry(entry_name: &str) -> CoreResult<()> {
    if entry_name.is_empty() || entry_name.contains('\0') {
        return Err(CoreError::invalid_input("unsafe archive entry path"));
    }
    // Normalize separators for Component parsing.
    let normalized = entry_name.replace('\\', "/");
    if normalized.starts_with('/') || normalized.starts_with("~/") {
        return Err(CoreError::invalid_input("unsafe archive entry path"));
    }
    // Windows drive / UNC style
    if normalized.len() >= 2 && normalized.as_bytes()[1] == b':' {
        return Err(CoreError::invalid_input("unsafe archive entry path"));
    }
    let path = Path::new(&normalized);
    if path.is_absolute() {
        return Err(CoreError::invalid_input("unsafe archive entry path"));
    }
    for c in path.components() {
        match c {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(CoreError::invalid_input("unsafe archive entry path"));
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    Ok(())
}

fn install_from_archive(
    root: &ProjectRoot,
    name: &str,
    archive: &Path,
    version_hint: Option<&str>,
) -> CoreResult<InstallReport> {
    validate_skill_id(name)?;
    let staging_rel = staging_rel(name)?;
    let staging_abs = root.resolve(&staging_rel)?;
    if staging_abs.as_path().as_std_path().exists() {
        fs::remove_dir_all(staging_abs.as_path().as_std_path())
            .map_err(|e| CoreError::io(e.to_string()))?;
    }
    extract_archive_safe(archive, staging_abs.as_path().as_std_path())?;

    // If archive extracted a single top-level dir matching name, flatten; else expect skill.yml at root.
    let content_root = resolve_content_root(staging_abs.as_path().as_std_path(), name)?;
    let manifest = read_skill_yml(content_root)?;
    if manifest.name != name {
        return Err(CoreError::invalid_input(format!(
            "archive skill name mismatch: expected {name}, got {}",
            manifest.name
        )));
    }
    let version = version_hint
        .map(str::to_string)
        .unwrap_or_else(|| manifest.version.clone());

    // Promote content_root into final packages/skills/<name> via staging rename.
    promote_staging(root, name, content_root)?;
    upsert_manifest_entry(root, name, &version)?;

    Ok(InstallReport {
        name: name.to_string(),
        version,
        installed_deps: Vec::new(),
        path: format!("{PACKAGES_SKILLS_REL}/{name}"),
    })
}

fn materialize_from_registry(
    root: &ProjectRoot,
    name: &str,
    version: Option<&str>,
    registry: &CompositeRegistry,
) -> CoreResult<String> {
    let skill = registry
        .get(name)?
        .ok_or_else(|| CoreError::not_found(format!("skill not found: {name}")))?;
    let ver = version.unwrap_or(skill.version.as_str());
    validate_skill_id(ver).map_err(|_| CoreError::invalid_input("invalid skill version"))?;

    let staging_rel = staging_rel(name)?;
    let staging_abs = root.resolve(&staging_rel)?;
    if staging_abs.as_path().as_std_path().exists() {
        fs::remove_dir_all(staging_abs.as_path().as_std_path())
            .map_err(|e| CoreError::io(e.to_string()))?;
    }
    fs::create_dir_all(staging_abs.as_path().as_std_path())
        .map_err(|e| CoreError::io(e.to_string()))?;

    if skill.source == SkillSource::Local {
        if let Some(local_root) = local_content_dir(&skill, ver) {
            copy_dir_safe(&local_root, staging_abs.as_path().as_std_path())?;
        } else {
            write_synthesized_package(staging_abs.as_path().as_std_path(), &skill, ver)?;
        }
    } else {
        write_synthesized_package(staging_abs.as_path().as_std_path(), &skill, ver)?;
    }

    promote_staging(root, name, staging_abs.as_path().as_std_path())?;
    Ok(ver.to_string())
}

fn local_content_dir(skill: &RegistrySkill, version: &str) -> Option<PathBuf> {
    let reg = LocalRegistry::from_env()?;
    let dir = reg.root().join(&skill.name).join(version);
    if dir.is_dir() && dir.join("skill.yml").is_file() {
        Some(dir)
    } else {
        None
    }
}

fn write_synthesized_package(dest: &Path, skill: &RegistrySkill, version: &str) -> CoreResult<()> {
    let manifest = SkillManifest {
        name: skill.name.clone(),
        version: version.to_string(),
        description: skill.description.clone(),
        author: skill.author.clone(),
        license: skill.license.clone(),
        dare_version: skill.dare_version.clone(),
        depends_on: skill.depends_on.clone(),
    };
    let yml = serde_yaml::to_string(&manifest)
        .map_err(|e| CoreError::config(format!("serialize skill.yml: {e}")))?;
    fs::write(dest.join("skill.yml"), yml).map_err(|e| CoreError::io(e.to_string()))?;
    let md = format!(
        "# {}\n\n{}\n\nAuthor: {}\n",
        skill.name, skill.description, skill.author
    );
    fs::write(dest.join("SKILL.md"), md).map_err(|e| CoreError::io(e.to_string()))?;
    Ok(())
}

fn promote_staging(root: &ProjectRoot, name: &str, content_root: &Path) -> CoreResult<()> {
    let final_rel = skill_rel(name)?;
    let final_abs = root.resolve(&final_rel)?;
    // Ensure parent packages/skills exists.
    if let Some(parent) = final_abs.as_path().as_std_path().parent() {
        fs::create_dir_all(parent).map_err(|e| CoreError::io(e.to_string()))?;
    }

    // If content_root is already the staging dir and equals intended layout, rename.
    // Otherwise copy into a fresh staging then rename.
    let staging_rel = staging_rel(name)?;
    let staging_abs = root.resolve(&staging_rel)?;

    if content_root != staging_abs.as_path().as_std_path() {
        if staging_abs.as_path().as_std_path().exists() {
            fs::remove_dir_all(staging_abs.as_path().as_std_path())
                .map_err(|e| CoreError::io(e.to_string()))?;
        }
        fs::create_dir_all(staging_abs.as_path().as_std_path())
            .map_err(|e| CoreError::io(e.to_string()))?;
        copy_dir_safe(content_root, staging_abs.as_path().as_std_path())?;
        // If content lived under a nested extract, drop the outer leftover only when distinct.
    }

    if final_abs.as_path().as_std_path().exists() {
        let bak = final_abs
            .as_path()
            .as_std_path()
            .with_extension("bak-remove");
        let _ = fs::remove_dir_all(&bak);
        fs::rename(final_abs.as_path().as_std_path(), &bak)
            .map_err(|e| CoreError::io(e.to_string()))?;
        match fs::rename(
            staging_abs.as_path().as_std_path(),
            final_abs.as_path().as_std_path(),
        ) {
            Ok(()) => {
                let _ = fs::remove_dir_all(&bak);
            }
            Err(e) => {
                let _ = fs::rename(&bak, final_abs.as_path().as_std_path());
                return Err(CoreError::io(e.to_string()));
            }
        }
    } else {
        fs::rename(
            staging_abs.as_path().as_std_path(),
            final_abs.as_path().as_std_path(),
        )
        .map_err(|e| CoreError::io(e.to_string()))?;
    }

    // Cleanup leftover nested staging parent if content_root was nested.
    let staging_parent = root.resolve(&SafeRelativePath::new(PACKAGES_SKILLS_REL)?)?;
    if let Ok(entries) = fs::read_dir(staging_parent.as_path()) {
        for ent in entries.flatten() {
            let n = ent.file_name();
            let Some(s) = n.to_str() else { continue };
            if s.starts_with(&format!(".staging-{name}")) && ent.path().is_dir() {
                let _ = fs::remove_dir_all(ent.path());
            }
        }
    }
    Ok(())
}

fn resolve_content_root<'a>(staging: &'a Path, name: &str) -> CoreResult<&'a Path> {
    let _ = name;
    if staging.join("skill.yml").is_file() {
        return Ok(staging);
    }
    // Search one level for skill.yml and flatten into staging root.
    if let Ok(entries) = fs::read_dir(staging) {
        let mut found: Option<PathBuf> = None;
        for ent in entries.flatten() {
            let p = ent.path();
            if p.is_dir() && p.join("skill.yml").is_file() {
                if found.is_some() {
                    return Err(CoreError::invalid_input(
                        "ambiguous archive layout (multiple skill.yml)",
                    ));
                }
                found = Some(p);
            }
        }
        if let Some(p) = found {
            flatten_single_child(staging, &p)?;
            return Ok(staging);
        }
    }
    Err(CoreError::invalid_input(
        "archive missing skill.yml at package root",
    ))
}

fn flatten_single_child(staging: &Path, child: &Path) -> CoreResult<()> {
    let tmp = staging.join(".flatten-tmp");
    if tmp.exists() {
        fs::remove_dir_all(&tmp).map_err(|e| CoreError::io(e.to_string()))?;
    }
    fs::rename(child, &tmp).map_err(|e| CoreError::io(e.to_string()))?;
    // Remove other leftovers in staging
    for ent in fs::read_dir(staging).map_err(|e| CoreError::io(e.to_string()))? {
        let ent = ent.map_err(|e| CoreError::io(e.to_string()))?;
        if ent.path() == tmp {
            continue;
        }
        let p = ent.path();
        if p.is_dir() {
            fs::remove_dir_all(&p).map_err(|e| CoreError::io(e.to_string()))?;
        } else {
            fs::remove_file(&p).map_err(|e| CoreError::io(e.to_string()))?;
        }
    }
    for ent in fs::read_dir(&tmp).map_err(|e| CoreError::io(e.to_string()))? {
        let ent = ent.map_err(|e| CoreError::io(e.to_string()))?;
        let dest = staging.join(ent.file_name());
        fs::rename(ent.path(), dest).map_err(|e| CoreError::io(e.to_string()))?;
    }
    fs::remove_dir_all(&tmp).map_err(|e| CoreError::io(e.to_string()))?;
    Ok(())
}

fn extract_tar_safe<R: Read>(reader: R, dest: &Path) -> CoreResult<()> {
    let mut archive = Archive::new(reader);
    let entries = archive
        .entries()
        .map_err(|e| CoreError::invalid_input(format!("invalid tar: {e}")))?;
    for entry in entries {
        let mut entry =
            entry.map_err(|e| CoreError::invalid_input(format!("invalid tar entry: {e}")))?;
        let path = entry
            .path()
            .map_err(|e| CoreError::invalid_input(format!("invalid tar path: {e}")))?
            .into_owned();
        let name = path.to_string_lossy();
        assert_safe_archive_entry(&name)?;
        let out = dest.join(path.as_path());
        ensure_under_dest(dest, &out)?;
        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&out).map_err(|e| CoreError::io(e.to_string()))?;
        } else {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent).map_err(|e| CoreError::io(e.to_string()))?;
            }
            let mut file = File::create(&out).map_err(|e| CoreError::io(e.to_string()))?;
            io::copy(&mut entry, &mut file).map_err(|e| CoreError::io(e.to_string()))?;
        }
    }
    Ok(())
}

fn extract_zip_safe(archive: &Path, dest: &Path) -> CoreResult<()> {
    let file = File::open(archive).map_err(|e| CoreError::io(e.to_string()))?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| CoreError::invalid_input(format!("invalid zip: {e}")))?;
    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .map_err(|e| CoreError::invalid_input(format!("invalid zip entry: {e}")))?;
        let name = file.name().to_string();
        assert_safe_archive_entry(&name)?;
        let out = dest.join(Path::new(&name.replace('\\', "/")));
        ensure_under_dest(dest, &out)?;
        if file.is_dir() {
            fs::create_dir_all(&out).map_err(|e| CoreError::io(e.to_string()))?;
        } else {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent).map_err(|e| CoreError::io(e.to_string()))?;
            }
            let mut outfile = File::create(&out).map_err(|e| CoreError::io(e.to_string()))?;
            io::copy(&mut file, &mut outfile).map_err(|e| CoreError::io(e.to_string()))?;
        }
    }
    Ok(())
}

fn ensure_under_dest(dest: &Path, candidate: &Path) -> CoreResult<()> {
    let dest_canon = fs::canonicalize(dest).map_err(|e| CoreError::io(e.to_string()))?;
    // candidate may not exist yet — canonicalize parent + join
    let parent = candidate.parent().unwrap_or(dest);
    if !parent.exists() {
        fs::create_dir_all(parent).map_err(|e| CoreError::io(e.to_string()))?;
    }
    let parent_canon = fs::canonicalize(parent).map_err(|e| CoreError::io(e.to_string()))?;
    let file_name = candidate
        .file_name()
        .ok_or_else(|| CoreError::invalid_input("unsafe archive entry path"))?;
    let full = parent_canon.join(file_name);
    if !full.starts_with(&dest_canon) {
        return Err(CoreError::invalid_input("unsafe archive entry path"));
    }
    Ok(())
}

fn copy_dir_safe(src: &Path, dest: &Path) -> CoreResult<()> {
    fs::create_dir_all(dest).map_err(|e| CoreError::io(e.to_string()))?;
    for ent in fs::read_dir(src).map_err(|e| CoreError::io(e.to_string()))? {
        let ent = ent.map_err(|e| CoreError::io(e.to_string()))?;
        let name = ent.file_name();
        let name_str = name.to_string_lossy();
        if name_str == ".." || name_str == "." {
            continue;
        }
        assert_safe_archive_entry(&name_str)?;
        let from = ent.path();
        let to = dest.join(&name);
        if from.is_dir() {
            copy_dir_safe(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| CoreError::io(e.to_string()))?;
        }
    }
    Ok(())
}

fn find_reverse_dependents(root: &ProjectRoot, name: &str) -> CoreResult<Vec<String>> {
    let packages = root.resolve(&SafeRelativePath::new(PACKAGES_SKILLS_REL)?)?;
    let mut deps = Vec::new();
    if !packages.as_path().as_std_path().is_dir() {
        return Ok(deps);
    }
    for ent in
        fs::read_dir(packages.as_path().as_std_path()).map_err(|e| CoreError::io(e.to_string()))?
    {
        let ent = ent.map_err(|e| CoreError::io(e.to_string()))?;
        let other = ent.file_name();
        let Some(other_name) = other.to_str() else {
            continue;
        };
        if other_name.starts_with('.') || other_name == name {
            continue;
        }
        if validate_skill_id(other_name).is_err() {
            continue;
        }
        let yml = ent.path().join("skill.yml");
        if !yml.is_file() {
            continue;
        }
        let text = fs::read_to_string(&yml).map_err(|e| CoreError::io(e.to_string()))?;
        let manifest: SkillManifest = serde_yaml::from_str(&text)
            .map_err(|e| CoreError::config(format!("invalid skill.yml for {other_name}: {e}")))?;
        if manifest.depends_on.iter().any(|d| d == name) {
            deps.push(other_name.to_string());
        }
    }
    deps.sort();
    Ok(deps)
}

fn skill_dir_exists(root: &ProjectRoot, name: &str) -> CoreResult<bool> {
    let rel = skill_rel(name)?;
    Ok(root.resolve(&rel)?.as_path().is_dir())
}

pub fn skill_rel(name: &str) -> CoreResult<SafeRelativePath> {
    validate_skill_id(name)?;
    SafeRelativePath::new(&format!("{PACKAGES_SKILLS_REL}/{name}"))
}

fn staging_rel(name: &str) -> CoreResult<SafeRelativePath> {
    validate_skill_id(name)?;
    SafeRelativePath::new(&format!("{PACKAGES_SKILLS_REL}/.staging-{name}"))
}

fn read_skill_yml(dir: &Path) -> CoreResult<SkillManifest> {
    let text =
        fs::read_to_string(dir.join("skill.yml")).map_err(|e| CoreError::io(e.to_string()))?;
    serde_yaml::from_str(&text).map_err(|e| CoreError::config(format!("invalid skill.yml: {e}")))
}

fn upsert_manifest_entry(root: &ProjectRoot, name: &str, version: &str) -> CoreResult<()> {
    let rel = SafeRelativePath::new(PROJECT_SKILLS_REL)?;
    let path = root.resolve(&rel)?;
    let mut manifest = if path.as_path().is_file() {
        load_skills_manifest(root, &rel)?
    } else {
        SkillsManifest {
            version: Some("1".into()),
            skills: Vec::new(),
            extra: Map::new(),
        }
    };
    if let Some(existing) = manifest.skills.iter_mut().find(|s| s.id == name) {
        existing.version = Some(version.to_string());
    } else {
        manifest.skills.push(SkillEntry {
            id: name.to_string(),
            version: Some(version.to_string()),
            extra: Map::new(),
        });
    }
    manifest.skills.sort_by(|a, b| a.id.cmp(&b.id));
    // Ensure parent .dare exists via atomic_write helper path.
    if let Some(parent) = path.as_path().parent() {
        fs::create_dir_all(parent).map_err(|e| CoreError::io(e.to_string()))?;
    }
    save_skills_manifest(root, &rel, &manifest)
}

fn remove_manifest_entry(root: &ProjectRoot, name: &str) -> CoreResult<()> {
    let rel = SafeRelativePath::new(PROJECT_SKILLS_REL)?;
    let path = root.resolve(&rel)?;
    if !path.as_path().is_file() {
        return Ok(());
    }
    let mut manifest = load_skills_manifest(root, &rel)?;
    manifest.skills.retain(|s| s.id != name);
    save_skills_manifest(root, &rel, &manifest)
}

/// Pack a directory into gzip tar (used by publish); entries are relative & safe.
pub fn write_tar_gz_from_dir(src_dir: &Path, out: &Path) -> CoreResult<()> {
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|e| CoreError::io(e.to_string()))?;
    }
    let file = File::create(out).map_err(|e| CoreError::io(e.to_string()))?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut builder = tar::Builder::new(enc);
    append_dir(&mut builder, src_dir, Path::new(""))?;
    let enc = builder
        .into_inner()
        .map_err(|e| CoreError::io(e.to_string()))?;
    enc.finish().map_err(|e| CoreError::io(e.to_string()))?;
    Ok(())
}

fn append_dir<W: Write>(
    builder: &mut tar::Builder<W>,
    src: &Path,
    prefix: &Path,
) -> CoreResult<()> {
    let mut entries: Vec<_> = fs::read_dir(src)
        .map_err(|e| CoreError::io(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CoreError::io(e.to_string()))?;
    entries.sort_by_key(|e| e.file_name());
    for ent in entries {
        let name = ent.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') {
            continue;
        }
        assert_safe_archive_entry(&name_str)?;
        let path = ent.path();
        let rel = prefix.join(&name);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        assert_safe_archive_entry(&rel_str)?;
        if path.is_dir() {
            append_dir(builder, &path, &rel)?;
        } else {
            let mut file = File::open(&path).map_err(|e| CoreError::io(e.to_string()))?;
            let meta = file.metadata().map_err(|e| CoreError::io(e.to_string()))?;
            let mut header = tar::Header::new_gnu();
            header.set_metadata(&meta);
            header
                .set_path(&rel_str)
                .map_err(|e| CoreError::io(e.to_string()))?;
            header.set_cksum();
            builder
                .append(&header, &mut file)
                .map_err(|e| CoreError::io(e.to_string()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{load_project_skills, FailingHttpGet, MockRegistry, RemoteRegistry};
    use dare_core::fs::atomic_write;
    use std::io::Write;

    fn project() -> (tempfile::TempDir, ProjectRoot) {
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        (dir, root)
    }

    fn offline_registry() -> CompositeRegistry {
        CompositeRegistry::new(
            MockRegistry,
            None,
            RemoteRegistry::with_http("https://example.invalid", Box::new(FailingHttpGet)),
        )
    }

    #[test]
    fn add_installs_dare_ax_and_manifest() {
        let (_dir, root) = project();
        let reg = offline_registry();
        let report = install_skill(&root, "dare-ax", &InstallOpts::default(), &reg).unwrap();
        assert_eq!(report.name, "dare-ax");
        let skill = root
            .resolve(&SafeRelativePath::new("packages/skills/dare-ax").unwrap())
            .unwrap();
        assert!(skill.as_path().join("skill.yml").is_file());
        let m = load_project_skills(&root).unwrap();
        assert!(m.skills.iter().any(|s| s.id == "dare-ax"));
    }

    #[test]
    fn add_installs_deps() {
        let (_dir, root) = project();
        let reg = offline_registry();
        let report =
            install_skill(&root, "dare-frontend-design", &InstallOpts::default(), &reg).unwrap();
        assert!(report.installed_deps.iter().any(|d| d == "dare-ax"));
        assert!(skill_dir_exists(&root, "dare-ax").unwrap());
        assert!(skill_dir_exists(&root, "dare-frontend-design").unwrap());
    }

    #[test]
    fn remove_deletes_files() {
        let (_dir, root) = project();
        let reg = offline_registry();
        install_skill(&root, "dare-ax", &InstallOpts::default(), &reg).unwrap();
        remove_skill(&root, "dare-ax").unwrap();
        assert!(!skill_dir_exists(&root, "dare-ax").unwrap());
        let m = load_project_skills(&root).unwrap();
        assert!(!m.skills.iter().any(|s| s.id == "dare-ax"));
    }

    #[test]
    fn remove_blocked_by_reverse_deps() {
        let (_dir, root) = project();
        let reg = offline_registry();
        install_skill(&root, "dare-frontend-design", &InstallOpts::default(), &reg).unwrap();
        let err = remove_skill(&root, "dare-ax").unwrap_err();
        assert!(err.message().contains("required by"));
    }

    #[test]
    fn update_recopies_content() {
        let (_dir, root) = project();
        let reg = offline_registry();
        install_skill(&root, "dare-ax", &InstallOpts::default(), &reg).unwrap();
        let rel = SafeRelativePath::new("packages/skills/dare-ax/SKILL.md").unwrap();
        atomic_write(&root, &rel, b"stale").unwrap();
        update_skill(&root, "dare-ax", &InstallOpts::default(), &reg).unwrap();
        let text = fs::read_to_string(root.resolve(&rel).unwrap().as_path()).unwrap();
        assert!(text.contains("dare-ax"));
        assert!(!text.contains("stale"));
    }

    #[test]
    fn tar_traversal_blocked() {
        let dir = tempfile::tempdir().unwrap();
        let tar_path = dir.path().join("evil.tar");
        {
            let f = File::create(&tar_path).unwrap();
            let mut b = tar::Builder::new(f);
            let mut header = tar::Header::new_gnu();
            header.set_size(4);
            header.set_mode(0o644);
            header.set_entry_type(tar::EntryType::Regular);
            // Bypass tar crate write-time path checks (legacy archives may contain `..`).
            {
                let name = b"../evil.txt\0";
                let old = header.as_old_mut();
                old.name[..name.len()].copy_from_slice(name);
            }
            header.set_cksum();
            b.append(&header, &b"boom"[..]).unwrap();
            b.finish().unwrap();
        }
        let dest = dir.path().join("out");
        fs::create_dir_all(&dest).unwrap();
        let err = extract_archive_safe(&tar_path, &dest).unwrap_err();
        assert!(err.message().contains("unsafe archive"));
    }

    #[test]
    fn zip_traversal_blocked() {
        let dir = tempfile::tempdir().unwrap();
        let zip_path = dir.path().join("evil.zip");
        {
            let f = File::create(&zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(f);
            let opts = zip::write::SimpleFileOptions::default();
            zip.start_file("../evil.txt", opts).unwrap();
            zip.write_all(b"boom").unwrap();
            zip.finish().unwrap();
        }
        let dest = dir.path().join("out");
        fs::create_dir_all(&dest).unwrap();
        let err = extract_archive_safe(&zip_path, &dest).unwrap_err();
        assert!(err.message().contains("unsafe archive"));
    }

    #[test]
    fn assert_safe_rejects_parent() {
        assert!(assert_safe_archive_entry("../x").is_err());
        assert!(assert_safe_archive_entry("/abs").is_err());
        assert!(assert_safe_archive_entry("ok/file.txt").is_ok());
    }
}
