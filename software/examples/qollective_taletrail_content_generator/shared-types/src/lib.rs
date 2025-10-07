//! Shared types for TaleTrail Content Generator
//!
//! This crate provides common types and utilities shared
//! across all TaleTrail services.
//!
//! Note: Configuration is handled per-service using Figment with
//! Defaults → config.toml → Environment variables hierarchy.

pub mod errors;
pub mod types;
pub mod envelope;
pub mod nats_nkey;
pub mod generated;

// Re-export commonly used items
pub use errors::{TaleTrailError, Result};
pub use nats_nkey::{load_nkey_from_file, connect_with_nkey};

// Re-export generated types for convenience
pub use generated::*;
