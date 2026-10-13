//! Summarize prompt — returns the templated user-message body.

pub fn summarize(text: &str) -> String {
    format!("Summarize the following text in 1-2 sentences.\n\n{text}")
}
