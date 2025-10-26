//! Generated types for TaleTrail Content Generator
//!
//! This crate contains auto-generated types from JSON Schema.
//! Do not edit manually - changes will be overwritten by regenerate-types.sh
//!
//! To regenerate: Run `./regenerate-types.sh` from this crate's root directory.

// Include generated module
mod generated;

// Extension modules for business logic
pub mod constants;
pub mod extensions;
pub mod presets;

// Re-export all generated types
pub use generated::*;

// Re-export extension methods
pub use extensions::*;

// Re-export presets
pub use presets::{PresetError, StoryStructurePreset};
