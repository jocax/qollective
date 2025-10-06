//! Shared types and constants for TaleTrail Content Generator
//!
//! This crate provides common types, constants, and utilities shared
//! across all TaleTrail services.

pub mod constants;
pub mod errors;
pub mod types;
pub mod envelope;

// Re-export commonly used items
pub use constants::*;
pub use errors::{TaleTrailError, Result};
