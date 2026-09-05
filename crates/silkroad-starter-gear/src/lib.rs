//! Versioned, catalogue-validated starter gear configuration.
//!
//! Definitions are compiled once at startup. Character Selection then resolves
//! the additive sets matching a character's race, clothing family, and weapon.

mod compiler;
mod definition;
mod runtime;

pub use definition::{StarterGearError, ValidationIssue};
pub use runtime::{
    StarterContext, StarterGear, StarterGearResolveError, StarterGearSelectionError, StarterItem, StarterSelection,
};
