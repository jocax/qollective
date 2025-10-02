// ABOUTME: Rust code generation module for converting JSON Schema to Rust code
// ABOUTME: Provides direct typify integration for type-safe code generation

pub mod direct_typify;

#[cfg(test)]
mod tests;

// Export the direct typify implementation
pub use direct_typify::*;
