//! Self-update domain primitives: home layout, channel, and exclusive lock.
//!
//! Download / apply / CLI live in later microplano tasks — this crate only
//! exposes paths, channel parsing, and `update.lock` coordination.

mod channel;
mod lock;
mod paths;

pub use channel::{Channel, ChannelParseError, DEFAULT_CHANNEL};
pub use lock::{
    acquire_lock, force_unlock_if_stale, ForceUnlockError, LockGuard, LockHeld, MSG_LOCK_HELD,
    STALE_LOCK_SECS,
};
pub use paths::{
    PathsError, SelfHome, BACKUP_DIR_NAME, ENV_SELF_HOME, LOCK_NAME, TMP_DIR_NAME,
};
