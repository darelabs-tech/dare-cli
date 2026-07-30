//! Application mode: dashboard (read-only) vs REST (legacy).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Read-only dashboard routes only.
    Dashboard,
    /// REST legacy surface (tools/context/blueprint/dag/tasks/graph/…).
    Rest,
}

impl AppMode {
    pub fn as_str(self) -> &'static str {
        match self {
            AppMode::Dashboard => "dashboard",
            AppMode::Rest => "rest",
        }
    }
}
