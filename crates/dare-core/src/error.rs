use thiserror::Error;

/// Stable public error classification for CLI exit mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Usage,
    NotFound,
    InvalidInput,
    Config,
    Io,
    Internal,
}

impl ErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorKind::Usage => "Usage",
            ErrorKind::NotFound => "NotFound",
            ErrorKind::InvalidInput => "InvalidInput",
            ErrorKind::Config => "Config",
            ErrorKind::Io => "Io",
            ErrorKind::Internal => "Internal",
        }
    }
}

/// Pure mapping — exhaustive and stable (microplano 004).
pub fn exit_code(kind: ErrorKind) -> i32 {
    match kind {
        ErrorKind::Internal => 1,
        ErrorKind::Usage => 2,
        ErrorKind::NotFound => 3,
        ErrorKind::InvalidInput | ErrorKind::Config => 4,
        ErrorKind::Io => 5,
    }
}

/// Domain errors for dare-core (`thiserror` only — no anyhow).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("{0}")]
    Usage(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InvalidInput(String),
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    Internal(String),
}

impl CoreError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            CoreError::Usage(_) => ErrorKind::Usage,
            CoreError::NotFound(_) => ErrorKind::NotFound,
            CoreError::InvalidInput(_) => ErrorKind::InvalidInput,
            CoreError::Config(_) => ErrorKind::Config,
            CoreError::Io(_) => ErrorKind::Io,
            CoreError::Internal(_) => ErrorKind::Internal,
        }
    }

    pub fn exit_code(&self) -> i32 {
        exit_code(self.kind())
    }

    pub fn message(&self) -> &str {
        match self {
            CoreError::Usage(m)
            | CoreError::NotFound(m)
            | CoreError::InvalidInput(m)
            | CoreError::Config(m)
            | CoreError::Io(m)
            | CoreError::Internal(m) => m.as_str(),
        }
    }

    pub fn usage(msg: impl Into<String>) -> Self {
        CoreError::Usage(crate::redact::redact(&msg.into()))
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        CoreError::NotFound(crate::redact::redact(&msg.into()))
    }

    pub fn invalid_input(msg: impl Into<String>) -> Self {
        CoreError::InvalidInput(crate::redact::redact(&msg.into()))
    }

    pub fn config(msg: impl Into<String>) -> Self {
        CoreError::Config(crate::redact::redact(&msg.into()))
    }

    pub fn io(msg: impl Into<String>) -> Self {
        CoreError::Io(crate::redact::redact(&msg.into()))
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        CoreError::Internal(crate::redact::redact(&msg.into()))
    }
}

pub type CoreResult<T> = Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_mapping_is_stable() {
        assert_eq!(exit_code(ErrorKind::Internal), 1);
        assert_eq!(exit_code(ErrorKind::Usage), 2);
        assert_eq!(exit_code(ErrorKind::NotFound), 3);
        assert_eq!(exit_code(ErrorKind::InvalidInput), 4);
        assert_eq!(exit_code(ErrorKind::Config), 4);
        assert_eq!(exit_code(ErrorKind::Io), 5);
        assert_eq!(ErrorKind::Usage.as_str(), "Usage");
        assert_eq!(CoreError::usage("x").exit_code(), 2);
        assert_eq!(
            CoreError::invalid_input("x").kind(),
            ErrorKind::InvalidInput
        );
    }
}
