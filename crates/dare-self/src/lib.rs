//! Self-update domain: home layout, channel, lock, plan, download, SHA-256 verify.
//!
//! Cosign / apply / CLI live in later microplano tasks.

mod channel;
mod download;
mod lock;
mod paths;
mod plan;
mod verify;

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
    PathsError, SelfHome, BACKUP_DIR_NAME, ENV_SELF_HOME, LOCK_NAME, TMP_DIR_NAME,
};
pub use plan::{
    asset_name_for, host_target_triple, plan_update, UpdateOpts, UpdatePlan, DEFAULT_RELEASE_REPO,
    ENV_RELEASE_REPO, PLAN_ACTIONS,
};
pub use verify::{verify_sha256, MSG_CHECKSUM_MISMATCH};
