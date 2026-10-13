//! npm-parity project generator for `dare init` (cycle rust-init-npm-parity-011).
//!
//! Mirrors `@dewtech/dare-cli@3.18.1` `utils/project-generator.ts` and
//! `utils/stack-bootstrap.ts`; deviations are the B-xx fixes listed in
//! `DARE/cycles/rust-init-npm-parity-011/DESIGN.md`.

pub mod bootstrap;
pub mod config;
pub mod console;
pub mod generator;
pub mod ide;
pub mod ordered_json;
pub mod render;
mod stack_table;
pub mod stacks;
pub mod tools;
pub mod writer;

pub use config::*;
