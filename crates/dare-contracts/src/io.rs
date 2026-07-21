//! Canonical contract I/O under [`ProjectRoot`].

use dare_core::{CoreError, CoreResult};
use dare_core::fs::{atomic_write, read_to_string};
use dare_core::{ProjectRoot, SafeRelativePath};
use dare_core::to_canonical_json_string;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::MAX_CONTRACT_BYTES;

pub fn read_limited(root: &ProjectRoot, rel: &SafeRelativePath) -> CoreResult<Vec<u8>> {
    let abs = root.resolve(rel)?;
    let meta = std::fs::metadata(abs.as_path().as_std_path()).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CoreError::not_found(format!("contract file not found: {}", rel.as_str()))
        } else {
            CoreError::io(e.to_string())
        }
    })?;
    if meta.len() > MAX_CONTRACT_BYTES {
        return Err(CoreError::config("contract file exceeds size limit"));
    }
    let s = read_to_string(root, rel)?;
    Ok(s.into_bytes())
}

pub fn write_json_atomic<T: Serialize>(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    value: &T,
) -> CoreResult<()> {
    let v = serde_json::to_value(value).map_err(|e| CoreError::config(e.to_string()))?;
    let s = to_canonical_json_string(&v)?;
    atomic_write(root, rel, s.as_bytes())
}

pub fn write_yaml_atomic<T: Serialize>(
    root: &ProjectRoot,
    rel: &SafeRelativePath,
    value: &T,
) -> CoreResult<()> {
    let s = serde_yaml::to_string(value).map_err(|e| CoreError::config(e.to_string()))?;
    atomic_write(root, rel, s.as_bytes())
}

pub fn from_json_slice<T: DeserializeOwned>(bytes: &[u8]) -> CoreResult<T> {
    serde_json::from_slice(bytes).map_err(|e| CoreError::config(format!("invalid json: {e}")))
}

pub fn from_yaml_str<T: DeserializeOwned>(s: &str) -> CoreResult<T> {
    serde_yaml::from_str(s).map_err(|e| CoreError::config(format!("invalid yaml: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::SafeRelativePath;
    use tempfile::tempdir;

    #[test]
    fn read_limited_rejects_oversize() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("big.bin").unwrap();
        let path = dir.path().join("big.bin");
        // Write slightly over 2 MiB
        let chunk = vec![b'x'; 1024 * 64];
        let mut f = std::fs::File::create(&path).unwrap();
        use std::io::Write;
        for _ in 0..(MAX_CONTRACT_BYTES / chunk.len() as u64 + 2) {
            f.write_all(&chunk).unwrap();
        }
        drop(f);
        let err = read_limited(&root, &rel).unwrap_err();
        assert!(matches!(err, CoreError::Config(_)));
        assert!(err.to_string().contains("size limit"));
    }

    #[test]
    fn write_json_atomic_lexicographic_keys() {
        let dir = tempdir().unwrap();
        let root = ProjectRoot::new(dir.path()).unwrap();
        let rel = SafeRelativePath::new("out.json").unwrap();
        let v = serde_json::json!({"z": 1, "a": 2});
        write_json_atomic(&root, &rel, &v).unwrap();
        let s = read_to_string(&root, &rel).unwrap();
        assert_eq!(s, r#"{"a":2,"z":1}"#);
    }
}
