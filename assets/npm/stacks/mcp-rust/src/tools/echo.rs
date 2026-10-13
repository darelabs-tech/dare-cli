//! Echo tool — pure function, unit-tested independently of the SDK.

/// Returns its input verbatim.
///
/// # Errors
/// Returns `Err` when `text` is empty.
pub fn echo(text: &str) -> Result<String, EchoError> {
    if text.is_empty() {
        return Err(EchoError::Empty);
    }
    Ok(text.to_string())
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EchoError {
    #[error("text must be a non-empty string")]
    Empty,
}
