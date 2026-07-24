//! Secure skill publish: MIT + dare_version gates, tar.gz, SHA-256, Ed25519 (microplano 045).

use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use dare_core::{CoreError, CoreResult, ProjectRoot, SafeRelativePath};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use crate::install::{skill_rel, write_tar_gz_from_dir, PACKAGES_SKILLS_REL};
use crate::model::{validate_skill_id, SkillManifest};

pub const ENV_SKILL_PRIVATE_KEY: &str = "DARE_SKILL_PRIVATE_KEY";
pub const REQUIRED_LICENSE: &str = "MIT";
pub const SIG_EXT: &str = ".minisig";
const SIG_HEADER: &str = "untrusted comment: dare-skills ed25519";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishReport {
    pub name: String,
    pub version: String,
    pub artifact: String,
    pub sha256: String,
    pub signature: Option<String>,
}

/// Reject non-MIT or missing dare_version.
pub fn validate_for_publish(manifest: &SkillManifest) -> CoreResult<()> {
    if manifest.license.trim() != REQUIRED_LICENSE {
        return Err(CoreError::invalid_input(format!(
            "publish requires license {REQUIRED_LICENSE}, got {}",
            manifest.license
        )));
    }
    match &manifest.dare_version {
        Some(v) if !v.trim().is_empty() => Ok(()),
        _ => Err(CoreError::invalid_input(
            "publish requires dare_version in skill.yml",
        )),
    }
}

/// Pack installed skill to `out_dir/<name>-<version>.tar.gz` + `.sha256` (+ `.minisig` if key).
pub fn publish_skill(root: &ProjectRoot, name: &str, out_dir: &Path) -> CoreResult<PublishReport> {
    validate_skill_id(name)?;
    let rel = skill_rel(name)?;
    let skill_dir = root.resolve(&rel)?;
    if !skill_dir.as_path().as_std_path().is_dir() {
        return Err(CoreError::not_found(format!("skill not installed: {name}")));
    }
    let yml = skill_dir.as_path().as_std_path().join("skill.yml");
    let text = std::fs::read_to_string(&yml).map_err(|e| CoreError::io(e.to_string()))?;
    let manifest: SkillManifest = serde_yaml::from_str(&text)
        .map_err(|e| CoreError::config(format!("invalid skill.yml: {e}")))?;
    if manifest.name != name {
        return Err(CoreError::invalid_input(format!(
            "skill.yml name mismatch: expected {name}, got {}",
            manifest.name
        )));
    }
    validate_for_publish(&manifest)?;

    std::fs::create_dir_all(out_dir).map_err(|e| CoreError::io(e.to_string()))?;
    let artifact_name = format!("{}-{}.tar.gz", manifest.name, manifest.version);
    let artifact = out_dir.join(&artifact_name);
    write_tar_gz_from_dir(skill_dir.as_path().as_std_path(), &artifact)?;

    let bytes = std::fs::read(&artifact).map_err(|e| CoreError::io(e.to_string()))?;
    let sha256 = hex_encode(&sha256_bytes(&bytes));
    let sha_path = PathBuf::from(format!("{}.sha256", artifact.display()));
    std::fs::write(&sha_path, format!("{sha256}  {artifact_name}\n"))
        .map_err(|e| CoreError::io(e.to_string()))?;

    let signature = match std::env::var(ENV_SKILL_PRIVATE_KEY) {
        Ok(key) if !key.trim().is_empty() => {
            sign_artifact(&artifact, key.trim())?;
            Some(format!("{}{SIG_EXT}", artifact.display()))
        }
        _ => None,
    };

    Ok(PublishReport {
        name: manifest.name,
        version: manifest.version,
        artifact: artifact.display().to_string(),
        sha256,
        signature,
    })
}

/// SHA-256 hex of file bytes (also used after pack).
pub fn sha256_file(path: &Path) -> CoreResult<String> {
    let bytes = std::fs::read(path).map_err(|e| CoreError::io(e.to_string()))?;
    Ok(hex_encode(&sha256_bytes(&bytes)))
}

pub fn sign_artifact(path: &Path, private_key_hex: &str) -> CoreResult<()> {
    let bytes = std::fs::read(path).map_err(|e| CoreError::io(e.to_string()))?;
    let sk = signing_key_from_hex(private_key_hex)?;
    let digest = sha256_bytes(&bytes);
    let sig = sk.sign(&digest);
    let body = format!("{SIG_HEADER}\n{}\n", B64.encode(sig.to_bytes()));
    let sig_path = sig_path_for(path);
    std::fs::write(&sig_path, body)
        .map_err(|e| CoreError::io(format!("write {}: {e}", sig_path.display())))?;
    Ok(())
}

pub fn verify_artifact(path: &Path, public_key_hex: &str) -> CoreResult<()> {
    let bytes = std::fs::read(path).map_err(|e| CoreError::io(e.to_string()))?;
    let sig_path = sig_path_for(path);
    let sig_text = std::fs::read_to_string(&sig_path).map_err(|_| {
        CoreError::invalid_input(format!("missing signature: {}", sig_path.display()))
    })?;
    let sig_b64 = parse_sig_body(&sig_text)?;
    let sig_bytes = B64
        .decode(sig_b64.trim())
        .map_err(|_| CoreError::invalid_input("invalid signature encoding"))?;
    if sig_bytes.len() != 64 {
        return Err(CoreError::invalid_input("invalid signature length"));
    }
    let mut arr = [0u8; 64];
    arr.copy_from_slice(&sig_bytes);
    let sig = Signature::from_bytes(&arr);
    let vk = verifying_key_from_hex(public_key_hex)?;
    let digest = sha256_bytes(&bytes);
    vk.verify(&digest, &sig)
        .map_err(|_| CoreError::invalid_input("signature verification failed"))?;
    Ok(())
}

pub fn public_key_hex_from_private(private_key_hex: &str) -> CoreResult<String> {
    let sk = signing_key_from_hex(private_key_hex)?;
    Ok(hex_encode(sk.verifying_key().as_bytes()))
}

pub fn sig_path_for(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(SIG_EXT);
    PathBuf::from(s)
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().into()
}

fn signing_key_from_hex(hex: &str) -> CoreResult<SigningKey> {
    Ok(SigningKey::from_bytes(&decode_hex32(hex)?))
}

fn verifying_key_from_hex(hex: &str) -> CoreResult<VerifyingKey> {
    VerifyingKey::from_bytes(&decode_hex32(hex)?)
        .map_err(|e| CoreError::config(format!("invalid public key: {e}")))
}

fn decode_hex32(hex: &str) -> CoreResult<[u8; 32]> {
    let hex = hex.trim();
    if hex.len() != 64 {
        return Err(CoreError::invalid_input(
            "key must be 64 hex chars (32 bytes)",
        ));
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|_| CoreError::invalid_input("key hex decode failed"))?;
    }
    Ok(out)
}

fn parse_sig_body(text: &str) -> CoreResult<&str> {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let first = lines
        .next()
        .ok_or_else(|| CoreError::invalid_input("empty signature file"))?;
    if first.starts_with("untrusted comment:") {
        lines
            .next()
            .ok_or_else(|| CoreError::invalid_input("signature body missing"))
    } else {
        Ok(first)
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Helper: relative path of packages/skills for docs/tests.
pub fn packages_skills_prefix() -> &'static str {
    PACKAGES_SKILLS_REL
}

/// Load skill.yml from an installed package (project-relative).
pub fn load_installed_manifest(root: &ProjectRoot, name: &str) -> CoreResult<SkillManifest> {
    let rel = SafeRelativePath::new(&format!("{PACKAGES_SKILLS_REL}/{name}/skill.yml"))?;
    let path = root.resolve(&rel)?;
    let text = std::fs::read_to_string(path.as_path()).map_err(|e| CoreError::io(e.to_string()))?;
    serde_yaml::from_str(&text).map_err(|e| CoreError::config(format!("invalid skill.yml: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::install::{install_skill, InstallOpts};
    use crate::registry::{CompositeRegistry, FailingHttpGet, MockRegistry, RemoteRegistry};
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn offline_registry() -> CompositeRegistry {
        CompositeRegistry::new(
            MockRegistry,
            None,
            RemoteRegistry::with_http("https://example.invalid", Box::new(FailingHttpGet)),
        )
    }

    #[test]
    fn validate_requires_mit_and_dare_version() {
        let mut m = SkillManifest {
            name: "x".into(),
            version: "1".into(),
            description: String::new(),
            author: String::new(),
            license: "Apache-2.0".into(),
            dare_version: Some(">=3".into()),
            depends_on: vec![],
        };
        assert!(validate_for_publish(&m).is_err());
        m.license = "MIT".into();
        m.dare_version = None;
        assert!(validate_for_publish(&m).is_err());
        m.dare_version = Some(">=3.0.0".into());
        assert!(validate_for_publish(&m).is_ok());
    }

    #[test]
    fn publish_writes_artifact_and_hash() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var(ENV_SKILL_PRIVATE_KEY);
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let reg = offline_registry();
        install_skill(&root, "dare-ax", &InstallOpts::default(), &reg).unwrap();
        let out = dir.path().join("dist");
        let report = publish_skill(&root, "dare-ax", &out).unwrap();
        assert!(Path::new(&report.artifact).is_file());
        assert_eq!(report.sha256.len(), 64);
        assert!(Path::new(&format!("{}.sha256", report.artifact)).is_file());
        assert!(report.signature.is_none());
    }

    #[test]
    fn publish_signs_when_key_set() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let reg = offline_registry();
        install_skill(&root, "dare-ax", &InstallOpts::default(), &reg).unwrap();
        let sk = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        std::env::set_var(ENV_SKILL_PRIVATE_KEY, sk);
        let out = dir.path().join("dist");
        let report = publish_skill(&root, "dare-ax", &out).unwrap();
        std::env::remove_var(ENV_SKILL_PRIVATE_KEY);
        assert!(report.signature.is_some());
        let pk = public_key_hex_from_private(sk).unwrap();
        verify_artifact(Path::new(&report.artifact), &pk).unwrap();
    }
}
