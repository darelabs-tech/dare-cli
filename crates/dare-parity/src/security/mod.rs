//! Security regression suite (CI-010..014): injection, env leak, archive, signature, bidi.

mod archive;
mod bidi;
mod env_leak;
mod injection;
mod signature;

pub use archive::test_archive_traversal_fixtures;
pub use bidi::test_bidi_path_rejected;
pub use env_leak::test_env_leak_absent;
pub use injection::test_command_injection_payloads;
pub use signature::test_signature_mismatch_fixtures;
