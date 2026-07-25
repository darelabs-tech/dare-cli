//! Canonical 11-stack registry (BLUEPRINT-046 §0.2–0.3).

use std::sync::OnceLock;

use dare_core::{CoreError, CoreResult};

use crate::trait_api::{GenericScaffolder, StackScaffolder};
use crate::types::{StackKind, StackMetadata, Toolchain, Transport};

/// Exact ASC list of supported stack ids.
pub const STACK_IDS: &[&str] = &[
    "go-gin",
    "go-stdlib",
    "mcp-go",
    "mcp-node-ts",
    "mcp-python",
    "mcp-rust",
    "node-nestjs",
    "php-laravel",
    "python-fastapi",
    "ruby-rails-8",
    "rust-axum",
];

pub const MSG_UNKNOWN_STACK: &str = "unknown stack id";
pub const MSG_HINT_RAILS: &str = "did you mean ruby-rails-8?";

/// Returns the 11 stack ids in canonical ASC order.
pub fn list_stack_ids() -> &'static [&'static str] {
    STACK_IDS
}

/// Resolve a scaffolder by id. Unknown ids → `InvalidInput` containing `unknown stack id`.
/// The alias `rails` also mentions `did you mean ruby-rails-8?`.
pub fn scaffolder_for(id: &str) -> CoreResult<&'static dyn StackScaffolder> {
    for &s in scaffolders() {
        if s.id() == id {
            return Ok(s);
        }
    }
    if id == "rails" {
        return Err(CoreError::InvalidInput(format!(
            "{MSG_UNKNOWN_STACK}: {MSG_HINT_RAILS}"
        )));
    }
    Err(CoreError::InvalidInput(format!(
        "{MSG_UNKNOWN_STACK}: `{id}`"
    )))
}

fn scaffolders() -> &'static [&'static dyn StackScaffolder] {
    static CELL: OnceLock<Vec<&'static dyn StackScaffolder>> = OnceLock::new();
    CELL.get_or_init(|| {
        STACK_IDS
            .iter()
            .map(|&id| {
                let meta = stack_metadata(id);
                let boxed: Box<GenericScaffolder> = Box::new(GenericScaffolder::new(id, meta));
                let leaked: &'static GenericScaffolder = Box::leak(boxed);
                leaked as &'static dyn StackScaffolder
            })
            .collect()
    })
    .as_slice()
}

fn stack_metadata(id: &'static str) -> StackMetadata {
    let (kind, language, rate_limit_rel) = match id {
        "go-gin" => (StackKind::Backend, "go", "internal/ratelimit/limiter.go"),
        "go-stdlib" => (StackKind::Backend, "go", "internal/ratelimit/limiter.go"),
        "mcp-go" => (StackKind::Mcp, "go", "internal/ratelimit/limiter.go"),
        "mcp-node-ts" => (StackKind::Mcp, "typescript", "src/rate-limit.ts"),
        "mcp-python" => (StackKind::Mcp, "python", "app/rate_limit.py"),
        "mcp-rust" => (StackKind::Mcp, "rust", "src/rate_limit.rs"),
        "node-nestjs" => (StackKind::Backend, "typescript", "src/rate-limit.ts"),
        "php-laravel" => (
            StackKind::Backend,
            "php",
            "app/Http/Middleware/RateLimitStarter.php",
        ),
        "python-fastapi" => (StackKind::Backend, "python", "app/rate_limit.py"),
        "ruby-rails-8" => (
            StackKind::Backend,
            "ruby",
            "config/initializers/rack_attack_starter.rb",
        ),
        "rust-axum" => (StackKind::Backend, "rust", "src/rate_limit.rs"),
        _ => unreachable!("STACK_IDS is closed"),
    };

    let default_transport = match kind {
        StackKind::Mcp => Some(Transport::Stdio),
        StackKind::Backend => None,
    };

    StackMetadata {
        id: id.to_string(),
        kind,
        language: language.to_string(),
        default_toolchain: Toolchain::None,
        default_transport,
        template_root: format!("stacks/{id}"),
        rate_limit_rel: rate_limit_rel.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dare_core::CoreError;

    #[test]
    fn registry_lists_eleven_sorted() {
        let ids = list_stack_ids();
        assert_eq!(ids.len(), 11);
        assert_eq!(
            ids,
            &[
                "go-gin",
                "go-stdlib",
                "mcp-go",
                "mcp-node-ts",
                "mcp-python",
                "mcp-rust",
                "node-nestjs",
                "php-laravel",
                "python-fastapi",
                "ruby-rails-8",
                "rust-axum",
            ]
        );
        let mut sorted = ids.to_vec();
        sorted.sort();
        assert_eq!(ids, sorted.as_slice());
    }

    #[test]
    fn scaffolder_unknown() {
        let err = match scaffolder_for("no-such-stack") {
            Ok(_) => panic!("expected error for unknown stack"),
            Err(e) => e,
        };
        match err {
            CoreError::InvalidInput(msg) => {
                assert!(
                    msg.contains(MSG_UNKNOWN_STACK),
                    "expected unknown stack id in `{msg}`"
                );
                assert!(!msg.contains(MSG_HINT_RAILS));
            }
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn scaffolder_rails_hint() {
        let err = match scaffolder_for("rails") {
            Ok(_) => panic!("expected error for rails alias"),
            Err(e) => e,
        };
        match err {
            CoreError::InvalidInput(msg) => {
                assert!(msg.contains(MSG_UNKNOWN_STACK), "msg={msg}");
                assert!(msg.contains(MSG_HINT_RAILS), "msg={msg}");
            }
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn metadata_rate_limit_paths() {
        let cases = [
            ("go-gin", "internal/ratelimit/limiter.go"),
            ("go-stdlib", "internal/ratelimit/limiter.go"),
            ("mcp-go", "internal/ratelimit/limiter.go"),
            ("mcp-node-ts", "src/rate-limit.ts"),
            ("mcp-python", "app/rate_limit.py"),
            ("mcp-rust", "src/rate_limit.rs"),
            ("node-nestjs", "src/rate-limit.ts"),
            (
                "php-laravel",
                "app/Http/Middleware/RateLimitStarter.php",
            ),
            ("python-fastapi", "app/rate_limit.py"),
            (
                "ruby-rails-8",
                "config/initializers/rack_attack_starter.rb",
            ),
            ("rust-axum", "src/rate_limit.rs"),
        ];
        for (id, rate) in cases {
            let s = match scaffolder_for(id) {
                Ok(s) => s,
                Err(e) => panic!("expected scaffolder for {id}: {e}"),
            };
            let meta = s.metadata();
            assert_eq!(meta.id, id);
            assert_eq!(meta.rate_limit_rel, rate);
            assert_eq!(meta.template_root, format!("stacks/{id}"));
            assert_eq!(meta.default_toolchain, Toolchain::None);
            match meta.kind {
                StackKind::Mcp => {
                    assert_eq!(meta.default_transport, Some(Transport::Stdio));
                }
                StackKind::Backend => {
                    assert_eq!(meta.default_transport, None);
                }
            }
        }
    }
}
