use thiserror::Error;

/// Erros de domínio da camada core.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
