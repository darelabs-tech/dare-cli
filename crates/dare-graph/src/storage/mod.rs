//! SQLite and JSON KnowledgeGraph backends.

mod json;
mod sqlite;

pub use json::JsonGraph;
pub use sqlite::SqliteGraph;
