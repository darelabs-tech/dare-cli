//! Apply self-update: lock → download/verify → backup → atomic replace → smoke → cleanup.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use dare_core::{CoreError, CoreResult};
use flate2::read::GzDecoder;
use tar::Archive;

use crate::download::{download_update_artifacts, DownloadedArtifacts, HttpClient, RealHttpClient};
use crate::lock::{acquire_lock, force_unlock_if_stale, LockHeld};
use crate::paths::{backup_binary_name, SelfHome};
use crate::plan::{plan_update, UpdateOpts};
use crate::verify::{
    allow_unsigned_enabled, reject_if_signing_skipped, timeout_from_env, verify_sha256,
    warn_allow_unsigned, SignatureVerifier,
};

/// Report produced by a successful [`apply_update`] / [`apply_with`] (schemaVersion 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyReport {
    pub schema_version: u32,
    pub ok: bool,
    pub mode: String,
    pub channel: String,
    pub current_version: String,
    pub target_tag: String,
    pub target_triple: String,
    pub asset_name: String,
    pub backup_path: PathBuf,
    pub replaced_path: PathBuf,
}

/// Injectable failpoints for testing interrupt / restore semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyFailpoint {
    /// After backup is written, before replacing `current_exe`.
    AfterBackupBeforeReplace,
    /// After replace succeeds, before smoke — forces restore from backup.
    AfterReplaceBeforeSmoke,
}

/// Injectable parameters for [`apply_with`] (tests / CLI orchestration).
pub struct ApplyParams<'a> {
    pub opts: UpdateOpts,
    pub home: SelfHome,
    pub current_exe: PathBuf,
    pub client: &'a dyn HttpClient,
    /// When set, skip download and reuse these paths.
    pub artifacts: Option<DownloadedArtifacts>,
    pub failpoint: Option<ApplyFailpoint>,
    pub force_unlock: bool,
    /// Skip `--version` smoke (unit tests with non-executable payloads).
    pub skip_smoke: bool,
}

/// Path to the rollback backup binary for this home.
pub fn backup_binary_path(home: &SelfHome) -> PathBuf {
    home.backup_binary_path()
}

/// Production entry: resolve home + `current_exe`, download via ureq, apply.
pub fn apply_update(
    opts: UpdateOpts,
    verifier: &dyn SignatureVerifier,
) -> CoreResult<ApplyReport> {
    let home = SelfHome::resolve().map_err(|e| CoreError::io(e.to_string()))?;
    let current_exe = std::env::current_exe()
        .map_err(|e| CoreError::not_found(format!("cannot resolve current_exe: {e}")))?;
    let timeout = timeout_from_env().as_secs().max(1);
    let client = RealHttpClient::new(timeout);
    apply_with(
        ApplyParams {
            opts,
            home,
            current_exe,
            client: &client,
            artifacts: None,
            failpoint: None,
            force_unlock: false,
            skip_smoke: false,
        },
        verifier,
    )
}

/// Full apply pipeline with injectable I/O (preferred for tests).
pub fn apply_with(
    params: ApplyParams<'_>,
    verifier: &dyn SignatureVerifier,
) -> CoreResult<ApplyReport> {
    let ApplyParams {
        opts,
        home,
        current_exe,
        client,
        artifacts,
        failpoint,
        force_unlock,
        skip_smoke,
    } = params;

    if !current_exe.is_file() {
        return Err(CoreError::not_found(format!(
            "current_exe is not a file: {}",
            current_exe.display()
        )));
    }

    home.ensure_dirs()
        .map_err(|e| CoreError::io(e.to_string()))?;

    if force_unlock {
        let _ = force_unlock_if_stale(&home);
    }

    let _lock = acquire_lock(&home).map_err(|e: LockHeld| {
        if e.message() == crate::lock::MSG_LOCK_HELD {
            CoreError::invalid_input(e.message())
        } else {
            CoreError::io(e.message())
        }
    })?;

    let plan = plan_update(opts)?;
    let arts = match artifacts {
        Some(a) => a,
        None => download_update_artifacts(client, &plan, &home)?,
    };

    let asset_bytes = fs::read(&arts.asset_path).map_err(|e| {
        CoreError::io(format!("read asset {}: {e}", arts.asset_path.display()))
    })?;
    let sums_text = fs::read_to_string(&arts.sums_path).map_err(|e| {
        CoreError::io(format!("read sums {}: {e}", arts.sums_path.display()))
    })?;

    verify_sha256(&asset_bytes, &sums_text, &plan.asset_name)?;

    // Always reject "signing skipped" before allow-unsigned / cosign.
    reject_if_signing_skipped(&arts.sig_path)?;
    if allow_unsigned_enabled() {
        warn_allow_unsigned();
    } else {
        verifier.verify_sums(&arts.sums_path, &arts.sig_path)?;
    }

    let extract_dir = home.tmp_dir().join("extract");
    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir).map_err(|e| CoreError::io(e.to_string()))?;
    }
    fs::create_dir_all(&extract_dir).map_err(|e| CoreError::io(e.to_string()))?;

    extract_archive(&arts.asset_path, &plan.asset_name, &extract_dir)?;
    let new_bin = find_dare_binary(&extract_dir)?;

    let backup_path = backup_binary_path(&home);
    fs::copy(&current_exe, &backup_path).map_err(|e| {
        CoreError::io(format!(
            "backup {} → {}: {e}",
            current_exe.display(),
            backup_path.display()
        ))
    })?;

    if failpoint == Some(ApplyFailpoint::AfterBackupBeforeReplace) {
        return Err(CoreError::internal(
            "apply failpoint: interrupted after backup (current_exe preserved)",
        ));
    }

    let replace_result = atomic_replace(&new_bin, &current_exe);
    if let Err(e) = replace_result {
        let _ = restore_from_backup(&backup_path, &current_exe);
        return Err(e);
    }

    if failpoint == Some(ApplyFailpoint::AfterReplaceBeforeSmoke) {
        let _ = restore_from_backup(&backup_path, &current_exe);
        return Err(CoreError::internal(
            "apply failpoint: interrupted after replace; restored backup",
        ));
    }

    if !skip_smoke {
        if let Err(e) = smoke_version(&current_exe, timeout_from_env()) {
            let _ = restore_from_backup(&backup_path, &current_exe);
            return Err(e);
        }
    }

    cleanup_tmp(&home);

    Ok(ApplyReport {
        schema_version: 1,
        ok: true,
        mode: "update".to_string(),
        channel: plan.channel,
        current_version: plan.current_version,
        target_tag: plan.target_tag,
        target_triple: plan.target_triple,
        asset_name: plan.asset_name,
        backup_path,
        replaced_path: current_exe,
    })
}

fn cleanup_tmp(home: &SelfHome) {
    let tmp = home.tmp_dir();
    if let Ok(entries) = fs::read_dir(&tmp) {
        for ent in entries.flatten() {
            let p = ent.path();
            if p.is_dir() {
                let _ = fs::remove_dir_all(&p);
            } else {
                let _ = fs::remove_file(&p);
            }
        }
    }
}

fn restore_from_backup(backup: &Path, target: &Path) -> CoreResult<()> {
    fs::copy(backup, target).map_err(|e| {
        CoreError::io(format!(
            "restore backup {} → {}: {e}",
            backup.display(),
            target.display()
        ))
    })?;
    Ok(())
}

/// Replace `target` with bytes/file at `new_bin`, restoring on failure when a side-old exists.
fn atomic_replace(new_bin: &Path, target: &Path) -> CoreResult<()> {
    #[cfg(windows)]
    {
        let side_old = side_old_path(target);
        let _ = fs::remove_file(&side_old);
        fs::rename(target, &side_old).map_err(|e| {
            CoreError::io(format!(
                "rename current aside {}: {e}",
                target.display()
            ))
        })?;
        match fs::copy(new_bin, target) {
            Ok(_) => {
                let _ = fs::remove_file(&side_old);
                Ok(())
            }
            Err(e) => {
                let _ = fs::rename(&side_old, target);
                Err(CoreError::io(format!(
                    "replace {}: {e}",
                    target.display()
                )))
            }
        }
    }
    #[cfg(not(windows))]
    {
        let staging = target.with_extension("dare-new");
        fs::copy(new_bin, &staging).map_err(|e| {
            CoreError::io(format!("stage new binary {}: {e}", staging.display()))
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let meta = fs::metadata(new_bin).map_err(|e| CoreError::io(e.to_string()))?;
            let mut perms = meta.permissions();
            let mode = perms.mode();
            perms.set_mode(mode | 0o111);
            let _ = fs::set_permissions(&staging, perms);
        }
        fs::rename(&staging, target).map_err(|e| {
            let _ = fs::remove_file(&staging);
            CoreError::io(format!("atomic rename onto {}: {e}", target.display()))
        })?;
        Ok(())
    }
}

#[cfg(windows)]
fn side_old_path(target: &Path) -> PathBuf {
    let mut name = target
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_else(|| "dare.exe".into());
    name.push(".dare-old");
    target.with_file_name(name)
}

fn smoke_version(exe: &Path, timeout: Duration) -> CoreResult<()> {
    let mut child = Command::new(exe)
        .arg("--version")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| CoreError::io(format!("smoke --version spawn failed: {e}")))?;

    let deadline = std::time::Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    return Ok(());
                }
                return Err(CoreError::io(format!(
                    "smoke --version failed (exit {status})"
                )));
            }
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(CoreError::io("smoke --version timed out"));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => return Err(CoreError::io(format!("smoke wait failed: {e}"))),
        }
    }
}

fn extract_archive(asset_path: &Path, asset_name: &str, dest: &Path) -> CoreResult<()> {
    if asset_name.ends_with(".zip") {
        extract_zip(asset_path, dest)
    } else if asset_name.ends_with(".tar.gz") || asset_name.ends_with(".tgz") {
        let file = File::open(asset_path)
            .map_err(|e| CoreError::io(format!("open archive {}: {e}", asset_path.display())))?;
        let gz = GzDecoder::new(file);
        extract_tar(gz, dest)
    } else {
        // Raw binary payload (tests / unusual assets): copy as dare[.exe]
        let dest_bin = dest.join(backup_binary_name());
        fs::copy(asset_path, &dest_bin).map_err(|e| {
            CoreError::io(format!("copy raw asset to {}: {e}", dest_bin.display()))
        })?;
        Ok(())
    }
}

fn extract_tar<R: Read>(reader: R, dest: &Path) -> CoreResult<()> {
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

fn extract_zip(archive: &Path, dest: &Path) -> CoreResult<()> {
    let file = File::open(archive)
        .map_err(|e| CoreError::io(format!("open zip {}: {e}", archive.display())))?;
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

fn assert_safe_archive_entry(name: &str) -> CoreResult<()> {
    if name.is_empty() || name.contains('\0') {
        return Err(CoreError::invalid_input("unsafe archive entry path"));
    }
    let n = name.replace('\\', "/");
    if n.starts_with('/') || n.starts_with("..") || n.contains("/../") || n.contains("/..") {
        return Err(CoreError::invalid_input("unsafe archive entry path"));
    }
    for part in n.split('/') {
        if part == ".." {
            return Err(CoreError::invalid_input("unsafe archive entry path"));
        }
    }
    Ok(())
}

fn ensure_under_dest(dest: &Path, candidate: &Path) -> CoreResult<()> {
    let dest_canon = fs::canonicalize(dest).map_err(|e| CoreError::io(e.to_string()))?;
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

fn find_dare_binary(extract_dir: &Path) -> CoreResult<PathBuf> {
    let preferred = [backup_binary_name(), "dare", "dare.exe"];
    for name in preferred {
        let p = extract_dir.join(name);
        if p.is_file() {
            return Ok(p);
        }
    }
    // Single regular file in extract root
    let mut files = Vec::new();
    collect_files(extract_dir, &mut files)?;
    if files.len() == 1 {
        return Ok(files.remove(0));
    }
    // Nested dare / dare.exe
    for f in &files {
        if let Some(n) = f.file_name().and_then(|s| s.to_str()) {
            if n == "dare" || n == "dare.exe" {
                return Ok(f.clone());
            }
        }
    }
    Err(CoreError::not_found(
        "release archive does not contain dare binary",
    ))
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> CoreResult<()> {
    for ent in fs::read_dir(dir).map_err(|e| CoreError::io(e.to_string()))? {
        let ent = ent.map_err(|e| CoreError::io(e.to_string()))?;
        let p = ent.path();
        if p.is_dir() {
            collect_files(&p, out)?;
        } else if p.is_file() {
            out.push(p);
        }
    }
    Ok(())
}

/// Build a zip archive containing a single file (test helper).
#[cfg(test)]
pub fn write_zip_with_file(zip_path: &Path, entry_name: &str, contents: &[u8]) -> CoreResult<()> {
    use std::io::Write;
    let file = File::create(zip_path).map_err(|e| CoreError::io(e.to_string()))?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default();
    zip.start_file(entry_name, opts)
        .map_err(|e| CoreError::io(e.to_string()))?;
    zip.write_all(contents)
        .map_err(|e| CoreError::io(e.to_string()))?;
    zip.finish()
        .map_err(|e| CoreError::io(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    let dig = h.finalize();
    dig.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::download::MockHttpClient;
    use crate::verify::{RejectSkippedVerifier, MSG_SIGNING_SKIPPED};
    use tempfile::tempdir;

    struct OkVerifier;
    impl SignatureVerifier for OkVerifier {
        fn verify_sums(&self, _sums: &Path, sig: &Path) -> CoreResult<()> {
            reject_if_signing_skipped(sig)
        }
    }

    #[test]
    fn apply_failpoint_preserves_previous() {
        let dir = tempdir().unwrap();
        let home = SelfHome::from_path(dir.path().join("self")).unwrap();

        let current = dir.path().join("install").join(backup_binary_name());
        fs::create_dir_all(current.parent().unwrap()).unwrap();
        fs::write(&current, b"OLD-BINARY-CONTENT").unwrap();

        let triple = if cfg!(windows) {
            "x86_64-pc-windows-msvc"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        let plan = plan_update(UpdateOpts {
            channel: None,
            version: Some("v0.1.0-alpha.2".into()),
            current_version: Some("0.1.0-alpha.0".into()),
            triple: Some(triple.into()),
        })
        .unwrap();

        let new_bytes = b"NEW-BINARY-CONTENT";
        let asset_path = home.tmp_dir().join(&plan.asset_name);
        if plan.asset_name.ends_with(".zip") {
            write_zip_with_file(&asset_path, backup_binary_name(), new_bytes).unwrap();
        } else {
            // Write a minimal tar.gz for unix
            write_targz_with_file(&asset_path, backup_binary_name(), new_bytes).unwrap();
        }
        let asset_bytes = fs::read(&asset_path).unwrap();
        let sums = format!("{}  {}\n", sha256_hex(&asset_bytes), plan.asset_name);
        let sums_path = home.tmp_dir().join("SHA256SUMS");
        let sig_path = home.tmp_dir().join("SHA256SUMS.sig");
        fs::write(&sums_path, &sums).unwrap();
        fs::write(&sig_path, b"good-signature-blob").unwrap();

        let arts = DownloadedArtifacts {
            asset_path,
            sums_path,
            sig_path,
        };

        let client = MockHttpClient::new();
        let err = apply_with(
            ApplyParams {
                opts: UpdateOpts {
                    channel: None,
                    version: Some("v0.1.0-alpha.2".into()),
                    current_version: Some("0.1.0-alpha.0".into()),
                    triple: Some(triple.into()),
                },
                home: home.clone(),
                current_exe: current.clone(),
                client: &client,
                artifacts: Some(arts),
                failpoint: Some(ApplyFailpoint::AfterBackupBeforeReplace),
                force_unlock: false,
                skip_smoke: true,
            },
            &OkVerifier,
        )
        .unwrap_err();

        assert!(
            err.message().contains("failpoint") || err.message().contains("interrupted"),
            "unexpected err: {}",
            err.message()
        );
        assert_eq!(
            fs::read(&current).unwrap(),
            b"OLD-BINARY-CONTENT",
            "current_exe must remain old after failpoint"
        );
        let backup = backup_binary_path(&home);
        assert!(backup.is_file(), "backup must exist after failpoint");
        assert_eq!(fs::read(&backup).unwrap(), b"OLD-BINARY-CONTENT");
    }

    #[test]
    fn apply_signing_skipped_before_replace() {
        let dir = tempdir().unwrap();
        let home = SelfHome::from_path(dir.path().join("self")).unwrap();
        let current = dir.path().join("bin").join(backup_binary_name());
        fs::create_dir_all(current.parent().unwrap()).unwrap();
        fs::write(&current, b"OLD").unwrap();

        let triple = "x86_64-pc-windows-msvc";
        let plan = plan_update(UpdateOpts {
            channel: None,
            version: Some("v1.0.0".into()),
            current_version: None,
            triple: Some(triple.into()),
        })
        .unwrap();

        let asset_path = home.tmp_dir().join(&plan.asset_name);
        write_zip_with_file(&asset_path, backup_binary_name(), b"NEW").unwrap();
        let asset_bytes = fs::read(&asset_path).unwrap();
        let sums = format!("{}  {}\n", sha256_hex(&asset_bytes), plan.asset_name);
        let sums_path = home.tmp_dir().join("SHA256SUMS");
        let sig_path = home.tmp_dir().join("SHA256SUMS.sig");
        fs::write(&sums_path, &sums).unwrap();
        fs::write(&sig_path, b"signing skipped").unwrap();

        let err = apply_with(
            ApplyParams {
                opts: UpdateOpts {
                    channel: None,
                    version: Some("v1.0.0".into()),
                    current_version: None,
                    triple: Some(triple.into()),
                },
                home,
                current_exe: current.clone(),
                client: &MockHttpClient::new(),
                artifacts: Some(DownloadedArtifacts {
                    asset_path,
                    sums_path,
                    sig_path,
                }),
                failpoint: None,
                force_unlock: false,
                skip_smoke: true,
            },
            &RejectSkippedVerifier,
        )
        .unwrap_err();

        assert_eq!(err.message(), MSG_SIGNING_SKIPPED);
        assert_eq!(fs::read(&current).unwrap(), b"OLD");
    }

    #[cfg(not(windows))]
    fn write_targz_with_file(path: &Path, entry_name: &str, contents: &[u8]) -> CoreResult<()> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        let file = File::create(path).map_err(|e| CoreError::io(e.to_string()))?;
        let enc = GzEncoder::new(file, Compression::default());
        let mut builder = tar::Builder::new(enc);
        let mut header = tar::Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        builder
            .append_data(&mut header, entry_name, contents)
            .map_err(|e| CoreError::io(e.to_string()))?;
        builder
            .finish()
            .map_err(|e| CoreError::io(e.to_string()))?;
        Ok(())
    }

    #[cfg(windows)]
    fn write_targz_with_file(_path: &Path, _entry_name: &str, _contents: &[u8]) -> CoreResult<()> {
        Err(CoreError::internal("tar.gz helper unused on windows tests"))
    }

    #[test]
    fn apply_success_with_mock_verifier() {
        let dir = tempdir().unwrap();
        let home = SelfHome::from_path(dir.path().join("self")).unwrap();
        let current = dir.path().join("bin").join(backup_binary_name());
        fs::create_dir_all(current.parent().unwrap()).unwrap();
        fs::write(&current, b"OLD-BINARY-CONTENT").unwrap();

        let triple = if cfg!(windows) {
            "x86_64-pc-windows-msvc"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        let plan = plan_update(UpdateOpts {
            channel: None,
            version: Some("v0.2.0".into()),
            current_version: Some("0.1.0".into()),
            triple: Some(triple.into()),
        })
        .unwrap();

        let new_bytes = b"NEW-BINARY-CONTENT";
        let asset_path = home.tmp_dir().join(&plan.asset_name);
        if plan.asset_name.ends_with(".zip") {
            write_zip_with_file(&asset_path, backup_binary_name(), new_bytes).unwrap();
        } else {
            write_targz_with_file(&asset_path, backup_binary_name(), new_bytes).unwrap();
        }
        let asset_bytes = fs::read(&asset_path).unwrap();
        let sums = format!("{}  {}\n", sha256_hex(&asset_bytes), plan.asset_name);
        let sums_path = home.tmp_dir().join("SHA256SUMS");
        let sig_path = home.tmp_dir().join("SHA256SUMS.sig");
        fs::write(&sums_path, &sums).unwrap();
        fs::write(&sig_path, b"sig-ok").unwrap();

        let report = apply_with(
            ApplyParams {
                opts: UpdateOpts {
                    channel: None,
                    version: Some("v0.2.0".into()),
                    current_version: Some("0.1.0".into()),
                    triple: Some(triple.into()),
                },
                home: home.clone(),
                current_exe: current.clone(),
                client: &MockHttpClient::new(),
                artifacts: Some(DownloadedArtifacts {
                    asset_path,
                    sums_path,
                    sig_path,
                }),
                failpoint: None,
                force_unlock: false,
                skip_smoke: true,
            },
            &OkVerifier,
        )
        .unwrap();

        assert!(report.ok);
        assert_eq!(report.mode, "update");
        assert_eq!(fs::read(&current).unwrap(), b"NEW-BINARY-CONTENT");
        assert_eq!(
            fs::read(backup_binary_path(&home)).unwrap(),
            b"OLD-BINARY-CONTENT"
        );
    }

}
