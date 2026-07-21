//! Kill process tree helpers.

use kill_tree::blocking::{kill_tree, kill_tree_with_config};
use kill_tree::Config;

use crate::error::{CoreError, CoreResult};

pub(crate) fn kill_tree_once(pid: u32) -> CoreResult<()> {
    kill_tree(pid)
        .map(|_| ())
        .map_err(|e| CoreError::io(format!("kill_tree failed: {e}")))
}

pub(crate) fn kill_tree_force(pid: u32) -> CoreResult<()> {
    let cfg = Config {
        signal: String::from("SIGKILL"),
        ..Config::default()
    };
    kill_tree_with_config(pid, &cfg)
        .map(|_| ())
        .map_err(|e| CoreError::io(format!("kill_tree force failed: {e}")))
}

/// TERM/default kill, then after `grace` force if still needed (caller polls child).
pub(crate) fn kill_with_grace(pid: u32) -> CoreResult<()> {
    kill_tree_once(pid)?;
    Ok(())
}
