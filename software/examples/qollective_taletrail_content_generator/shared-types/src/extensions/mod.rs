//! Business logic extensions for generated types
//!
//! This module provides implementation methods for types generated from the JSON schema.
//! Generated types focus on data structures and serialization, while extensions add
//! business logic, validation, and utility methods.

pub mod dag;
pub mod content_node;
pub mod trail;

pub use dag::*;
pub use content_node::*;
pub use trail::*;
