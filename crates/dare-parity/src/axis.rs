//! Comparison axes for golden parity cases (RF-03).

use serde::{Deserialize, Serialize};

/// Closed set of dimensions compared by the golden runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompareAxis {
    Exit,
    Stdout,
    Stderr,
    /// Directory listing of relative paths, sorted.
    Tree,
    /// File bytes/text after normalize.
    Content,
    /// `dare.config.json` / `.dare/state.json` subset.
    State,
    /// HTTP status + body (normalized JSON).
    Http,
}
