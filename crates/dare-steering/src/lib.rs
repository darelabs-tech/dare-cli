//! Steering file discovery: frontmatter, glob resolve, and `.env*` deny.

mod env_deny;
mod frontmatter;
mod report;
mod resolve;

pub use env_deny::is_env_excluded;
pub use frontmatter::{parse_steering_markdown, ParseOutcome, ParsedSteering};
pub use report::{SteeringBlock, SteeringListItem, SteeringListReport, SteeringShowReport};
pub use resolve::{list_steering, show_steering};

pub const STEERING_LIST_SCHEMA: u32 = 1;
pub const STEERING_SHOW_SCHEMA: u32 = 1;
pub const STEERING_DIR_REL: &str = ".dare/steering";
pub const PROJECT_DNA_REL: &str = "DARE/PROJECT-DNA.md";
pub const PATTERNS_REL: &str = "DARE/PATTERNS.md";
pub const PRIORITY_DEFAULT: i32 = 100;
pub const MSG_ENV_EXCLUDED: &str = "steering target excluded: .env* paths are not eligible";
pub const MSG_PATH_ESCAPE: &str = "path escapes project root";
