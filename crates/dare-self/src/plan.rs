//! Build an [`UpdatePlan`] (schemaVersion 1) for dry-run / apply.

use dare_core::{CoreError, CoreResult};

use crate::channel::Channel;

/// Default GitHub `owner/repo` for release downloads (override via [`ENV_RELEASE_REPO`]).
pub const DEFAULT_RELEASE_REPO: &str = "dewtech/dare-cli";

/// Env override for the GitHub `owner/repo` used in asset URLs.
pub const ENV_RELEASE_REPO: &str = "DARE_SELF_RELEASE_REPO";

/// Stable action list for human / JSON dry-run (BLUEPRINT-053).
pub const PLAN_ACTIONS: &[&str] = &[
    "download",
    "verify-sha256",
    "verify-sig",
    "backup",
    "replace",
];

/// Options for [`plan_update`].
///
/// Provide **either** [`Self::channel`] **or** [`Self::version`] (not both, not neither).
#[derive(Debug, Clone)]
pub struct UpdateOpts {
    /// Channel selection (`beta` / `stable`). Mutually exclusive with [`Self::version`].
    pub channel: Option<Channel>,
    /// Explicit release tag (e.g. `v0.1.0-alpha.2`). Mutually exclusive with [`Self::channel`].
    pub version: Option<String>,
    /// Best-effort current binary version string.
    pub current_version: Option<String>,
    /// Override rustc target triple (tests); default = host triple.
    pub triple: Option<String>,
}

/// Dry-run / apply input plan (schemaVersion 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdatePlan {
    pub schema_version: u32,
    pub channel: String,
    pub current_version: String,
    pub target_tag: String,
    pub target_triple: String,
    pub asset_name: String,
    pub asset_url: String,
    pub sums_url: String,
    pub sig_url: String,
    pub actions: Vec<String>,
}

/// Build an [`UpdatePlan`] from options (no network).
///
/// Channel-only planning (GitHub Releases API resolve) is not wired in this phase —
/// callers must pass [`UpdateOpts::version`] with the target tag.
pub fn plan_update(opts: UpdateOpts) -> CoreResult<UpdatePlan> {
    let (channel_label, tag_raw) = match (&opts.channel, &opts.version) {
        (Some(_), Some(_)) => {
            return Err(CoreError::usage(
                "provide either --channel or --version, not both",
            ));
        }
        (None, None) => {
            return Err(CoreError::usage(
                "provide --channel or --version",
            ));
        }
        (None, Some(ver)) => ("version".to_string(), ver.clone()),
        (Some(_ch), None) => {
            return Err(CoreError::invalid_input(
                "channel release resolution requires a target tag; pass version until GitHub resolve is wired",
            ));
        }
    };

    let target_tag = normalize_tag(&tag_raw)?;
    let target_triple = opts
        .triple
        .clone()
        .unwrap_or_else(host_target_triple);
    if target_triple.trim().is_empty() {
        return Err(CoreError::invalid_input("target triple must not be empty"));
    }

    let asset_name = asset_name_for(&target_tag, &target_triple);
    let base = release_download_base(&target_tag);
    let asset_url = format!("{base}/{asset_name}");
    let sums_url = format!("{base}/SHA256SUMS");
    let sig_url = format!("{base}/SHA256SUMS.sig");

    Ok(UpdatePlan {
        schema_version: 1,
        channel: channel_label,
        current_version: opts.current_version.unwrap_or_default(),
        target_tag,
        target_triple,
        asset_name,
        asset_url,
        sums_url,
        sig_url,
        actions: PLAN_ACTIONS.iter().map(|s| (*s).to_string()).collect(),
    })
}

/// ADR-008 asset file name: `dare-${TAG}-${TARGET}.tar.gz` or `.zip` on Windows triples.
pub fn asset_name_for(tag: &str, triple: &str) -> String {
    let ext = if is_windows_triple(triple) {
        "zip"
    } else {
        "tar.gz"
    };
    format!("dare-{tag}-{triple}.{ext}")
}

fn is_windows_triple(triple: &str) -> bool {
    triple.contains("windows")
}

/// Normalize a release tag: trim; ensure a leading `v` for bare semver-like tags.
fn normalize_tag(raw: &str) -> CoreResult<String> {
    let t = raw.trim();
    if t.is_empty() {
        return Err(CoreError::invalid_input("version/tag must not be empty"));
    }
    if t.starts_with('v') || t.starts_with('V') {
        Ok(format!("v{}", &t[1..]))
    } else {
        Ok(format!("v{t}"))
    }
}

fn release_repo() -> String {
    match std::env::var(ENV_RELEASE_REPO) {
        Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => DEFAULT_RELEASE_REPO.to_string(),
    }
}

fn release_download_base(tag: &str) -> String {
    format!(
        "https://github.com/{}/releases/download/{tag}",
        release_repo()
    )
}

/// Best-effort rustc host triple for the five ADR-008 release targets.
pub fn host_target_triple() -> String {
    #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
    {
        return "x86_64-unknown-linux-gnu".to_string();
    }
    #[cfg(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))]
    {
        return "aarch64-unknown-linux-gnu".to_string();
    }
    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    {
        return "x86_64-apple-darwin".to_string();
    }
    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    {
        return "aarch64-apple-darwin".to_string();
    }
    #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
    {
        return "x86_64-pc-windows-msvc".to_string();
    }
    #[cfg(not(any(
        all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
        all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"),
        all(target_arch = "x86_64", target_os = "macos"),
        all(target_arch = "aarch64", target_os = "macos"),
        all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    )))]
    {
        format!(
            "{}-{}-{}",
            std::env::consts::ARCH,
            std::env::consts::OS,
            std::env::consts::FAMILY
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_asset_name_windows_zip() {
        let plan = plan_update(UpdateOpts {
            channel: None,
            version: Some("v0.1.0-alpha.2".into()),
            current_version: Some("0.1.0-alpha.0".into()),
            triple: Some("x86_64-pc-windows-msvc".into()),
        })
        .unwrap();
        assert_eq!(plan.schema_version, 1);
        assert_eq!(plan.channel, "version");
        assert_eq!(plan.target_tag, "v0.1.0-alpha.2");
        assert_eq!(
            plan.asset_name,
            "dare-v0.1.0-alpha.2-x86_64-pc-windows-msvc.zip"
        );
        assert!(plan.asset_url.ends_with(&plan.asset_name));
        assert!(plan.sums_url.ends_with("/SHA256SUMS"));
        assert!(plan.sig_url.ends_with("/SHA256SUMS.sig"));
        assert_eq!(
            plan.actions,
            PLAN_ACTIONS
                .iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn plan_asset_name_unix_targz() {
        let plan = plan_update(UpdateOpts {
            channel: None,
            version: Some("0.1.0-alpha.2".into()),
            current_version: None,
            triple: Some("x86_64-unknown-linux-gnu".into()),
        })
        .unwrap();
        assert_eq!(plan.target_tag, "v0.1.0-alpha.2");
        assert_eq!(
            plan.asset_name,
            "dare-v0.1.0-alpha.2-x86_64-unknown-linux-gnu.tar.gz"
        );
    }

    #[test]
    fn plan_rejects_both_channel_and_version() {
        let err = plan_update(UpdateOpts {
            channel: Some(Channel::Beta),
            version: Some("v1.0.0".into()),
            current_version: None,
            triple: None,
        })
        .unwrap_err();
        assert_eq!(err.kind(), dare_core::ErrorKind::Usage);
    }

    #[test]
    fn plan_rejects_channel_without_tag() {
        let err = plan_update(UpdateOpts {
            channel: Some(Channel::Stable),
            version: None,
            current_version: None,
            triple: Some("x86_64-unknown-linux-gnu".into()),
        })
        .unwrap_err();
        assert_eq!(err.kind(), dare_core::ErrorKind::InvalidInput);
    }
}
