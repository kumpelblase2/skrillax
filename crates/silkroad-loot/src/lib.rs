//! Definition loading, validation, compilation, and runtime generation for
//! Silkroad loot tables.
//!
//! This crate deliberately has no ECS or agent dependency. Callers compile an
//! immutable [`LootTables`] value from their Media catalogues, then invoke it
//! from their own game-loop adapter.

mod compiler;
mod definition;
mod runtime;

pub use definition::{LootStartupError, LootValidationIssue};
pub use runtime::{LootRandom, LootTables, SystemRandom};
