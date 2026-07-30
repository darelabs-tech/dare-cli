//! Download release asset + `SHA256SUMS` + `SHA256SUMS.sig` into [`SelfHome`] tmp.

use std::io::Read;
use std::path::{Path, PathBuf};

use dare_core::{CoreError, CoreResult};

use crate::paths::SelfHome;
use crate::plan::UpdatePlan;

/// Frozen User-Agent for GitHub HTTPS fetches (BLUEPRINT-053).
pub const GITHUB_UA: &str = "dare-cli-self-update";

/// Minimal HTTP GET for release artifacts (mockable in unit tests).
pub trait HttpClient {
    fn get_bytes(&self, url: &str) -> CoreResult<Vec<u8>>;
}

/// Paths written under `SelfHome/tmp/` for one update run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadedArtifacts {
    pub asset_path: PathBuf,
    pub sums_path: PathBuf,
    pub sig_path: PathBuf,
}

/// Download asset + checksums + signature into `{home}/tmp/`.
///
/// Does not verify bytes — callers run [`crate::verify::verify_sha256`] next.
pub fn download_update_artifacts(
    client: &dyn HttpClient,
    plan: &UpdatePlan,
    home: &SelfHome,
) -> CoreResult<DownloadedArtifacts> {
    home.ensure_dirs()
        .map_err(|e| CoreError::io(e.to_string()))?;

    let tmp = home.tmp_dir();
    let asset_bytes = client.get_bytes(&plan.asset_url)?;
    let sums_bytes = client.get_bytes(&plan.sums_url)?;
    let sig_bytes = client.get_bytes(&plan.sig_url)?;

    let asset_path = tmp.join(&plan.asset_name);
    let sums_path = tmp.join("SHA256SUMS");
    let sig_path = tmp.join("SHA256SUMS.sig");

    write_file(&asset_path, &asset_bytes)?;
    write_file(&sums_path, &sums_bytes)?;
    write_file(&sig_path, &sig_bytes)?;

    Ok(DownloadedArtifacts {
        asset_path,
        sums_path,
        sig_path,
    })
}

fn write_file(path: &Path, bytes: &[u8]) -> CoreResult<()> {
    std::fs::write(path, bytes).map_err(|e| CoreError::io(format!("write {}: {e}", path.display())))
}

/// Production HTTPS client via `ureq` (optional for later apply path).
#[derive(Debug, Default, Clone)]
pub struct RealHttpClient {
    timeout_secs: u64,
}

impl RealHttpClient {
    pub fn new(timeout_secs: u64) -> Self {
        Self { timeout_secs }
    }
}

impl HttpClient for RealHttpClient {
    fn get_bytes(&self, url: &str) -> CoreResult<Vec<u8>> {
        if !(url.starts_with("https://") || url.starts_with("http://127.0.0.1")) {
            return Err(CoreError::invalid_input(
                "download URL must be https (or http://127.0.0.1 for tests)",
            ));
        }
        let resp = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .user_agent(GITHUB_UA)
            .build()
            .get(url)
            .call()
            .map_err(|e| CoreError::io(format!("http get failed: {e}")))?;

        let status = resp.status();
        if !(200..300).contains(&status) {
            return Err(CoreError::io(format!("http status {status} for download")));
        }

        let mut buf = Vec::new();
        resp.into_reader()
            .read_to_end(&mut buf)
            .map_err(|e| CoreError::io(format!("http body read failed: {e}")))?;
        Ok(buf)
    }
}

/// In-memory map of URL → bytes for unit tests (no network).
#[derive(Debug, Default, Clone)]
pub struct MockHttpClient {
    pub responses: std::collections::HashMap<String, Vec<u8>>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, url: impl Into<String>, body: impl Into<Vec<u8>>) {
        self.responses.insert(url.into(), body.into());
    }
}

impl HttpClient for MockHttpClient {
    fn get_bytes(&self, url: &str) -> CoreResult<Vec<u8>> {
        self.responses
            .get(url)
            .cloned()
            .ok_or_else(|| CoreError::not_found(format!("mock missing url: {url}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{plan_update, UpdateOpts};
    use tempfile::tempdir;

    #[test]
    fn download_writes_three_files_via_mock() {
        let plan = plan_update(UpdateOpts {
            channel: None,
            version: Some("v0.1.0-alpha.2".into()),
            current_version: None,
            triple: Some("x86_64-unknown-linux-gnu".into()),
        })
        .unwrap();

        let mut mock = MockHttpClient::new();
        mock.insert(&plan.asset_url, b"asset-bytes".as_slice());
        mock.insert(&plan.sums_url, b"sums".as_slice());
        mock.insert(&plan.sig_url, b"sig".as_slice());

        let dir = tempdir().unwrap();
        let home = SelfHome::from_path(dir.path().join("self")).unwrap();
        let got = download_update_artifacts(&mock, &plan, &home).unwrap();

        assert_eq!(std::fs::read(&got.asset_path).unwrap(), b"asset-bytes");
        assert_eq!(std::fs::read(&got.sums_path).unwrap(), b"sums");
        assert_eq!(std::fs::read(&got.sig_path).unwrap(), b"sig");
    }
}
