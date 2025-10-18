//! Constraint Enforcer library
//!
//! This library provides comprehensive constraint enforcement for TaleTrail content generation.
//! It validates vocabulary levels, theme consistency, and required story elements.

pub mod config;
pub mod constraints;
pub mod discovery;
pub mod envelope_handlers;
pub mod requirements;
pub mod server;
pub mod theme;
pub mod vocabulary;

// Re-export envelope handler for external use
pub use envelope_handlers::ConstraintEnforcerHandler;
