//! Shared Axum application state.

use std::sync::Arc;

use dare_core::ProjectRoot;

use crate::mode::AppMode;

#[derive(Clone)]
pub struct AppState {
    pub root: Arc<ProjectRoot>,
    pub token: Arc<str>,
    pub body_limit: usize,
    pub mode: AppMode,
    pub version: String,
    /// When true, require Bearer even without `ConnectInfo` (oneshot tests).
    pub force_auth: bool,
}

impl AppState {
    pub fn new(
        root: ProjectRoot,
        token: impl Into<Arc<str>>,
        body_limit: usize,
        mode: AppMode,
        version: impl Into<String>,
    ) -> Self {
        Self {
            root: Arc::new(root),
            token: token.into(),
            body_limit,
            mode,
            version: version.into(),
            force_auth: false,
        }
    }
}
