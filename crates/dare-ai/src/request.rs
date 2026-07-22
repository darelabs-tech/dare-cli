use dare_core::{ProjectRoot, SafeRelativePath};

#[derive(Debug, Clone)]
pub struct EnrichRequest {
    pub command: String,
    pub title: String,
    pub description: String,
    pub current_markdown: String,
    pub cwd: Option<(ProjectRoot, SafeRelativePath)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnrichRaw {
    pub stdout: String,
    pub stderr_redacted: String,
    pub exit_code: i32,
}
