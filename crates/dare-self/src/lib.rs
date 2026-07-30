//! Self-update domain: home layout, channel, lock, plan, download, verify, apply.
//!
//! Rollback / uninstall / CLI surface live in later microplano tasks.

mod apply;
mod channel;
mod download;
mod lock;
mod paths;
mod plan;
mod verify;

pub use apply::{
    apply_update, apply_with, backup_binary_path, ApplyFailpoint, ApplyParams, ApplyReport,
};
pub use channel::{Channel, ChannelParseError, DEFAULT_CHANNEL};
pub use download::{
    download_update_artifacts, DownloadedArtifacts, HttpClient, MockHttpClient, RealHttpClient,
    GITHUB_UA,
};
pub use lock::{
    acquire_lock, force_unlock_if_stale, ForceUnlockError, LockGuard, LockHeld, MSG_LOCK_HELD,
    STALE_LOCK_SECS,
};
pub use paths::{
    backup_binary_name, PathsError, SelfHome, BACKUP_DIR_NAME, ENV_SELF_HOME, LOCK_NAME,
    TMP_DIR_NAME,
};
pub use plan::{
    asset_name_for, host_target_triple, plan_update, UpdateOpts, UpdatePlan, DEFAULT_RELEASE_REPO,
    ENV_RELEASE_REPO, PLAN_ACTIONS,
};
pub use verify::{
    allow_unsigned_enabled, reject_if_signing_skipped, timeout_from_env, verify_sha256,
    warn_allow_unsigned, CosignCliVerifier, RejectSkippedVerifier, SignatureVerifier,
    DEFAULT_TIMEOUT_SECS, ENV_ALLOW_UNSIGNED, ENV_COSIGN_IDENTITY, ENV_COSIGN_KEY,
    ENV_COSIGN_OIDC_ISSUER, ENV_TIMEOUT, MSG_ALLOW_UNSIGNED_WARNING, MSG_CHECKSUM_MISMATCH,
    MSG_COSIGN_MISSING, MSG_SIGNING_SKIPPED, SIGNING_SKIPPED_PREFIX,
};
