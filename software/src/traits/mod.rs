// ABOUTME: Core trait definitions for the Qollective transport architecture
// ABOUTME: Provides internal framework traits for unified transport abstraction

//! Core transport traits for the Qollective framework.
//!
//! This module provides the internal trait architecture that enables unified
//! transport abstraction across all protocol implementations. These traits
//! are used internally by the framework and are not part of the public API.

pub mod catalog;
pub mod handlers;
pub mod receivers;
pub mod senders;

// Re-export internal traits
// Traits are now exported through the prelude module
// pub use senders::UnifiedEnvelopeSender;
// pub use receivers::{UnifiedEnvelopeReceiver};
// pub use handlers::{EnvelopeHandler, ContextDataHandler};

// Tests module
#[cfg(test)]
mod tests;
