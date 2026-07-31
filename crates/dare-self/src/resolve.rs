//! Resolve a release channel to a GitHub Release tag (BLUEPRINT-053 / 056).

use dare_core::{CoreError, CoreResult};
use serde::Deserialize;

use crate::channel::Channel;
use crate::download::{HttpClient, RealHttpClient};
use crate::plan::{release_api_base, release_repo};
use crate::verify::timeout_from_env;

/// Frozen message when stable has no non-prerelease Release (exit 4).
pub const MSG_STABLE_UNAVAILABLE: &str =
    "stable channel has no non-prerelease GitHub Release";

/// Frozen message when beta has no prerelease Release (exit 4).
pub const MSG_BETA_UNAVAILABLE: &str = "beta channel has no prerelease GitHub Release";

#[derive(Debug, Deserialize)]
struct GhRelease {
    tag_name: String,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
}

/// Resolve `channel` to a release tag via the GitHub Releases API.
///
/// - **stable** → `GET /repos/{repo}/releases/latest` (non-prerelease); empty → [`MSG_STABLE_UNAVAILABLE`]
/// - **beta** → first non-draft prerelease in `GET /repos/{repo}/releases`; empty → [`MSG_BETA_UNAVAILABLE`]
pub fn resolve_channel_tag(channel: Channel, client: &dyn HttpClient) -> CoreResult<String> {
    let api = release_api_base();
    let repo = release_repo();
    match channel {
        Channel::Stable => resolve_stable(&api, &repo, client),
        Channel::Beta => resolve_beta(&api, &repo, client),
    }
}

fn resolve_stable(api: &str, repo: &str, client: &dyn HttpClient) -> CoreResult<String> {
    let url = format!("{api}/repos/{repo}/releases/latest");
    let body = match client.get_bytes(&url) {
        Ok(b) => b,
        Err(e) if is_http_404(&e) => {
            return Err(CoreError::invalid_input(MSG_STABLE_UNAVAILABLE));
        }
        Err(e) => return Err(e),
    };
    let rel: GhRelease = serde_json::from_slice(&body)
        .map_err(|e| CoreError::io(format!("parse GitHub latest release: {e}")))?;
    if rel.draft || rel.prerelease || rel.tag_name.trim().is_empty() {
        return Err(CoreError::invalid_input(MSG_STABLE_UNAVAILABLE));
    }
    normalize_resolved_tag(&rel.tag_name)
}

fn resolve_beta(api: &str, repo: &str, client: &dyn HttpClient) -> CoreResult<String> {
    let url = format!("{api}/repos/{repo}/releases?per_page=30");
    let body = client.get_bytes(&url)?;
    let releases: Vec<GhRelease> = serde_json::from_slice(&body)
        .map_err(|e| CoreError::io(format!("parse GitHub releases list: {e}")))?;
    let tag = releases
        .into_iter()
        .find(|r| r.prerelease && !r.draft && !r.tag_name.trim().is_empty())
        .map(|r| r.tag_name)
        .ok_or_else(|| CoreError::invalid_input(MSG_BETA_UNAVAILABLE))?;
    normalize_resolved_tag(&tag)
}

fn normalize_resolved_tag(raw: &str) -> CoreResult<String> {
    let t = raw.trim();
    if t.is_empty() {
        return Err(CoreError::invalid_input(
            "resolved release tag must not be empty",
        ));
    }
    if t.starts_with('v') || t.starts_with('V') {
        Ok(format!("v{}", &t[1..]))
    } else {
        Ok(format!("v{t}"))
    }
}

fn is_http_404(err: &CoreError) -> bool {
    err.message().contains("http status 404")
}

/// Convenience: resolve with the production HTTPS client.
pub fn resolve_channel_tag_http(channel: Channel) -> CoreResult<String> {
    let client = RealHttpClient::new(timeout_from_env().as_secs());
    resolve_channel_tag(channel, &client)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::download::MockHttpClient;
    use crate::plan::{DEFAULT_RELEASE_API, DEFAULT_RELEASE_REPO};

    #[test]
    fn stable_resolves_latest_non_prerelease() {
        let mut mock = MockHttpClient::new();
        mock.insert(
            format!(
                "{DEFAULT_RELEASE_API}/repos/{DEFAULT_RELEASE_REPO}/releases/latest"
            ),
            br#"{"tag_name":"v4.0.0","prerelease":false,"draft":false}"#.as_slice(),
        );
        let tag = resolve_channel_tag(Channel::Stable, &mock).unwrap();
        assert_eq!(tag, "v4.0.0");
    }

    #[test]
    fn stable_empty_on_404() {
        struct Status404;
        impl HttpClient for Status404 {
            fn get_bytes(&self, _url: &str) -> CoreResult<Vec<u8>> {
                Err(CoreError::io("http status 404 for download"))
            }
        }
        let err = resolve_channel_tag(Channel::Stable, &Status404).unwrap_err();
        assert_eq!(err.kind(), dare_core::ErrorKind::InvalidInput);
        assert!(err.message().contains(MSG_STABLE_UNAVAILABLE));
    }

    #[test]
    fn beta_picks_first_prerelease() {
        let mut mock = MockHttpClient::new();
        mock.insert(
            format!("{DEFAULT_RELEASE_API}/repos/{DEFAULT_RELEASE_REPO}/releases?per_page=30"),
            br#"[
              {"tag_name":"v4.0.0","prerelease":false,"draft":false},
              {"tag_name":"v4.0.0-rc1","prerelease":true,"draft":false}
            ]"#
            .as_slice(),
        );
        let tag = resolve_channel_tag(Channel::Beta, &mock).unwrap();
        assert_eq!(tag, "v4.0.0-rc1");
    }
}
