//! Native AST engine with tree-sitter grammars and regex fallback (microplano 035).

mod analyze;
mod extract;
mod language;
mod merge;
mod model;
mod parse;
mod regex_fallback;

pub use analyze::analyze_source;
pub use language::detect_language;
pub use model::{DataModel, Entity, HttpEndpoint, Language, SourceKind, MAX_SOURCE_BYTES};
pub use parse::{grammar_available, parse_source, ParseError};
